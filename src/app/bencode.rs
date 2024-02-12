use std::collections::HashMap;
use anyhow::{Result, anyhow};
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Str(Vec<u8>),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
}
pub use Value::{Dict, Int, List, Str};

fn parse_int(buffer: &[u8], start: &mut usize) -> Result<Value> {
    if buffer.get(*start) == Some(&b'i') {
        let end_pos = buffer[*start + 1..]
            .iter()
            .position(|&c| c == b'e')
            .ok_or(anyhow!("Expected 'e' after 'i'"))?
            + *start
            + 1;

        let num = &buffer[*start + 1..end_pos];
        let num_str =
            std::str::from_utf8(num).map_err(|_|anyhow!("Failed to convert string length to utf-8."))?;
        let num = num_str
            .parse::<i64>()
            .map_err(|_| anyhow!("Failed to parse num_str as u64"))?;
        *start = end_pos + 1;
        Ok(Int(num))
    } else {
        Err(anyhow!("Integer encoding must start with 'i'"))
    }
}

fn parse_str(buffer: &[u8], start: &mut usize) -> Result<Value> {
    let input = &buffer[*start..];
    let delimiter = input
        .iter()
        .position(|&c| c == b':')
        .ok_or(anyhow!("Expected to find delimiter : while parsing Vec<u8>."))?;

    let len = std::str::from_utf8(&input[0..delimiter])
        .map_err(|_| anyhow!("Failed to interpred string length as utf-8."))?
        .parse::<usize>()
        .map_err(|_| anyhow!("Failed to parse size length into usize"))?;
    let s = &input[delimiter + 1..delimiter + 1 + len];
    *start += delimiter + 1 + len as usize;
    Ok(Value::Str(s.to_owned()))
}

fn parse_list(buffer: &[u8], start: &mut usize) -> Result<Value> {
    if buffer.get(*start) == Some(&b'l') {
        *start += 1;
        let mut list: Vec<Value> = Vec::new();
        while buffer.get(*start) != Some(&b'e') {
            list.push(parse_bencode(buffer, start)?)
        }

        *start += 1;

        Ok(Value::List(list))
    } else {
        return Err(anyhow!("expected list to start with l"));
    }
}

fn parse_dict(buffer: &[u8], start: &mut usize) -> Result<Value> {
    if buffer.get(*start) == Some(&b'd') {
        *start += 1;
        let mut map: HashMap<String, Value> = HashMap::new();
        while buffer.get(*start) != Some(&b'e') {
            if let Str(key) = parse_bencode(buffer, start)? {
                let value = parse_bencode(buffer, start)?;
                let utf8_key = std::str::from_utf8(&key)
                    .map_err(|_| anyhow!("Failed to parse map key as valid utf-8."))?;
                map.insert(utf8_key.to_owned(), value);
            }
        }
        *start += 1;

        Ok(Dict(map))
    } else {
        return Err(anyhow!("expected list to start with d"));
    }
}

fn parse_bencode(buffer: &[u8], start: &mut usize) -> Result<Value> {
    match &buffer.get(*start) {
        Some(b'i') => parse_int(buffer, start),
        Some(&c) if c.is_ascii_digit() => parse_str(buffer, start),
        Some(b'l') => parse_list(buffer, start),
        Some(b'd') => parse_dict(buffer, start),
        _ => Err(anyhow!("Invalid bencode format or unsupported bencode value.")),
    }
}

pub fn decode(buffer: &[u8]) -> Result<Value> {
    let mut n: usize = 0;
    parse_bencode(buffer, &mut n)
}

#[allow(dead_code)]
fn print_blist(values: &Vec<Value>) {
    print!("[");
    for (i, value) in values.iter().enumerate() {
        print_bvalue(value);
        if i != values.len() - 1 {
            print!(", ");
        }
    }
    print!("]");
}

fn blist_to_string(values: &Vec<Value>) -> Result<String> {
    let mut output = "".to_owned();
    output += "[";
    for (i, value) in values.iter().enumerate() {
        output += &to_string(value)?;
        if i != values.len() - 1 {
            output += ",";
        }
    }
    output += "]";
    Ok(output)
}

#[allow(dead_code)]
fn bdict_to_string_old(values: &HashMap<String, Value>) -> Result<String> {
    let mut output = "".to_owned();
    output += "{";
    let mut sorted_keys = Vec::<String>::new();
    for (key, _) in values.iter() {
        sorted_keys.push(key.to_owned());
    }
    sorted_keys.sort();
    for (i, key) in sorted_keys.iter().enumerate() {
        output += &format!("\"{key}\":");
        output += &to_string(&values[key])?;
        if i != values.len() - 1 {
            output += ",";
        }
    }
    output += "}";
    Ok(output)
}

#[allow(dead_code)]
fn bdict_to_string(values: &HashMap<String, Value>) -> Result<String> {
    let mut sorted_keys: Vec<&String> = values.keys().collect();
    sorted_keys.sort();

    let entries: Result<Vec<String>> = sorted_keys
        .into_iter()
        .map(|key| to_string(&values[key]).map(|value| format!("\"{key}\":{value}")))
        .collect();

    Ok(format!("{{{}}}", entries?.join(",")))
}

#[allow(dead_code)]
fn print_bdict(map: &HashMap<String, Value>) {
    print!("{{");
    for (i, (key, value)) in map.iter().enumerate() {
        print!("\"{}\" : ", key);
        print_bvalue(value);
        if i != map.len() - 1 {
            print!(", ");
        }
    }
    print!("}}");
}

#[allow(dead_code)]
pub fn print_bvalue(value: &Value) {
    match value {
        Int(x) => println!("{:#?}", x),
        Str(s) => println!("{:#?}", s),
        List(list) => println!("{:#?}", list),
        Dict(values) => println!("{:#?}", values),
    }
}

pub fn to_string(value: &Value) -> Result<String> {
    Ok(match value {
        Int(x) => format!("{:?}", x),
        Str(s) => format!(
            "{:?}",
            std::str::from_utf8(s).map_err(|_| anyhow!("Error converting bytes to utf-8"))?
        ),
        List(list) => blist_to_string(&list)?,
        Dict(values) => bdict_to_string(&values)?,
    }
    .to_owned())
}


fn blist_to_vec_u8(values: &Vec<Value>) -> Result<Vec<u8>> {
    let mut output:Vec<u8> = "".as_bytes().to_owned();
    output.push(b'l');
    for (i, value) in values.iter().enumerate() {
        output.extend_from_slice(&to_vec_u8(value)?);
    }
    output.push(b'e');
    Ok(output)
}
#[allow(dead_code)]
fn bdict_to_vecu8(values: &HashMap<String, Value>) -> Result<Vec<u8>> {
    let mut output = "".as_bytes().to_owned();
    output.push(b'd');
    let mut sorted_keys = Vec::<String>::new();
    for (key, _) in values.iter() {
        sorted_keys.push(key.to_owned());
    }
    sorted_keys.sort();
    for (i, key) in sorted_keys.iter().enumerate() {
        output.extend_from_slice(format!("{:?}:", key.len()).as_bytes());
        output.extend_from_slice(key.as_bytes());
        output.extend_from_slice(&to_vec_u8(&values[key])?);
    }
    output.push(b'e');
    Ok(output)
}

pub fn to_vec_u8(value: &Value) -> Result<Vec<u8>> {
    Ok(match value {
        Int(x) => format!("i{:?}e", x).as_bytes().to_owned(),
        Str(s) => [format!("{:?}:", s.len()).as_bytes(), s].concat(),
        List(list) => blist_to_vec_u8(&list)?,
        Dict(values) => bdict_to_vecu8(&values)?,
    }.to_owned())
}



#[cfg(test)]
mod tests {
    use super::*; // Bring everything from the outer module into the scope of the tests module
    #[test]
    fn decode_int_success() {
        let buffer = "i42e";
        assert_eq!(decode(buffer.as_bytes()).unwrap(), Value::Int(42));
    }

    #[test]
    fn decode_str_success() {
        let buffer = "4:spam";
        assert_eq!(
            decode(buffer.as_bytes()).unwrap(),
            Value::Str("spam".to_owned().into())
        );
    }
    #[test]
    fn decode_malformed_int() {
        let buffer = "i42"; // Missing 'e' at the end
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for malformed int"
        );

        let buffer = "ie"; // Missing integer value
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for missing integer value"
        );
    }

    #[test]
    fn decode_malformed_str() {
        let buffer = "4spam"; // Missing ':' delimiter
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for malformed string"
        );

        let buffer = ":spam"; // Missing length
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for missing string length"
        );
    }

    #[test]
    fn decode_malformed_dict() {
        let buffer = "d3:bar4:spam"; // Missing ending 'e' for dict
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for malformed dict"
        );

        let buffer = "d3:bar4:spam3:foo"; // Key without value
        assert!(
            decode(buffer.as_bytes()).is_err(),
            "Expected error for key without value"
        );
    }
    #[test]
    fn decode_nested_list() {
        let buffer = "lli42eei43eee";
        assert_eq!(
            decode(buffer.as_bytes()).unwrap(),
            Value::List(vec![
                Value::List(vec![Value::Int(42)]),
                Value::Int(43)
            ])
        );
    }

    #[test]
    fn decode_nested_dict() {
        let buffer = "d4:dictd3:keyi42eee";
        let mut inner_dict = HashMap::new();
        inner_dict.insert("key".to_owned(), Value::Int(42));

        let mut expected_dict = HashMap::new();
        expected_dict.insert("dict".to_owned(), Value::Dict(inner_dict));

        assert_eq!(
            decode(buffer.as_bytes()).unwrap(),
            Value::Dict(expected_dict)
        );
    }

    #[test]
    fn decode_nested_dict_in_list() {
        let buffer = "li24ed3:keyli3123e3:heli23e3:assi1337eeei23ed3:assi23eee";
        let decoded = decode(buffer.as_bytes()).unwrap();
        let mut vec1: Vec<Value> = Vec::new();
        vec1.push(Int(3123));
        vec1.push(Str("hel".to_owned().into()));
        vec1.push(Int(23));
        vec1.push(Str("ass".to_owned().into()));
        vec1.push(Int(1337)); // Corrected value to match input
        let mut d1 = HashMap::new();
        d1.insert("key".to_owned(), List(vec1));
        let mut outer_vec: Vec<Value> = Vec::new();
        outer_vec.push(Int(24));
        outer_vec.push(Dict(d1));
        outer_vec.push(Int(23));
        outer_vec.push(Dict(HashMap::from([("ass".to_owned(), Int(23))]))); // Correct usage of d2 according to input
        let expected = List(outer_vec);
        assert_eq!(decoded, expected);
    }
}

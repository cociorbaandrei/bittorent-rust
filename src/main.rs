mod app;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let args = env::args().collect();

    app::entrypoint(args)
}

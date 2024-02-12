mod app;
use std::env;
use anyhow::{Result};


fn main() -> Result<()> {
    let args = env::args().collect();
    app::entrypoint(args)
}

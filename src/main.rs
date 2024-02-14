mod app;
use std::env;
use anyhow::{Result};
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = env::args().collect();
    app::entrypoint(args).await
}

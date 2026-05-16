use anyhow::Result;
use clap::Parser;

mod auth;
mod cli;
mod config;
mod gh;
mod output;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Cli::parse();
    cli::run(args).await
}

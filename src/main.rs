mod cli;
mod config;
mod forum;
mod gh;

use clap::Parser;
use geek_cli::output;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    let format = args.format;
    if let Err(err) = cli::run(args).await {
        output::report_error(&err, format);
    }
}

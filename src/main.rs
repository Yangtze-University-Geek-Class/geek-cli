use clap::Parser;

mod api;
mod cli;
mod commands;
mod config;
mod forum;
mod gh;
mod output;
mod session;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    let format = args.format;
    if let Err(err) = cli::run(args).await {
        report(&err, format);
    }
}

fn report(err: &anyhow::Error, format: output::Format) -> ! {
    if let Some(api) = err.downcast_ref::<api::ApiError>() {
        match format {
            output::Format::Json | output::Format::Pretty => {
                let line = serde_json::to_string(api).unwrap_or_else(|_| api.to_string());
                eprintln!("{line}");
            }
            output::Format::Table => eprintln!("{api}"),
        }
        std::process::exit(api.exit_code());
    }
    if let Some(usage) = err.downcast_ref::<api::UsageError>() {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    eprintln!("{err:#}");
    std::process::exit(1);
}

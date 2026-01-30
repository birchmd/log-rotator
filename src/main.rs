use {
    clap::Parser,
    log_rotator::{config::Config, log_redirect},
    tokio::io::{BufReader, stdin},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let input = BufReader::new(stdin());

    log_redirect(input, &config).await?;

    Ok(())
}

use {clap::Parser, std::path::PathBuf};

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(short, long)]
    pub dir: PathBuf,
    #[arg(short, long)]
    pub prefix: String,
    #[arg(short, long, default_value("5"))]
    pub delete_after: Option<u8>,
}

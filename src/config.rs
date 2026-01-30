use {clap::Parser, std::path::PathBuf};

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(short, long)]
    pub dir: PathBuf,
    #[arg(short, long)]
    pub prefix: String,
}

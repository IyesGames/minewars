use crate::prelude::*;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,
    #[arg(short, long)]
    pub debug: bool,
    #[arg(long, value_name = "FILE")]
    pub log: Option<PathBuf>,
    #[arg(short, long, value_name = "FILE")]
    pub session: Vec<PathBuf>,
}

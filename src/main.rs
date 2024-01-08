use clap::Parser;
use rustyline::Result;

use feather_repl::repl::{repl, ReplOptions};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long)]
    estimate: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    repl(ReplOptions::default().with_estimate(args.estimate))
}

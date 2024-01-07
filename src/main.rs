use clap::Parser;
use rustyline::Result;

use reel::repl::{repl, ReplOptions};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long)]
    each_expr: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    repl(ReplOptions::default().with_each_expr(args.each_expr))
}

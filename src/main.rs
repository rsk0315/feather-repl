use clap::Parser;
use rustyline::Result;

use feather_repl::repl::{repl, ReplOptions};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Which subexpressions to estimate.
    /// Valid values: "+lit" for literals, "+par" for parentheses,
    /// "+bin" for binary operations, or the comma-separated value of these.
    #[arg(short, long)]
    estimate: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    repl(ReplOptions::default().with_estimate(args.estimate))
}

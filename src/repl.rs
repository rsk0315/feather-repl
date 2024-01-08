use combine::EasyParser;
use homedir::get_my_home;
use rustyline::{
    config::{Behavior, Config},
    error::ReadlineError,
    DefaultEditor,
};

use crate::{
    ast::EvalOptions,
    constants::AUX_COLOR,
    parser::parse_line,
    ui::{backmatter, error_report, frontmatter},
    utils::StrPaint,
};

pub struct ReplOptions {
    estimate: Vec<String>,
}

impl Default for ReplOptions {
    fn default() -> Self { Self::new() }
}

impl ReplOptions {
    pub fn new() -> Self { Self { estimate: vec![] } }

    pub fn with_estimate(mut self, arg: Vec<String>) -> Self {
        self.estimate = arg;
        self
    }
}

pub fn repl(opts: ReplOptions) -> rustyline::Result<()> {
    let cfg = Config::builder().behavior(Behavior::PreferTerm).build();
    let mut rl = DefaultEditor::with_config(cfg)?;

    let histfile = get_my_home().unwrap().unwrap().join(".float_repl_history");

    if rl.load_history(&histfile).is_err() {
        eprintln!("No previous history.");
    }

    let mut opts = EvalOptions::default().with_estimate(opts.estimate);
    for nl in 1.. {
        let readline = rl.readline(&">> ".fg(AUX_COLOR).to_string());
        match readline {
            Ok(line) if line.trim().is_empty() => {}
            Ok(line) if line.starts_with(":") => {
                rl.add_history_entry(line.to_owned())?;
                opts.update(&line[1..]);
            }
            Ok(line) => {
                rl.add_history_entry(line.to_owned())?;
                frontmatter("stdin", nl);
                match parse_line().easy_parse(line.as_str()) {
                    Ok(ast) => backmatter(&line, ast.0.eval(&line, &opts, 0)),
                    Err(e) => error_report(e, &line),
                }
            }

            Err(ReadlineError::Interrupted) => {
                eprintln!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                continue;
            }
        }
    }

    rl.save_history(&histfile)?;

    Ok(())
}

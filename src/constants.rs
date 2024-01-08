use yansi::Color;

pub const EMPH_COLOR: Color = Color::Fixed(201);
pub const AUX_COLOR: Color = Color::Fixed(3);
pub const DARK_COLOR: Color = Color::Fixed(246);
pub const ERR_COLOR: Color = Color::Fixed(9);
pub const OK_COLOR: Color = Color::Fixed(10);

pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub const PROLOGUE: &str = r#"Welcome to feather REPL. Type ":help" for help."#;

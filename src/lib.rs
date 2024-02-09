use clap::{ColorChoice, Parser};
use once_cell::sync::Lazy;

pub mod commands;
pub mod git;
pub mod open_ai;
pub mod prompt;
pub mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// OenAI API key
    #[arg(short, long, env)]
    pub openai_api_key: String,

    // TODO: Find more elegant way to get the default value
    /// Path to the git repository
    #[arg(short, long, default_value_t = String::from(std::env::current_dir().unwrap().to_string_lossy()))]
    pub path: String,

    /// OpenAI model to use
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    pub model: String,

    /// Number of options to generate. In non-interactive mode, optins will be separated by newline
    #[arg(short, long, default_value = "3")]
    pub results: usize,

    /// Will generate one message, print to stdout and exit
    #[arg(short, long, default_value = "false")]
    pub non_interactive: bool,

    /// Colorize output
    #[arg(short, long, default_value = "auto")]
    pub color: ColorChoice,

    /// print debug information such as prompt
    #[arg(short, long, default_value = "false")]
    pub debug: bool,
}

pub static ARGS: Lazy<Args> = Lazy::new(|| {
    let args = Args::parse();
    console::set_colors_enabled(args.color == ColorChoice::Always);
    args
});

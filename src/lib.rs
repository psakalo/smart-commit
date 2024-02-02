use clap::Parser;
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

    /// Number of options to generate in interactive model
    #[arg(short, long, default_value = "5", conflicts_with = "non_interactive")]
    pub results: usize,

    /// Will generate one message, print to stdout and exit
    #[arg(short, long, default_value = "false")]
    pub non_interactive: bool,
}

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);

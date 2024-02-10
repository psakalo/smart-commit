use std::process;

use smart_commit::{
    args::ARGS,
    commands::{run_interactive, run_non_interactive},
};

fn main() {
    if let Err(err) = match ARGS.non_interactive {
        true => run_non_interactive(),
        false => run_interactive(),
    } {
        eprintln!("Error occured: {}", err);
        process::exit(1);
    }
}

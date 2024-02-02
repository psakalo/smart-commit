use std::error::Error;

use crate::{prompt::CommitCompletionData, tui::Input, tui::OptionsMenu, ARGS};

pub fn run_interactive() -> Result<(), Box<dyn Error>> {
    let mut completion_data =
        CommitCompletionData::from_path(&ARGS.path, &ARGS.model, ARGS.results)?;
    let mut results = completion_data.complete_commit_messages()?;

    let choice = loop {
        let mut options: Vec<String> = results
            .iter()
            .map(|r| format!("Accept \"{}\"", r))
            .collect();

        options.push(String::from("Refine prompt"));

        let menu = OptionsMenu::new(
            String::from("Choose from generated options, or refine prompt"),
            options,
        );

        let choice = menu.read_answer()?;

        if choice == results.len() {
            let input = Input::new(String::from("What would you like to change"));
            let new_prompt = input.read_answer()?;
            results = completion_data.refine_commit_messages(&new_prompt)?;
        } else {
            break choice;
        }
    };

    println!("{}", results[choice]);

    Ok(())
}

pub fn run_non_interactive() -> Result<(), Box<dyn Error>> {
    Ok(())
}

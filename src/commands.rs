use std::error::Error;

use crate::{prompt::CommitCompletionData, tui::Input, tui::OptionsMenu, ARGS};

pub fn run_interactive() -> Result<(), Box<dyn Error>> {
    let mut completion_data = CommitCompletionData::from_path(&ARGS.path, &ARGS.model)?;
    let mut results = completion_data.complete_commit_messages(ARGS.results, None)?;

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

        // Refine prompt was sslected
        if choice == results.len() {
            let input = Input::new(String::from("What would you like to change"));
            let new_prompt = input.read_answer(&completion_data.refinement_prompt.as_deref())?;
            results = completion_data.complete_commit_messages(ARGS.results, Some(new_prompt))?;
        } else {
            break choice;
        }
    };

    print!("{}", results[choice]);

    Ok(())
}

pub fn run_non_interactive() -> Result<(), Box<dyn Error>> {
    let mut completion_data = CommitCompletionData::from_path(&ARGS.path, &ARGS.model)?;
    let results = completion_data.complete_commit_messages(ARGS.results, None)?;

    println!("{}", results.join("\n"));

    Ok(())
}

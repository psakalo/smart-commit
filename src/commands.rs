use anyhow::Result;
use inquire::{Select, Text};

use crate::{args::ARGS, open_ai, prompt::CommitCompletionData};

pub fn run_interactive() -> Result<()> {
    let openai_client = open_ai::OpenAIClient::new(open_ai::OpenAIKey::new_ensure()?);
    let mut completion_data = CommitCompletionData::from_path(&ARGS.path, &ARGS.model)?;
    let mut results =
        completion_data.complete_commit_messages(&openai_client, ARGS.results, None)?;

    let choice = loop {
        let mut options: Vec<String> = results
            .iter()
            .map(|r| format!("Accept \"{}\"", r))
            .collect();

        options.push(String::from("Refine prompt"));

        let choice = Select::new("Choose from generated options, or refine prompt:", options)
            .raw_prompt()?
            .index;

        // Refine prompt was sslected
        if choice == results.len() {
            let mut refinement_prompt = String::new();

            if let Some(prompt) = completion_data.refinement_prompt.as_ref() {
                refinement_prompt.push_str(prompt);
            }

            let refinement_prompt = Text::new("What would you like to change:")
                .with_initial_value(&refinement_prompt)
                .prompt()?;

            results = completion_data.complete_commit_messages(
                &openai_client,
                ARGS.results,
                Some(refinement_prompt),
            )?;
        } else {
            break choice;
        }
    };

    print!("{}", results[choice]);

    Ok(())
}

pub fn run_non_interactive() -> Result<()> {
    let openai_client = open_ai::OpenAIClient::new(open_ai::OpenAIKey::new_ensure()?);
    let mut completion_data = CommitCompletionData::from_path(&ARGS.path, &ARGS.model)?;
    let results = completion_data.complete_commit_messages(&openai_client, ARGS.results, None)?;

    println!("{}", results.join("\n"));

    Ok(())
}

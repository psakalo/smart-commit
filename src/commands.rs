use anyhow::Result;
use inquire::{Select, Text};

use crate::{
    args::ARGS,
    open_ai,
    prompt::{common::PromptData, common::PromptGenerator, single_request::SingleRequestPrompt},
};

pub fn run_interactive() -> Result<()> {
    let openai_client = open_ai::OpenAIClient::new(open_ai::OpenAIKey::new_ensure()?);
    let repo = git2::Repository::open(&ARGS.path)?;
    let completion_data = PromptData::from_repo(&repo, &openai_client, &ARGS.model)?;
    let mut prompt = SingleRequestPrompt::from(completion_data);
    let mut results = prompt.generate_results(ARGS.results)?;

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

            if let Some(prompt) = prompt.get_refinement_prompt().as_ref() {
                refinement_prompt.push_str(prompt);
            }

            let refinement_prompt = Text::new("What would you like to change:")
                .with_initial_value(&refinement_prompt)
                .prompt()?;

            prompt.update_refinement_prompt(refinement_prompt);
            results = prompt.generate_results(ARGS.results)?;
        } else {
            break choice;
        }
    };

    print!("{}", results[choice]);

    Ok(())
}

pub fn run_non_interactive() -> Result<()> {
    let openai_client = open_ai::OpenAIClient::new(open_ai::OpenAIKey::new_ensure()?);
    let repo = git2::Repository::open(&ARGS.path)?;
    let completion_data = PromptData::from_repo(&repo, &openai_client, &ARGS.model)?;
    let prompt = SingleRequestPrompt::from(completion_data);
    let results = prompt.generate_results(ARGS.results)?;

    println!("{}", results.join("\n"));

    Ok(())
}

use std::process;

use smart_commit::open_ai::get_completion;
use smart_commit::prompt::CommitCompletionData;

fn main() {
    println!("Getting completion");

    let completion_data =
        CommitCompletionData::from_path(&String::from("/Users/pavlosakalo/Developer/smart-commit"))
            .unwrap_or_else(|err| {
                eprintln!("Error getting completion data: {}", err);
                process::exit(1);
            });

    let completion_data = completion_data;
    let prompt = completion_data.build_prompt().unwrap_or_else(|err| {
        eprintln!("Error building prompt: {}", err);
        process::exit(1);
    });

    println!("Prompt:\n{}", prompt);

    let commit_message = get_completion("gpt-3.5-turbo", &prompt).unwrap_or_else(|err| {
        eprintln!("Error getting completion: {}", err);
        process::exit(1);
    });

    println!("{}", commit_message.message.content.unwrap());
}

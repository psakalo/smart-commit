use std::{error::Error, process::Command};

pub fn get_log_messages(dir: &String, limit: u32) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%s",
            &format!("--max-count={}", limit),
        ])
        .current_dir(dir)
        .output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

pub fn run_commit(
    dir: &String,
    message: &str,
    extra_args: &[String],
) -> Result<(), Box<dyn Error>> {
    let mut args = vec![
        String::from("commit"),
        String::from("-m"),
        message.to_string(),
    ];

    args.extend_from_slice(extra_args);

    Command::new("git")
        // pass -F - and args
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    Ok(())
}

fn get_staged_files(dir: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--diff-filter=ACDMRTUXB", "--name-only"])
        .current_dir(dir)
        .output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

fn get_file_diff(dir: &String, file: &String, context: usize) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["diff", "--cached", format!("-U{}", context).as_str(), file])
        .current_dir(dir)
        .output()?;
    let output = String::from_utf8(output.stdout)?;
    Ok(output)
}

const DEFAULT_SURROUNDING_LINES: usize = 0;

/// This function tries to do it's best to evenly trim all staged diffs to not exceed total_length
/// First full diff is generated, and if it doesn't exceed total_length, it's returned as is
/// Next, new diffs are requested with reduced context lines. If total_length is still exceeded,
/// diffs are sorted by length, and the longest is trimmed by 1 line from start and end
/// until total_length is reached
pub fn get_staged_diff(dir: &String, total_length: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let files = get_staged_files(dir)?;

    let diff: Vec<String> = files
        .iter()
        .map(|file| get_file_diff(dir, file, DEFAULT_SURROUNDING_LINES))
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;

    let current_length: usize = diff.iter().fold(0, |acc, x| acc + x.len());
    if current_length <= total_length {
        return Ok(diff);
    }

    // Get diff with reduced context lines
    let mut diff = files
        .iter()
        .map(|file| get_file_diff(dir, file, 0))
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;

    loop {
        let current_length: usize = diff.iter().fold(0, |acc, x| acc + x.len());
        if current_length <= total_length {
            return Ok(diff);
        }

        diff.sort_by(|a, b| a.len().cmp(&b.len()));

        let longest_diff = diff.pop().unwrap();
        let trimmed_diff = longest_diff
            .lines()
            .skip(1)
            .take(longest_diff.lines().count() - 2)
            .collect::<Vec<&str>>()
            .join("\n");

        diff.push(trimmed_diff);
    }
}

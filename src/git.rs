use std::{error::Error, process::Command};

pub fn get_log_messages(dir: &String, limit: u32) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(&[
            "log",
            "--pretty=format:%s",
            &format!("--max-count={}", limit),
        ])
        .current_dir(dir)
        .output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

fn get_staged_files(dir: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(&["diff", "--name-only"])
        .current_dir(dir)
        .output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

const MAX_LINES_PER_FILE: usize = 50;

fn trim_diff(diff: String, max_lines: usize) -> String {
    let mut trimmed_diff = String::new();
    let mut line_count: usize = 0;

    for line in diff.lines() {
        if line_count >= max_lines {
            let remaining_lines = diff.lines().count() - line_count;
            trimmed_diff.push_str(&format!(
                "\n<this diff has {} more lines, which were trimmed to save space>",
                remaining_lines
            ));
            break;
        }

        trimmed_diff.push_str(line);
        trimmed_diff.push('\n');
        line_count += 1;
    }

    trimmed_diff
}

pub fn get_staged_diff(dir: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let files = get_staged_files(dir)?;

    let mut diff = Vec::new();

    for file in files {
        let output = Command::new("git")
            .args(&["diff", "-U10", &file])
            .current_dir(dir)
            .output();

        if let Ok(output) = output {
            if let Ok(output) = String::from_utf8(output.stdout) {
                diff.push(trim_diff(output, MAX_LINES_PER_FILE));
            }
        }
    }

    Ok(diff)
}

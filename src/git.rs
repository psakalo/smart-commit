use std::{collections::HashMap, fmt};

use git2::{Delta, DiffFormat, Repository};

pub enum GitError {
    Git2Error(git2::Error),
    DiffProcessingError,
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        GitError::Git2Error(e)
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitError::Git2Error(e) => write!(f, "Git2 error: {}", e),
            GitError::DiffProcessingError => write!(f, "Diff processing error"),
        }
    }
}

pub fn get_log_messages(dir: &str, limit: usize) -> Result<Vec<String>, GitError> {
    let repo = Repository::open(dir)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    revwalk
        .take(limit)
        .map(|id| {
            id.and_then(|id| repo.find_commit(id))
                .map(|commit| commit.summary().unwrap_or_default().to_string())
                .map_err(GitError::from)
        })
        .collect::<Result<Vec<String>, GitError>>()
}

#[derive(PartialEq, Clone, Copy)]
pub enum FileDiffType {
    Added,
    Deleted,
    Modified,
}

pub struct FileDiff {
    pub delta_type: FileDiffType,
    pub formatted_diff: String,
}

pub fn get_staged_diff(dir: &str) -> Result<HashMap<String, FileDiff>, GitError> {
    let repo = Repository::open(dir)?;
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), None, None)?;

    let mut file_diffs: HashMap<String, FileDiff> = HashMap::new();

    diff.print(DiffFormat::Patch, |delta, _hunk, line| {
        let file_path = delta
            .new_file()
            .path()
            .map(|p| p.to_string_lossy().to_string());
        if file_path.is_none() {
            return false;
        }
        let file_path = file_path.unwrap();

        let delta_type = match delta.status() {
            Delta::Added => FileDiffType::Added,
            Delta::Deleted => FileDiffType::Deleted,
            Delta::Modified => FileDiffType::Modified,
            _ => return false,
        };

        let file_diff = file_diffs.entry(file_path).or_insert_with(|| FileDiff {
            delta_type,
            formatted_diff: String::new(),
        });

        // I don't know if this can happen, but lets be safe
        if file_diff.delta_type != delta_type {
            return false;
        }

        file_diff.formatted_diff.push_str(
            format!(
                "{}{}",
                line.origin(),
                String::from_utf8_lossy(line.content())
            )
            .as_ref(),
        );

        true
    })?;

    Ok(file_diffs)
}

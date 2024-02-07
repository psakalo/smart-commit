use std::{
    error::Error,
    path::{self},
};

/// Createa a clone of the current git repository in a temp dir
fn clone_repo(temp_dir: &tempdir::TempDir) -> Result<git2::Repository, Box<dyn Error>> {
    let repo = git2::Repository::open_from_env()?;
    let target_path = temp_dir.path().to_path_buf();

    Ok(git2::build::RepoBuilder::new()
        .clone(repo.workdir().unwrap().to_str().unwrap(), &target_path)?)
}

const TEST_COMMIT_SHA: &str = "c0fe7a4968366fbf7965266ea29dc106ab9aec6f";

/// Create branch from TEST_COMMIT_SHA, soft reset last commit, and stage changes
fn setup_repo(repo: &mut git2::Repository, stage: bool) -> Result<path::PathBuf, Box<dyn Error>> {
    let test_commit = repo.find_commit(git2::Oid::from_str(TEST_COMMIT_SHA)?)?;

    let branch_name = "smart-commit-test";
    let branch = repo.branch(branch_name, &test_commit, false)?;

    repo.set_head(branch.get().name().unwrap())?;

    repo.reset(&repo.revparse_single("HEAD")?, git2::ResetType::Hard, None)?;
    repo.reset(&repo.revparse_single("HEAD^")?, git2::ResetType::Soft, None)?;

    if stage {
        let mut index = repo.index()?;
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
    }

    Ok(repo.workdir().unwrap().to_path_buf())
}

/// Setup a git repo for testing
pub fn setup_git_repo(temp_dir: &tempdir::TempDir) -> Result<path::PathBuf, Box<dyn Error>> {
    let mut repo = clone_repo(temp_dir)?;

    setup_repo(&mut repo, true)?;

    Ok(repo.workdir().unwrap().to_path_buf())
}

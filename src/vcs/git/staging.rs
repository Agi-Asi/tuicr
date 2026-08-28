use git2::Repository;
use std::path::Path;

use crate::error::Result;

pub fn stage_file(repo: &Repository, path: &Path) -> Result<()> {
    let mut index = repo.index()?;
    // `add_path` stats the worktree, so it cannot stage a tracked file that has
    // been deleted. When the path is still in the index but missing on disk,
    // stage the deletion with `remove_path` instead. New (untracked) files and
    // modified files keep the existing `add_path` path.
    if index.get_path(path, 0).is_some() && !path.is_file() {
        index.remove_path(path)?;
    } else {
        index.add_path(path)?;
    }
    index.write()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn stage_file_adds_to_index() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let repo = Repository::init(temp_dir.path()).expect("failed to init repo");

        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello\n").unwrap();

        stage_file(&repo, Path::new("test.txt")).unwrap();

        let index = repo.index().unwrap();
        assert!(index.get_path(Path::new("test.txt"), 0).is_some());
    }

    #[test]
    fn stage_file_stages_a_deleted_tracked_file() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let repo = Repository::init(temp_dir.path()).expect("failed to init repo");

        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        fs::remove_file(&file_path).unwrap();

        stage_file(&repo, Path::new("test.txt")).unwrap();

        let statuses = repo.statuses(None).unwrap();
        let status = statuses
            .iter()
            .find(|entry| entry.path() == Some("test.txt"))
            .expect("deleted file should appear in status");
        assert!(status.status().contains(git2::Status::INDEX_DELETED));
    }
}

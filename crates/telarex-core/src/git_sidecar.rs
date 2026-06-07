use std::path::Path;
use anyhow::Result;

/// A lightweight Git wrapper for sidecar operations.
///
/// Provides status, stage, commit, push, fetch, and log operations
/// without interfering with the primary CRDT-based sync system.
pub struct GitSidecar {
    repo: git2::Repository,
}

/// The current state of the working tree relative to HEAD.
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: String,
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
}

impl GitSidecar {
    /// Open or create a Git repository at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = if path.join(".git").exists() {
            git2::Repository::open(path)?
        } else {
            git2::Repository::init(path)?
        };
        Ok(Self { repo })
    }

    /// Return the working tree status (branch, modified, staged, untracked, ahead/behind).
    pub fn status(&self) -> Result<GitStatus> {
        let branch = self
            .repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "HEAD".to_string());

        let statuses = self.repo.statuses(None)?;
        let mut modified = 0usize;
        let mut staged = 0usize;
        let mut untracked = 0usize;

        for entry in statuses.iter() {
            let s = entry.status();
            if s.contains(git2::Status::CURRENT) { continue; }
            if s.intersects(git2::Status::INDEX_MODIFIED | git2::Status::INDEX_NEW) {
                staged += 1;
            }
            if s.intersects(git2::Status::WT_MODIFIED | git2::Status::WT_DELETED) {
                modified += 1;
            }
            if s.contains(git2::Status::WT_NEW) {
                untracked += 1;
            }
        }

        let (ahead, behind) = self.count_ahead_behind()?;

        Ok(GitStatus { branch, modified, staged, untracked, ahead, behind })
    }

    fn count_ahead_behind(&self) -> Result<(usize, usize)> {
        let head = self.repo.head().ok().and_then(|h| h.target());
        let branch = self.repo.head().ok().and_then(|h| h.shorthand().map(|s| s.to_string()));
        let upstream = branch.as_ref().and_then(|b| {
            self.repo.find_branch(b, git2::BranchType::Local).ok()
                .and_then(|b| b.upstream().ok())
                .and_then(|u| u.get().target())
        });

        match (head, upstream) {
            (Some(h), Some(u)) => {
                let (a, b) = self.repo.graph_ahead_behind(h, u)?;
                Ok((a as usize, b as usize))
            }
            _ => Ok((0, 0)),
        }
    }

    /// Stage specific paths for the next commit.
    pub fn stage(&self, paths: &[&Path]) -> Result<()> {
        let mut index = self.repo.index()?;
        for path in paths {
            index.add_path(path)?;
        }
        index.write()?;
        Ok(())
    }

    /// Stage all changes (new, modified, deleted) for the next commit.
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Unstage all currently staged changes.
    pub fn unstage_all(&self) -> Result<()> {
        let head = self.repo.head()?;
        let tree = head.peel_to_tree()?;
        let mut index = self.repo.index()?;
        index.read_tree(&tree)?;
        index.write()?;
        Ok(())
    }

    /// Create a commit with the given message. Returns the commit OID.
    pub fn commit(&self, message: &str) -> Result<String> {
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let parent_commit = self.repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        let signature = git2::Signature::now("TelaRex User", "user@telarex")?;
        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        Ok(oid.to_string())
    }

    /// Push the given branch to the named remote.
    pub fn push(&self, remote: &str, branch: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.push_update_reference(|_refname, status| {
            if let Some(msg) = status {
                Err(git2::Error::from_str(msg))
            } else {
                Ok(())
            }
        });
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);
        remote.push(&[&refspec], Some(&mut opts))?;
        Ok(())
    }

    /// Fetch latest references from the named remote.
    pub fn fetch(&self, remote: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote)?;
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.transfer_progress(|_| true);
        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(callbacks);
        remote.fetch(&["refs/heads/*:refs/heads/*"], Some(&mut opts), None)?;
        Ok(())
    }

    /// Return up to `max_count` recent commits.
    pub fn log(&self, max_count: usize) -> Result<Vec<GitCommit>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        for oid in revwalk.take(max_count) {
            if let Ok(oid) = oid {
                if let Ok(commit) = self.repo.find_commit(oid) {
                    commits.push(GitCommit {
                        oid: oid.to_string(),
                        message: commit.message().unwrap_or("").to_string(),
                        author: commit.author().name().unwrap_or("unknown").to_string(),
                        time: commit.time().seconds(),
                    });
                }
            }
        }
        Ok(commits)
    }
}

/// A single commit entry returned by [`GitSidecar::log`].
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub oid: String,
    pub message: String,
    pub author: String,
    pub time: i64,
}

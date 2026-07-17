use std::process::Command;
use std::path::PathBuf;

/// A git repository, rooted at a specific path.
///
/// `Repo::open()` discovers the root from the current working directory
/// (production use). `Repo::open_at(path)` uses an explicit path with no
/// discovery, so each test can point at its own disposable temp repo.
pub struct Repo {
    root: PathBuf,
}

impl Repo {
    /// Discover the repo root from the current working directory.
    ///
    /// Returns an error if the `git` binary itself can't be run (e.g. not
    /// installed / not on PATH). If `git` runs but we're outside a repo,
    /// falls back to the current directory instead of failing, so the app
    /// doesn't crash just because it was opened outside a repo.
    pub fn open() -> Result<Self, String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| format!("Could not run 'git'. Is it installed and on your PATH? ({})", e))?;

        let root = if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            PathBuf::from(path_str)
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        Ok(Repo { root })
    }

    /// Use an explicit path, no discovery. For tests.
    pub fn open_at(path: impl Into<PathBuf>) -> Self {
        Repo { root: path.into() }
    }

    /// Run a git command and return the trimmed output.
    /// Use this for almost everything (getting branch names, hashes, etc.)
    fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute git: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Run a git command and return the RAW output (preserving whitespace).
    /// Use this ONLY when column alignment matters (like `git status`).
    fn run_raw(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute git: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Get the status of the repository (porcelain format)
    pub fn status_porcelain(&self) -> Result<Vec<(String, String)>, String> {
        let output = self.run_raw(&["status", "--porcelain=v1"])?;

        if output.is_empty() {
            return Ok(Vec::new());
        }

        let files = output
            .lines()
            .map(|line| {
                if line.len() < 4 {
                    return ("?".to_string(), line.to_string());
                }
                let (status, path) = line.split_at(3);
                (status.trim().to_string(), path.trim().to_string())
            })
            .collect();

        Ok(files)
    }

    /// Check if local branch is ahead/behind remote
    pub fn ahead_behind(&self, branch: &str) -> Result<(usize, usize), String> {
        let arg = format!("{}...@{{u}}", branch);
        let output = self.run(&["rev-list", "--left-right", "--count", &arg])?;

        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() >= 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            Ok((ahead, behind))
        } else {
            Ok((0, 0))
        }
    }

    /// Push the current branch to origin, establishing a tracking link
    pub fn push_upstream(&self, branch: &str) -> Result<String, String> {
        self.run(&["push", "-u", "origin", branch])
    }

    /// Set the upstream for a branch
    pub fn set_remote(&self, branch: &str, remote: &str) -> Result<String, String> {
        let upstream = format!("{}/{}", remote, branch);
        self.run(&["branch", "--set-upstream-to", &upstream, branch])
    }

    /// Switch to an existing branch
    pub fn switch_branch(&self, name: &str) -> Result<String, String> {
        self.run(&["checkout", name])
    }

    /// Create and switch to a new branch
    pub fn create_branch(&self, name: &str) -> Result<String, String> {
        self.run(&["checkout", "-b", name])
    }

    /// Fetch latest changes/branches from remote (without merging)
    pub fn fetch(&self) -> Result<String, String> {
        self.run(&["fetch"])
    }

    /// List remote branches with details
    pub fn list_remote_branches(&self) -> Result<Vec<String>, String> {
        let output = self.run(&[
            "for-each-ref",
            "--format=%(refname:short)|%(authorname)|%(authordate:relative)",
            "refs/remotes/",
        ])?;

        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    /// Pull a specific branch from origin
    pub fn pull_branch(&self, branch_name: &str) -> Result<String, String> {
        let clean_name = branch_name.trim_start_matches("origin/");
        self.run(&["pull", "origin", clean_name])
    }

    /// Stash current changes
    pub fn stash_push(&self) -> Result<String, String> {
        self.run(&["stash", "push", "-m", "rfx auto-stash"])
    }

    /// Pop the latest stash
    pub fn stash_pop(&self) -> Result<String, String> {
        self.run(&["stash", "pop"])
    }

    /// Undo the last commit but keep changes in the working directory
    pub fn reset_soft(&self, count: usize) -> Result<String, String> {
        let arg = format!("HEAD~{}", count);
        self.run(&["reset", "--soft", &arg])
    }

    pub fn branch(&self) -> Result<String, String> {
        self.run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    pub fn add(&self, files: &[String]) -> Result<String, String> {
        let mut args = vec!["add"];
        args.extend(files.iter().map(|s| s.as_str()));
        self.run(&args)
    }

    pub fn add_all(&self) -> Result<String, String> {
        self.run(&["add", "-A"])
    }

    pub fn commit(&self, message: &str) -> Result<String, String> {
        self.run(&["commit", "-m", message])
    }

    pub fn list_branches(&self) -> Result<Vec<String>, String> {
        let output = self.run(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
        Ok(output.lines().map(|line| line.trim().to_string()).collect())
    }

    pub fn first_commit(&self, branch: &str) -> Result<String, String> {
        let output = self.run(&["log", "--reverse", "--format=%an|%ad", "--date=short", branch])?;
        Ok(output.lines().next().unwrap_or("Unknown|Unknown").to_string())
    }

    pub fn last_commit(&self, branch: &str) -> Result<String, String> {
        let output = self.run(&["log", "-1", "--format=%ad|%s", "--date=short", branch])?;
        Ok(output.lines().next().unwrap_or("Unknown|No commit").to_string())
    }

    pub fn list_remotes(&self) -> Result<Vec<String>, String> {
        let output = self.run(&["remote", "-v"])?;
        Ok(output.lines().map(|line| line.trim().to_string()).collect())
    }

    pub fn list_commits(&self, branch: &str, count: usize) -> Result<Vec<String>, String> {
        let count_arg = format!("-{}", count);
        let output = self.run(&["log", &count_arg, "--pretty=format:%h|%an|%ad|%s", "--date=short", branch])?;
        Ok(output.lines().map(|line| line.trim().to_string()).collect())
    }

    pub fn merge_abort(&self) -> Result<String, String> {
        self.run(&["merge", "--abort"])
    }
}

# Repo Refactor + Test Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the process-global `OnceLock` repo-root cache with an injectable `Repo` struct, then add real test coverage across the CLI core, the desktop Tauri backend, and the desktop React frontend.

**Architecture:** `Repo::open()` (production, discovers root from cwd) / `Repo::open_at(path)` (tests, explicit path) replaces the free-function `adapters::` API with methods on `Repo`. `core` functions take `&Repo` as their first parameter. `ui` (CLI) and the desktop Tauri backend each construct one `Repo` per invocation/command and pass it down. Rust tests use the `tempfile` crate to build disposable git repos per test — fully parallel-safe. Frontend tests use Vitest + React Testing Library with `@tauri-apps/api/core`'s `invoke` mocked.

**Tech Stack:** Rust (existing: clap, prettytable, serde, inquire), `tempfile` (new dev-dependency), Vitest + `@testing-library/react` + `@testing-library/jest-dom` + `jsdom` (new, `rfx-desktop` only).

## Global Constraints

- No behavior change to production code paths — `Repo::open()` must discover the repo root exactly as the old `get_repo_root()` did (via `git rev-parse --show-toplevel`, falling back to cwd on failure).
- All renamed methods drop the `git_` prefix (e.g. `adapters::git_branch()` → `repo.branch()`) since the receiver already reads as "git".
- Every new/changed Rust file must compile with `cargo build` producing zero new warnings (the pre-existing unused `Stdio` import gets fixed as part of this work, not left in place).
- Frontend and Rust test dependencies are dev-only (`[dev-dependencies]` / `devDependencies`) — never added to production dependency lists.
- Desktop backend and frontend are in scope this pass, per user direction — the earlier "leave desktop untouched" concern from the design spec no longer applies.

---

### Task 1: Checkpoint commit of existing desktop WIP

**Files:**
- None created/modified — this is a git-only step.

**Interfaces:**
- Consumes: nothing.
- Produces: a clean working tree for Task 2 onward to build on.

- [ ] **Step 1: Review what's currently uncommitted**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && git status --short`

Expected output (or equivalent — files already modified/untracked from the in-progress dirty-state/sync feature):
```
 M rfx-desktop/src-tauri/src/lib.rs
 M rfx-desktop/src/App.tsx
 M rfx-desktop/src/components/ActionBar.tsx
 M rfx-desktop/src/components/BranchList.tsx
 M rfx-desktop/src/components/FileSelector.tsx
 M rfx-desktop/src/components/RemoteSelector.tsx
 M src/adapters/mod.rs
?? rfx-desktop/src/components/DirtyStateModal.tsx
?? rfx-desktop/src/components/SwitchBranchModal.tsx
```

- [ ] **Step 2: Stage and commit everything as one checkpoint**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src-tauri/src/lib.rs rfx-desktop/src/App.tsx \
  rfx-desktop/src/components/ActionBar.tsx rfx-desktop/src/components/BranchList.tsx \
  rfx-desktop/src/components/FileSelector.tsx rfx-desktop/src/components/RemoteSelector.tsx \
  rfx-desktop/src/components/DirtyStateModal.tsx rfx-desktop/src/components/SwitchBranchModal.tsx \
  src/adapters/mod.rs
git commit -m "Checkpoint: dirty-state/sync handling for desktop sync flow

Commits in-progress work (dirty-state modal, switch-branch modal, smart_sync
dirty-check gating) as its own commit so the upcoming Repo refactor and test
suite land as a separable diff."
```

- [ ] **Step 3: Verify clean tree**

Run: `git status --short`
Expected: no output (clean tree).

---

### Task 2: `Repo` struct in `src/adapters/mod.rs`

**Files:**
- Modify: `src/adapters/mod.rs` (full rewrite)

**Interfaces:**
- Produces: `pub struct Repo`, `Repo::open() -> Self`, `Repo::open_at(path: impl Into<PathBuf>) -> Self`, and these methods on `Repo` (all `pub fn`, `&self` receiver):
  - `status_porcelain(&self) -> Result<Vec<(String, String)>, String>`
  - `ahead_behind(&self, branch: &str) -> Result<(usize, usize), String>`
  - `push_upstream(&self, branch: &str) -> Result<String, String>`
  - `set_remote(&self, branch: &str, remote: &str) -> Result<String, String>`
  - `switch_branch(&self, name: &str) -> Result<String, String>`
  - `create_branch(&self, name: &str) -> Result<String, String>`
  - `fetch(&self) -> Result<String, String>`
  - `list_remote_branches(&self) -> Result<Vec<String>, String>`
  - `pull_branch(&self, branch_name: &str) -> Result<String, String>`
  - `stash_push(&self) -> Result<String, String>`
  - `stash_pop(&self) -> Result<String, String>`
  - `reset_soft(&self, count: usize) -> Result<String, String>`
  - `branch(&self) -> Result<String, String>`
  - `add(&self, files: &[String]) -> Result<String, String>`
  - `add_all(&self) -> Result<String, String>`
  - `commit(&self, message: &str) -> Result<String, String>`
  - `list_branches(&self) -> Result<Vec<String>, String>`
  - `first_commit(&self, branch: &str) -> Result<String, String>`
  - `last_commit(&self, branch: &str) -> Result<String, String>`
  - `list_remotes(&self) -> Result<Vec<String>, String>`
  - `list_commits(&self, branch: &str, count: usize) -> Result<Vec<String>, String>`
  - `merge_abort(&self) -> Result<String, String>`

- [ ] **Step 1: Write the new `src/adapters/mod.rs`**

```rust
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
    pub fn open() -> Self {
        let output = Command::new("git")
            .args(&["rev-parse", "--show-toplevel"])
            .output()
            .expect("Failed to execute git rev-parse");

        let root = if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            PathBuf::from(path_str)
        } else {
            // Fallback: if we aren't in a git repo, just use current dir
            // (this prevents the app from crashing if opened outside a repo)
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        Repo { root }
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
```

- [ ] **Step 2: Build to verify it compiles**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo build 2>&1`

Expected: fails with errors in `src/core/mod.rs` (still calling the old `adapters::git_*` free functions) — that's expected, Task 3 fixes it. Confirm the *only* errors are `unresolved import`/`cannot find function` in `src/core/mod.rs`, not in `src/adapters/mod.rs` itself.

- [ ] **Step 3: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add src/adapters/mod.rs
git commit -m "Replace global repo-root cache with injectable Repo struct

adapters::get_repo_root() cached the repo root in a process-global
OnceLock, set once from whichever call happened first. That makes it
impossible to test git-shelling logic against disposable temp repos.
Repo::open() (production) / Repo::open_at(path) (tests) replaces it.
core/ui/desktop callers are updated in the following commits."
```

---

### Task 3: Thread `&Repo` through `src/core/mod.rs`

**Files:**
- Modify: `src/core/mod.rs` (full rewrite)

**Interfaces:**
- Consumes: `Repo` and its methods from Task 2.
- Produces: every `core::` function below now takes `repo: &Repo` as its first parameter (struct definitions `BranchInfo`, `CommitInfo`, `RemoteInfo`, `RemoteBranchInfo`, `FileChange`, `StatusSummary` are unchanged):
  - `get_status(repo: &Repo) -> Result<StatusSummary, String>`
  - `branches_detailed(repo: &Repo) -> Result<Vec<BranchInfo>, String>`
  - `remotes_detailed(repo: &Repo) -> Result<Vec<RemoteInfo>, String>`
  - `commits_detailed(repo: &Repo, branch: &str, count: usize) -> Result<Vec<CommitInfo>, String>`
  - `get_changed_files(repo: &Repo) -> Result<Vec<FileChange>, String>`
  - `stash_changes(repo: &Repo) -> Result<String, String>`
  - `pop_stash(repo: &Repo) -> Result<String, String>`
  - `stage_files(repo: &Repo, files: &[String]) -> Result<String, String>`
  - `stage_all_files(repo: &Repo) -> Result<String, String>`
  - `create_commit(repo: &Repo, message: &str) -> Result<String, String>`
  - `validate_new_branch_name(repo: &Repo, name: &str) -> Result<(), String>`
  - `switch_remote(repo: &Repo, branch: &str, remote: &str) -> Result<String, String>`
  - `switch_branch(repo: &Repo, name: &str) -> Result<String, String>`
  - `create_branch(repo: &Repo, name: &str) -> Result<String, String>`
  - `get_remote_branches(repo: &Repo) -> Result<Vec<RemoteBranchInfo>, String>`
  - `pull_specific_branch(repo: &Repo, branch_full_name: &str) -> Result<String, String>`
  - `push_branch(repo: &Repo, branch_name: &str) -> Result<String, String>`
  - `undo_last_commit(repo: &Repo) -> Result<String, String>`
  - `parse_remote_url(url: &str) -> (Option<String>, Option<String>, Option<String>)` stays private and unchanged (pure, no repo needed).

- [ ] **Step 1: Write the new `src/core/mod.rs`**

```rust
use crate::adapters::Repo;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub author: String,
    pub time_created: String,
    pub last_change: String,
    pub last_commit: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub direction: String,

    // Derived (best-effort)
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RemoteBranchInfo {
    pub full_name: String, // e.g. origin/main
    pub short_name: String, // e.g. main
    pub author: String,
    pub date: String,
}

/// Represents a file change in a commit
#[derive(Clone, Debug, Serialize)]
pub struct FileChange {
    pub status: String, // e.g., "M", "??", "D"
    pub path: String,
}

/// Summary of the repository status
#[derive(Serialize)]
pub struct StatusSummary {
    pub branch: String,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub changes: Vec<FileChange>,
}

/// Get the current status summary
pub fn get_status(repo: &Repo) -> Result<StatusSummary, String> {
    let branch = repo.branch()?;
    let changes = get_changed_files(repo)?;

    let (ahead, behind) = match repo.ahead_behind(&branch) {
        Ok((a, b)) => (Some(a), Some(b)),
        Err(_) => (None, None),
    };

    Ok(StatusSummary {
        branch,
        ahead,
        behind,
        changes,
    })
}

/// List branches with detailed info
pub fn branches_detailed(repo: &Repo) -> Result<Vec<BranchInfo>, String> {
    let branch_names = repo.list_branches()?;

    let branches: Vec<BranchInfo> = branch_names
        .into_iter()
        .map(|branch| {
            // First commit (creator info)
            let first_commit = repo.first_commit(&branch)
                .unwrap_or_else(|_| "Unknown|Unknown".to_string());
            let mut parts = first_commit.split('|');
            let author = parts.next().unwrap_or("Unknown").to_string();
            let time_created = parts.next().unwrap_or("Unknown").to_string();

            // Last commit info
            let last_commit = repo.last_commit(&branch)
                .unwrap_or_else(|_| "Unknown|No commit".to_string());
            let mut last_parts = last_commit.split('|');
            let last_change = last_parts.next().unwrap_or("Unknown").to_string();
            let last_commit_msg = last_parts.next().unwrap_or("No commit").to_string();

            BranchInfo {
                name: branch,
                author,
                time_created,
                last_change,
                last_commit: last_commit_msg,
            }
        })
        .collect();

    Ok(branches)
}

fn parse_remote_url(url: &str) -> (Option<String>, Option<String>, Option<String>) {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(stripped) = url.strip_prefix("https://") {
        let parts: Vec<&str> = stripped.split('/').collect();
        if parts.len() >= 3 {
            return (
                Some(parts[0].to_string()),
                Some(parts[1].to_string()),
                Some(parts[2].trim_end_matches(".git").to_string()),
            );
        }
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(stripped) = url.strip_prefix("git@") {
        let parts: Vec<&str> = stripped.split(':').collect();
        if parts.len() == 2 {
            let host = parts[0].to_string();
            let path: Vec<&str> = parts[1].split('/').collect();
            if path.len() == 2 {
                return (
                    Some(host),
                    Some(path[0].to_string()),
                    Some(path[1].trim_end_matches(".git").to_string()),
                );
            }
        }
    }

    (None, None, None)
}

/// List remotes with detailed info
pub fn remotes_detailed(repo: &Repo) -> Result<Vec<RemoteInfo>, String> {
    let raw_remotes = repo.list_remotes()?;

    let remotes = raw_remotes
        .into_iter()
        .filter_map(|line| {
            // Example line:
            // origin https://github.com/user/repo.git (fetch)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return None;
            }

            let name = parts[0].to_string();
            let url = parts[1].to_string();
            let direction = parts[2]
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();

            let (host, owner, repo) = parse_remote_url(&url);

            Some(RemoteInfo {
                name,
                url,
                direction,
                host,
                owner,
                repo,
            })
        })
        .collect();

    Ok(remotes)
}

/// List commits with detailed info
pub fn commits_detailed(repo: &Repo, branch: &str, count: usize) -> Result<Vec<CommitInfo>, String> {
    let raw_commits = repo.list_commits(branch, count)?;
    let commits: Vec<CommitInfo> = raw_commits.into_iter().map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        CommitInfo {
            hash: parts.get(0).unwrap_or(&"").to_string(),
            author: parts.get(1).unwrap_or(&"").to_string(),
            date: parts.get(2).unwrap_or(&"").to_string(),
            message: parts.get(3).unwrap_or(&"").to_string(),
        }
    }).collect();
    Ok(commits)
}

// Get list of changed files
pub fn get_changed_files(repo: &Repo) -> Result<Vec<FileChange>, String> {
    let raw_files = repo.status_porcelain()?;

    let changes = raw_files
        .into_iter()
        .map(|(status, path)| FileChange { status, path })
        .collect();

    Ok(changes)
}

// Stash current changes
pub fn stash_changes(repo: &Repo) -> Result<String, String> {
    repo.stash_push()
}

// Pop the latest stash
pub fn pop_stash(repo: &Repo) -> Result<String, String> {
    repo.stash_pop()
}

// Stage specific files
pub fn stage_files(repo: &Repo, files: &[String]) -> Result<String, String> {
    if files.is_empty() {
        return Ok("No files to stage".to_string());
    }
    repo.add(files)
}

// Stage all files
pub fn stage_all_files(repo: &Repo) -> Result<String, String> {
    repo.add_all()
}

// Create commit with message
pub fn create_commit(repo: &Repo, message: &str) -> Result<String, String> {
    let msg = message.trim();

    if msg.is_empty() {
        return Err("Commit message cannot be empty.".to_string());
    }

    // We can keep the length check here as a business rule
    if msg.len() < 3 {
        return Err("Commit message is too short.".to_string());
    }

    repo.commit(msg)
}

/// Check if a branch name is valid and available
pub fn validate_new_branch_name(repo: &Repo, name: &str) -> Result<(), String> {
    let name = name.trim();

    // 1. Basic Syntax Rules
    if name.is_empty() {
        return Err("Branch name cannot be empty.".to_string());
    }
    if name.contains(char::is_whitespace) {
        return Err("Branch names cannot contain spaces.".to_string());
    }

    // 2. Check Existence
    let existing_branches = repo.list_branches()?;
    if existing_branches.iter().any(|b| b == name) {
        return Err(format!("A branch named '{}' already exists.", name));
    }

    Ok(())
}

/// Switch the upstream remote for a branch
pub fn switch_remote(repo: &Repo, branch: &str, remote: &str) -> Result<String, String> {
    repo.set_remote(branch, remote)
}

/// Switch to an existing branch
pub fn switch_branch(repo: &Repo, name: &str) -> Result<String, String> {
    repo.switch_branch(name)
}

/// Create and switch to a new branch
pub fn create_branch(repo: &Repo, name: &str) -> Result<String, String> {
    repo.create_branch(name)
}

/// Get list of remote branches with details
pub fn get_remote_branches(repo: &Repo) -> Result<Vec<RemoteBranchInfo>, String> {
    // 1. Fetch first
    let _ = repo.fetch();

    // 2. Get list
    let raw = repo.list_remote_branches()?;

    let branches = raw.into_iter().filter_map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 { return None; }

        let full_name = parts[0].to_string();

        // --- FILTERS ---
        if full_name.ends_with("/HEAD") || full_name == "HEAD" {
            return None;
        }

        if !full_name.contains('/') {
            return None;
        }

        let short_name = full_name.splitn(2, '/').nth(1).unwrap_or(&full_name).to_string();

        Some(RemoteBranchInfo {
            full_name,
            short_name,
            author: parts[1].to_string(),
            date: parts[2].to_string(),
        })
    }).collect();

    Ok(branches)
}

/// Execute the pull for a specific branch
pub fn pull_specific_branch(repo: &Repo, branch_full_name: &str) -> Result<String, String> {
    repo.pull_branch(branch_full_name)
}

/// Push changes to the remote
pub fn push_branch(repo: &Repo, branch_name: &str) -> Result<String, String> {
    repo.push_upstream(branch_name)
}

/// Undo the last commit
pub fn undo_last_commit(repo: &Repo) -> Result<String, String> {
    // We strictly undo 1 commit
    repo.reset_soft(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_remote_url() {
        let (host, owner, repo) = parse_remote_url("https://github.com/AshwinJ127/resolve.git");
        assert_eq!(host, Some("github.com".to_string()));
        assert_eq!(owner, Some("AshwinJ127".to_string()));
        assert_eq!(repo, Some("resolve".to_string()));
    }

    #[test]
    fn parses_https_remote_url_without_git_suffix() {
        let (host, owner, repo) = parse_remote_url("https://github.com/AshwinJ127/resolve");
        assert_eq!(host, Some("github.com".to_string()));
        assert_eq!(owner, Some("AshwinJ127".to_string()));
        assert_eq!(repo, Some("resolve".to_string()));
    }

    #[test]
    fn parses_ssh_remote_url() {
        let (host, owner, repo) = parse_remote_url("git@github.com:AshwinJ127/resolve.git");
        assert_eq!(host, Some("github.com".to_string()));
        assert_eq!(owner, Some("AshwinJ127".to_string()));
        assert_eq!(repo, Some("resolve".to_string()));
    }

    #[test]
    fn returns_none_for_malformed_url() {
        let (host, owner, repo) = parse_remote_url("not-a-url");
        assert_eq!(host, None);
        assert_eq!(owner, None);
        assert_eq!(repo, None);
    }
}
```

- [ ] **Step 2: Run the new unit tests**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo test --lib parse 2>&1`
Expected: 4 tests pass (`parses_https_remote_url`, `parses_https_remote_url_without_git_suffix`, `parses_ssh_remote_url`, `returns_none_for_malformed_url`).

- [ ] **Step 3: Build to verify the rest compiles**

Run: `cargo build 2>&1`
Expected: fails with errors in `src/main.rs`/`src/ui/mod.rs` (still calling old `core::` signatures without `repo`) and `rfx-desktop/src-tauri/src/lib.rs`. Confirm no errors remain in `src/core/mod.rs` or `src/adapters/mod.rs`.

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add src/core/mod.rs
git commit -m "Thread &Repo through core:: functions

Every core function that touched git now takes repo: &Repo explicitly
instead of reaching into the adapters module's global state. Also adds
unit tests for parse_remote_url now that it's easy to isolate."
```

---

### Task 4: Update `src/main.rs` and `src/ui/mod.rs` to construct and thread `Repo`

**Files:**
- Modify: `src/main.rs` (full rewrite)
- Modify: `src/ui/mod.rs` (full rewrite)

**Interfaces:**
- Consumes: `Repo` (Task 2), `core::` functions taking `&Repo` (Task 3).
- Produces: every `ui::` function takes `repo: &Repo` as its first parameter:
  - `ui::switch_remote(repo: &Repo)`
  - `ui::switch_branch(repo: &Repo)`
  - `ui::show_branches(repo: &Repo, json: bool)`
  - `ui::print_branches_json(branches: &[BranchInfo])` — unchanged, no repo needed.
  - `ui::show_remotes(repo: &Repo, json: bool)`
  - `ui::show_commits(repo: &Repo, branch: &str, count: usize, json: bool)`
  - `ui::new_commit(repo: &Repo)`
  - `ui::new_branch(repo: &Repo)`
  - `ui::show_status(repo: &Repo)`
  - `ui::pull(repo: &Repo)`
  - `ui::push(repo: &Repo)`
  - `ui::undo(repo: &Repo)`

- [ ] **Step 1: Write the new `src/main.rs`**

```rust
use rfx::{adapters, core};
use clap::{Parser, Subcommand, ValueEnum};
mod ui;

/// Resolve CLI - rfx
#[derive(Parser)]
#[command(name = "rfx")]
#[command(about = "A beginner-friendly tool to fix Git workflow issues", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull changes safely
    Pull,
    /// Push changes safely
    Push,

    Show {
        #[arg(value_enum)]
        entity: ShowEntity,

        /// Output as JSON instead of table
        #[arg(long)]
        json: bool,

        /// Branch name for commits (optional)
        #[arg(long, default_value = "main")]
        branch: String,

        /// Number of commits to show
        #[arg(long, default_value_t = 10)]
        count: usize,
    },

    /// Create something new (commit, branch, etc.)
    New {
        #[command(subcommand)]
        entity: NewEntity,
    },   
    
    Status,

    Undo,

    /// Switch to a different branch or remote
    Switch {
        #[command(subcommand)]
        entity: SwitchEntity,
    },
}

#[derive(Subcommand)]
enum SwitchEntity {
    /// Switch to a different branch
    Branch,
    /// Switch to a different remote
    Remote,
}

#[derive(ValueEnum, Clone)]
enum ShowEntity {
    Branches,
    Remotes,
    Commits,
}

#[derive(Subcommand)]
enum NewEntity {
    /// Create a new commit
    Commit,

    /// Create a new branch
    Branch,
}



fn main() {
    let cli = Cli::parse();
    let repo = adapters::Repo::open();

    match cli.command {
        Commands::Pull => ui::pull(&repo),
        Commands::Push => ui::push(&repo),
        Commands::Show { entity, json, branch, count } => match entity {
            ShowEntity::Branches => ui::show_branches(&repo, json),
            ShowEntity::Remotes => ui::show_remotes(&repo, json),
            ShowEntity::Commits => ui::show_commits(&repo, &branch, count, json),
        },
        Commands::New { entity } => match entity {
            NewEntity::Commit => ui::new_commit(&repo),
            NewEntity::Branch => ui::new_branch(&repo), 
        },
        Commands::Status => ui::show_status(&repo),
        Commands::Undo => ui::undo(&repo),
        Commands::Switch { entity } => match entity {
            SwitchEntity::Branch => ui::switch_branch(&repo),
            SwitchEntity::Remote => ui::switch_remote(&repo),
        },
    }
}
```

Note: `use rfx::{adapters, core};` stays exactly as before — it's load-bearing. `mod ui;` makes `ui/mod.rs` part of the binary crate, and `ui/mod.rs`'s `crate::adapters::...`/`crate::core::...` paths resolve through this re-export at the binary crate root. Removing it breaks `ui/mod.rs` even though `main.rs` itself never writes `adapters::`/`core::` directly.

- [ ] **Step 2: Write the new `src/ui/mod.rs`**

```rust
use prettytable::{Table, Row, Cell, format};
use inquire::{Confirm, MultiSelect, Text, validator::Validation, Select};

use crate::adapters::Repo;
use crate::core::{
    BranchInfo, branches_detailed, commits_detailed, create_branch, create_commit,
    get_changed_files, get_remote_branches, get_status, push_branch,
    pull_specific_branch, remotes_detailed, stage_all_files, stage_files,
    undo_last_commit, validate_new_branch_name,
};

pub fn switch_remote(repo: &Repo) {
    // 1. Get current branch
    let current_branch = match repo.branch() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error getting current branch: {}", e);
            return;
        }
    };

    // 2. Get remotes
    let remotes = match remotes_detailed(repo) {
        Ok(r) => r.into_iter().filter(|r| r.direction == "fetch").collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error fetching remotes: {}", e);
            return;
        }
    };

    if remotes.is_empty() {
        println!("No remotes found to switch to.");
        return;
    }

    // 3. Format the menu
    let options: Vec<String> = remotes.iter().map(|r| {
        format!("{} ({})", r.name, r.url)
    }).collect();

    let selection = Select::new("Select a remote to track:", options)
        .with_page_size(10)
        .prompt();

    let selected_remote_name = match selection {
        Ok(s) => {
            let index = remotes.iter().position(|r| {
                let fmt = format!("{} ({})", r.name, r.url);
                fmt == s
            }).unwrap();
            &remotes[index].name
        }
        Err(_) => {
            println!("Cancelled.");
            return;
        }
    };

    // 4. EXECUTE
    println!("\nSetting upstream for '{}' to '{}'...", current_branch, selected_remote_name);

    match crate::core::switch_remote(repo, &current_branch, selected_remote_name) {
        Ok(_) => {
            println!("\nSuccess! Branch '{}' is now tracking '{}'.", current_branch, selected_remote_name);
        }
        Err(e) => {
            eprintln!("\nError setting remote: {}", e);
        }
    }
}

pub fn switch_branch(repo: &Repo) {
    // 0. PRE-FLIGHT CHECK for uncommitted changes
    let changes = match get_changed_files(repo) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to check for changes: {}", e);
            return;
        }
    };

    if !changes.is_empty() {
        println!("\nYou have uncommitted changes. To switch branches, you need to save them first.");
        let options = vec![
            "Stash changes and switch",
            "Commit changes now",
            "Cancel",
        ];
        let choice = Select::new("What would you like to do?", options).prompt();

        match choice {
            Ok("Stash changes and switch") => {
                println!("\nStashing changes...");
                if let Err(e) = crate::core::stash_changes(repo) {
                    eprintln!("Error stashing changes: {}", e);
                    return;
                }
            }
            Ok("Commit changes now") => {
                new_commit(repo);
                // After commit, check if there are still changes. If so, abort.
                if !get_changed_files(repo).unwrap_or_default().is_empty() {
                    println!("\nCommit was cancelled or failed. Aborting switch.");
                    return;
                }
            }
            _ => {
                println!("Switch cancelled.");
                return;
            }
        }
    }

    // 1. Get detailed list of LOCAL branches
    let branches = match branches_detailed(repo) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading branches: {}", e);
            return;
        }
    };

    if branches.is_empty() {
        println!("No branches found to switch to.");
        return;
    }

    // 2. Identify current branch to mark it as default
    let current_branch = match repo.branch() {
        Ok(b) => b,
        Err(_) => String::new(),
    };

    // 3. Format the menu
    let options: Vec<String> = branches.iter().map(|b| {
        let marker = if b.name == current_branch { "*" } else { " " };
        format!("{} {: <20} | Last commit: {}", marker, b.name, b.last_commit)
    }).collect();

    let default_index = branches.iter().position(|b| b.name == current_branch).unwrap_or(0);

    let selection = Select::new("Select branch to switch to:", options)
        .with_starting_cursor(default_index)
        .with_page_size(10)
        .prompt();

    let selected_branch_name = match selection {
        Ok(s) => {
            let index = branches.iter().position(|b| {
                let marker = if b.name == current_branch { "*" } else { " " };
                let fmt = format!("{} {: <20} | Last commit: {}", marker, b.name, b.last_commit);
                fmt == s
            }).unwrap();
            &branches[index].name
        }
        Err(_) => {
            println!("Cancelled.");
            // If we stashed, we should pop it back
            if !changes.is_empty() {
                let _ = crate::core::pop_stash(repo);
            }
            return;
        }
    };
    
    if selected_branch_name == &current_branch {
        println!("You are already on branch '{}'.", current_branch);
        // If we stashed, we should pop it back
        if !changes.is_empty() {
            let _ = crate::core::pop_stash(repo);
        }
        return;
    }

    // 4. EXECUTE ---
    println!("\nSwitching to '{}'...", selected_branch_name);
    
    match crate::core::switch_branch(repo, selected_branch_name) {
        Ok(_) => {
            println!("\nSuccess! Switched to branch '{}'.", selected_branch_name);
            // If we stashed changes, try to pop them
            if !changes.is_empty() {
                println!("Applying stashed changes...");
                if let Err(e) = crate::core::pop_stash(repo) {
                    eprintln!("\nWarning: Could not apply stashed changes.");
                    eprintln!("   Run `git stash pop` manually to resolve conflicts.");
                    eprintln!("   Error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("\nError switching branch: {}", e);
            // If we stashed, we should pop it back
            if !changes.is_empty() {
                let _ = crate::core::pop_stash(repo);
            }
        }
    }
}

/// Display branches in a table or JSON
pub fn show_branches(repo: &Repo, json: bool) {
    let branches = match crate::core::branches_detailed(repo) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Error retrieving branches: {}", err);
            return;
        }
    };

    if json {
        print_branches_json(&branches);
        return;
    }

    let mut table = Table::new();

    // Compact format: header line only
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Branch"),
        Cell::new("Author"),
        Cell::new("Created"),
        Cell::new("Last Change"),
        Cell::new("Last Commit"),
    ]));

    for b in branches {
        let branch = if b.name.len() > 10 { format!("{}…", &b.name[..9]) } else { b.name };
        let author = if b.author.len() > 15 { format!("{}…", &b.author[..14]) } else { b.author };
        let commit_msg = if b.last_commit.len() > 25 { format!("{}…", &b.last_commit[..24]) } else { b.last_commit };

        table.add_row(Row::new(vec![
            Cell::new(&branch),
            Cell::new(&author),
            Cell::new(&b.time_created),
            Cell::new(&b.last_change),
            Cell::new(&commit_msg),
        ]));
    }

    table.printstd();
}

/// Print branches as JSON
pub fn print_branches_json(branches: &[BranchInfo]) {
    match serde_json::to_string_pretty(branches) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize branches: {}", e),
    }
}

/// Display remotes in a table or JSON
pub fn show_remotes(repo: &Repo, json: bool) {
    let remotes = match remotes_detailed(repo) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error fetching remotes: {}", e);
            return;
        }
    };

    if json {
        match serde_json::to_string_pretty(&remotes) {
            Ok(j) => println!("{}", j),
            Err(e) => eprintln!("Failed to serialize remotes: {}", e),
        }
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Name"),
        Cell::new("Direction"),
        Cell::new("Host"),
        Cell::new("Owner"),
        Cell::new("Repo"),
    ]));

    for r in remotes {
        let owner = r.owner.unwrap_or_else(|| "-".into());
        let repo_name = r.repo.unwrap_or_else(|| "-".into());
        let host = r.host.unwrap_or_else(|| "-".into());

        table.add_row(Row::new(vec![
            Cell::new(&r.name),
            Cell::new(&r.direction),
            Cell::new(&host),
            Cell::new(&owner),
            Cell::new(&repo_name),
        ]));
    }

    table.printstd();
}

/// Display commits in a table or JSON
pub fn show_commits(repo: &Repo, branch: &str, count: usize, json: bool) {
    let commits = match commits_detailed(repo, branch, count) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error fetching commits: {}", e);
            return;
        }
    };

    if json {
        match serde_json::to_string_pretty(&commits) {
            Ok(j) => println!("{}", j),
            Err(e) => eprintln!("Failed to serialize commits: {}", e),
        }
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Hash"),
        Cell::new("Author"),
        Cell::new("Date"),
        Cell::new("Message"),
    ]));

    for c in commits {
        let hash = if c.hash.len() > 7 { &c.hash[..7] } else { &c.hash };
        let author = if c.author.len() > 15 {
            format!("{}…", &c.author[..14])
        } else {
            c.author
        };
        let message = if c.message.len() > 30 {
            format!("{}…", &c.message[..29])
        } else {
            c.message
        };

        table.add_row(Row::new(vec![
            Cell::new(hash),
            Cell::new(&author),
            Cell::new(&c.date),
            Cell::new(&message),
        ]));
    }

    table.printstd();
}

/// Create a new commit with user-provided message
pub fn new_commit(repo: &Repo) {
    // 1. Get current status via Core
    let changes = match get_changed_files(repo) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to check status: {}", e);
            return;
        }
    };

    if changes.is_empty() {
        println!("Working directory is clean. Nothing to commit.");
        return;
    }

    // 2. Display changes
    println!("\nChanged files:");
    for file in &changes {
        let label = match file.status.as_str() {
            "??" => "[New]",
            "M" => "[Mod]",
            "D" => "[Del]",
            _ => "[...]",
        };
        println!("  {} {}", label, file.path);
    }
    println!();

    // 3. Ask: Commit everything?
    let commit_all = Confirm::new("Do you want to commit all changes?")
        .with_default(true)
        .prompt();

    match commit_all {
        Ok(true) => {
            if let Err(e) = stage_all_files(repo) {
                eprintln!("Error staging files: {}", e);
                return;
            }
        }
        Ok(false) => {
            // 4. Interactive Selection
            let file_options: Vec<String> = changes
                .iter()
                .map(|f| f.path.clone())
                .collect();

            let selected_files = MultiSelect::new("Select files to include (Space to toggle):", file_options)
                .with_page_size(10)
                .prompt();

            match selected_files {
                Ok(files) if files.is_empty() => {
                    println!("No files selected. Aborting commit.");
                    return;
                }
                Ok(files) => {
                    if let Err(e) = stage_files(repo, &files) {
                        eprintln!("Error staging files: {}", e);
                        return;
                    }
                }
                Err(_) => {
                    println!("Selection cancelled.");
                    return;
                }
            }
        }
        Err(_) => return,
    }

    // 5. Prompt for Message
    let message_prompt = Text::new("Commit message:")
        .with_validator(|input: &str| {
            if input.trim().len() < 3 {
                Ok(Validation::Invalid("Message is too short.".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    match message_prompt {
        Ok(msg) => {
            match create_commit(repo, msg.trim()) {
                Ok(out) => {
                    println!("\nSuccess! Commit created.");
                    // Only show the summary line from git output
                    if let Some(line) = out.lines().next() {
                         println!("{}", line);
                    }
                }
                Err(e) => eprintln!("\nError committing: {}", e),
            }
        }
        Err(_) => println!("Commit cancelled."),
    }
}

pub fn new_branch(repo: &Repo) {
    // 1. Prompt for Name
    let name_prompt = Text::new("Name for new branch:")
        .with_validator(|input: &str| {
            match validate_new_branch_name(repo, input) {
                Ok(_) => Ok(Validation::Valid),
                Err(msg) => Ok(Validation::Invalid(msg.into())),
            }
        })
        .prompt();

    let name = match name_prompt {
        Ok(n) => n.trim().to_string(),
        Err(_) => { println!("Cancelled."); return; }
    };

    // 2. Check for Uncommitted Changes (The "Error" Prevention)
    let changes = match get_changed_files(repo) {
        Ok(c) => c,
        Err(_) => Vec::new(), 
    };

    if !changes.is_empty() {
        println!("\nWarning: You have uncommitted changes.");
        println!("   If you create a new branch now, these changes will move with you.");
        
        let count = changes.len();
        if count <= 5 {
            for file in changes {
                println!("   - {}", file.path);
            }
        } else {
            println!("   - {} files changed...", count);
        }
        println!();

        let confirm = Confirm::new("Do you want to proceed and carry these changes over?")
            .with_default(false)
            .prompt();

        match confirm {
            Ok(true) => (),
            _ => {
                println!("Cancelled. Please commit or stash your changes first.");
                return;
            }
        }
    }

    // 3. Execute
    match create_branch(repo, &name) {
        Ok(_) => {
            println!("\nSuccess! New branch '{}' created.", name);
            println!("   You have been switched to this branch automatically.");
        },
        Err(e) => eprintln!("\nError creating branch: {}", e),
    }
}

pub fn show_status(repo: &Repo) {
    let status = match get_status(repo) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error getting status: {}", e);
            return;
        }
    };

    println!("\nBranch: {}", status.branch);

    // 1. Sync Status Logic
    match (status.ahead, status.behind) {
        (Some(0), Some(0)) => println!("Status: Up to date with remote"),
        (Some(a), Some(b)) => {
            if a > 0 { println!("Status: {} commit(s) ahead (Needs Push)", a); }
            if b > 0 { println!("Status: {} commit(s) behind (Needs Pull)", b); }
        }
        (None, None) => {
            println!("Status: Not published (Local only)");
        }
        _ => {}, 
    }
    println!();

    // 2. File Status
    if status.changes.is_empty() {
        println!("Working directory is clean.");
    } else {
        println!("Unsaved Changes:");
        for file in status.changes {
            let label = match file.status.as_str() {
                "??" => "[New]",
                "M" | "M " => "[Mod]",
                "D" | "D " => "[Del]",
                _ => "[...]",
            };
            println!("  {} {}", label, file.path);
        }
        println!("\nTip: Use 'rfx new commit' to save these.");
    }
    println!();
}

pub fn pull(repo: &Repo) {
    // --- STEP 1: SAFETY CHECK (The "Action Prompt") ---
    loop {
        let changes = match get_changed_files(repo) {
            Ok(c) => c,
            Err(_) => Vec::new(),
        };

        if changes.is_empty() {
            break;
        }

        println!("\nYou have uncommitted changes:");
        for file in changes.iter().take(5) {
            println!("   - {}", file.path);
        }
        if changes.len() > 5 { println!("   ...and {} more.", changes.len() - 5); }
        println!();

        let options = vec!["Commit changes now", "Cancel"];
        let choice = Select::new("What would you like to do?", options).prompt();

        match choice {
            Ok("Commit changes now") => {
                new_commit(repo); 
            }
            _ => {
                println!("Pull cancelled.");
                return;
            }
        }
    }

    // --- STEP 2: BRANCH SELECTION ---
    println!("\nFetching latest updates from remote...");
    
    let branches = match get_remote_branches(repo) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error fetching branches: {}", e);
            return;
        }
    };

    if branches.is_empty() {
        println!("No remote branches found. (Are you connected to the internet?)");
        return;
    }

    let options: Vec<String> = branches.iter().map(|b| {
        format!("{: <15} | {: <15} | {}", b.short_name, b.author, b.date)
    }).collect();

    let selection = Select::new("Select branch to pull from:", options)
        .with_page_size(10)
        .prompt();

    let selected_branch = match selection {
        Ok(s) => {
            let index = branches.iter().position(|b| {
                 let fmt = format!("{: <15} | {: <15} | {}", b.short_name, b.author, b.date);
                 fmt == s
            }).unwrap();
            &branches[index]
        }
        Err(_) => {
            println!("Cancelled.");
            return;
        }
    };

    // --- STEP 3: EXECUTE ---
    println!("\n⬇ Pulling from '{}'...", selected_branch.full_name);

    match pull_specific_branch(repo, &selected_branch.full_name) {
        Ok(out) => {
            if out.contains("Already up to date") {
                 println!("Already up to date.");
            } else {
                 println!("Success! Updates received.");
                 println!("{}", out);
            }
        }
        Err(e) => {
            if e.to_lowercase().contains("conflict") {
                eprintln!("\nMerge Conflict Detected:");
                eprintln!("   We downloaded the code, but couldn't combine it automatically.");
                eprintln!("   Please open the conflicting files and resolve the issues.");
            } else {
                eprintln!("\nError pulling:");
                eprintln!("{}", e);
            }
        }
    }
}

pub fn push(repo: &Repo) {
    // --- STEP 1: SAFETY CHECK ---
    loop {
        let changes = match get_changed_files(repo) {
            Ok(c) => c,
            Err(_) => Vec::new(),
        };

        if changes.is_empty() {
            break;
        }

        println!("\nYou have uncommitted changes:");
        for file in changes.iter().take(5) {
            println!("   - {}", file.path);
        }
        if changes.len() > 5 { println!("   ...and {} more.", changes.len() - 5); }
        println!();

        let options = vec![
            "Commit changes now (Recommended)",
            "Push existing commits (Keep changes local)", 
            "Cancel"
        ];
        
        let choice = Select::new("What would you like to do?", options).prompt();

        match choice {
            Ok("Commit changes now (Recommended)") => {
                new_commit(repo); 
            }
            Ok("Push existing commits (Keep changes local)") => {
                println!("\n[Note] Your uncommitted changes will NOT be sent to the server.");
                break;
            }
            _ => {
                println!("Push cancelled.");
                return;
            }
        }
    }

    // --- STEP 2: BRANCH SELECTION ---
    println!("\nPreparing to push...");

    // 1. Get detailed list of LOCAL branches
    let branches = match branches_detailed(repo) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading branches: {}", e);
            return;
        }
    };

    if branches.is_empty() {
        println!("No branches found (this is very strange).");
        return;
    }

    // 2. Identify current branch to mark it as default
    let current_branch = repo.branch().unwrap_or_default();

    // 3. Format the menu
    let options: Vec<String> = branches.iter().map(|b| {
        let marker = if b.name == current_branch { "*" } else { " " };
        format!("{} {: <15} | {: <15} | {}", marker, b.name, b.author, b.last_change)
    }).collect();

    let default_index = branches.iter().position(|b| b.name == current_branch).unwrap_or(0);

    let selection = Select::new("Select branch to push:", options)
        .with_starting_cursor(default_index)
        .with_page_size(10)
        .prompt();

    let selected_branch_name = match selection {
        Ok(s) => {
            let index = branches.iter().position(|b| {
                let marker = if b.name == current_branch { "*" } else { " " };
                let fmt = format!("{} {: <15} | {: <15} | {}", marker, b.name, b.author, b.last_change);
                fmt == s
            }).unwrap();
            &branches[index].name
        }
        Err(_) => {
            println!("Cancelled.");
            return;
        }
    };

    // --- STEP 3: EXECUTE ---
    println!("\nPushing '{}' to origin...", selected_branch_name);
    
    match push_branch(repo, selected_branch_name) {
        Ok(out) => {
            println!("\nSuccess! Code pushed to origin.");
            if !out.trim().is_empty() {
                println!("{}", out); 
            }
        }
        Err(e) => {
            if e.contains("rejected") || e.contains("fetch first") {
                eprintln!("\n[Push Rejected]");
                eprintln!("The remote repository has changes that you do not have.");
                eprintln!("(This usually means someone else pushed code recently).");
                eprintln!("\nAction: Run 'rfx pull' first to update your branch.");
            } 
            else if e.contains("Could not read from remote") {
                eprintln!("\n[Connection Error]");
                eprintln!("Could not connect to the remote server.");
            }
            else {
                eprintln!("\nError pushing changes:");
                eprintln!("{}", e);
            }
        }
    }
}

pub fn undo(repo: &Repo) {
    let last_commit_msg = match repo.last_commit("HEAD") {
        Ok(s) => {
            s.split('|').nth(1).unwrap_or("Unknown").to_string()
        },
        Err(_) => "Unknown".to_string(),
    };

    println!("\n[Undo Last Commit]");
    println!("This will unsave commit: \"{}\"", last_commit_msg);
    println!("Your files will NOT be deleted. They will move back to 'Unsaved Changes'.");
    println!();

    let confirm = Confirm::new("Are you sure you want to undo this commit?")
        .with_default(false)
        .prompt();

    match confirm {
        Ok(true) => {
            match undo_last_commit(repo) {
                Ok(_) => {
                    println!("\nSuccess! Commit undone.");
                    println!("Your changes are now waiting in the staging area.");
                },
                Err(e) => {
                    eprintln!("\nError undoing commit:");
                    eprintln!("{}", e);
                    if e.contains("ambiguous argument") || e.contains("unknown revision") {
                         eprintln!("(Hint: You cannot undo if there are no commits yet).");
                    }
                }
            }
        },
        _ => println!("Cancelled."),
    }
}
```

Note: the closure passed to `Text::new(...).with_validator(|input: &str| { ... validate_new_branch_name(repo, input) ... })` in `new_branch` captures `repo` by reference — this compiles fine since `repo: &Repo` outlives the closure and the closure only needs an immutable borrow.

- [ ] **Step 3: Build**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo build 2>&1`
Expected: the `rfx` binary builds with zero warnings (the `unused import: Stdio` warning from before Task 2 is gone since the new `adapters/mod.rs` never imported `Stdio`). Errors, if any, should only be in `rfx-desktop/src-tauri` (fixed in Task 9) — confirm with:

Run: `cargo build -p rfx 2>&1`
Expected: `Finished` with no warnings or errors.

- [ ] **Step 4: Manual smoke test**

Run: `./target/debug/rfx status`
Expected: prints branch/status info for this repo (same output shape as before the refactor).

- [ ] **Step 5: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add src/main.rs src/ui/mod.rs
git commit -m "Thread Repo through main.rs and ui/mod.rs

main() constructs one Repo::open() and passes it down through every
ui:: function, which now call repo.<method>() / core::<fn>(repo, ...)
instead of the old global-state adapters:: free functions."
```

---

### Task 5: `tempfile` dev-dependency + shared test repo helper

**Files:**
- Modify: `Cargo.toml`
- Create: `tests/common/mod.rs`

**Interfaces:**
- Produces: `pub fn temp_repo() -> (tempfile::TempDir, rfx::adapters::Repo)` — creates a fresh `git init`-ed repo in a temp dir, configures `user.name`/`user.email`, makes one initial commit on `main` so branch/log-based functions have something to read, and returns both the `TempDir` guard (must be kept alive for the duration of the test — dropping it deletes the directory) and a `Repo` opened at that path.

- [ ] **Step 1: Add `tempfile` as a dev-dependency**

Edit `Cargo.toml`, adding a `[dev-dependencies]` section after `[dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write `tests/common/mod.rs`**

```rust
use rfx::adapters::Repo;
use std::process::Command;
use tempfile::TempDir;

/// Creates a fresh git repo in a temp dir with one initial commit on `main`,
/// and returns (guard, repo). Keep the guard alive for the test's duration —
/// dropping it deletes the directory.
pub fn temp_repo() -> (TempDir, Repo) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    run(path, &["init", "-b", "main"]);
    run(path, &["config", "user.name", "Test User"]);
    run(path, &["config", "user.email", "test@example.com"]);

    std::fs::write(path.join("README.md"), "init\n").expect("failed to write README");
    run(path, &["add", "README.md"]);
    run(path, &["commit", "-m", "Initial commit"]);

    let repo = Repo::open_at(path);
    (dir, repo)
}

/// Creates a bare repo (usable as a fake "origin") in its own temp dir.
pub fn bare_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    run(dir.path(), &["init", "--bare", "-b", "main"]);
    dir
}

fn run(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {:?} failed in {:?}", args, dir);
}
```

- [ ] **Step 3: Verify it compiles (no tests reference it yet, so just check the crate builds)**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo build --tests 2>&1`
Expected: succeeds — `tests/common/mod.rs` isn't picked up as its own test binary (no top-level `.rs` file directly in `tests/`), so no `unused function` warnings yet; Task 6 starts using it.

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add Cargo.toml Cargo.lock tests/common/mod.rs
git commit -m "Add tempfile dev-dependency and shared test-repo helper

temp_repo() gives each test its own disposable git repo via
Repo::open_at, so tests can run fully in parallel with no shared state."
```

---

### Task 6: Core integration tests — status, branches, commits, stash/undo

**Files:**
- Create: `tests/core_tests.rs`

**Interfaces:**
- Consumes: `temp_repo()`/`bare_repo()` from `tests/common/mod.rs` (Task 5), `core::` functions from Task 3.

- [ ] **Step 1: Write `tests/core_tests.rs`**

```rust
mod common;

use common::temp_repo;
use rfx::core;

#[test]
fn get_status_reports_clean_working_directory() {
    let (_guard, repo) = temp_repo();

    let status = core::get_status(&repo).expect("get_status failed");

    assert_eq!(status.branch, "main");
    assert!(status.changes.is_empty());
}

#[test]
fn get_status_reports_dirty_working_directory() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("new_file.txt"), "hello\n").unwrap();

    let status = core::get_status(&repo).expect("get_status failed");

    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].path, "new_file.txt");
    assert_eq!(status.changes[0].status, "??");
}

#[test]
fn branches_detailed_lists_all_local_branches_with_metadata() {
    let (_guard, repo) = temp_repo();
    core::create_branch(&repo, "feature-x").expect("create_branch failed");
    repo.switch_branch("main").expect("switch back to main failed");

    let branches = core::branches_detailed(&repo).expect("branches_detailed failed");
    let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();

    assert!(names.contains(&"main"));
    assert!(names.contains(&"feature-x"));
    for b in &branches {
        assert_eq!(b.author, "Test User");
        assert_eq!(b.last_commit, "Initial commit");
    }
}

#[test]
fn create_commit_stages_and_commits_end_to_end() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("a.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");

    let result = core::create_commit(&repo, "Add a.txt");
    assert!(result.is_ok());

    let commits = core::commits_detailed(&repo, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Add a.txt");
}

#[test]
fn create_commit_rejects_empty_message() {
    let (_guard, repo) = temp_repo();
    let result = core::create_commit(&repo, "   ");
    assert_eq!(result, Err("Commit message cannot be empty.".to_string()));
}

#[test]
fn create_commit_rejects_too_short_message() {
    let (_guard, repo) = temp_repo();
    let result = core::create_commit(&repo, "ab");
    assert_eq!(result, Err("Commit message is too short.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_empty() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "");
    assert_eq!(result, Err("Branch name cannot be empty.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_whitespace() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "has space");
    assert_eq!(result, Err("Branch names cannot contain spaces.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_existing_branch() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "main");
    assert_eq!(result, Err("A branch named 'main' already exists.".to_string()));
}

#[test]
fn validate_new_branch_name_accepts_new_unique_name() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "feature-y");
    assert!(result.is_ok());
}

#[test]
fn create_branch_then_switch_branch_round_trips() {
    let (_guard, repo) = temp_repo();
    core::create_branch(&repo, "feature-z").expect("create_branch failed");
    // create_branch does `checkout -b`, so we should already be on it.
    assert_eq!(repo.branch().unwrap(), "feature-z");

    core::switch_branch(&repo, "main").expect("switch_branch failed");
    assert_eq!(repo.branch().unwrap(), "main");
}

#[test]
fn stash_changes_then_pop_stash_restores_dirty_file() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("dirty.txt"), "wip\n").unwrap();

    core::stash_changes(&repo).expect("stash_changes failed");
    let status_after_stash = core::get_status(&repo).expect("get_status failed");
    assert!(status_after_stash.changes.is_empty());

    core::pop_stash(&repo).expect("pop_stash failed");
    let status_after_pop = core::get_status(&repo).expect("get_status failed");
    assert_eq!(status_after_pop.changes.len(), 1);
    assert_eq!(status_after_pop.changes[0].path, "dirty.txt");
}

#[test]
fn undo_last_commit_removes_commit_but_keeps_changes() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("b.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");
    core::create_commit(&repo, "Add b.txt").expect("create_commit failed");

    core::undo_last_commit(&repo).expect("undo_last_commit failed");

    let commits = core::commits_detailed(&repo, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Initial commit");

    let status = core::get_status(&repo).expect("get_status failed");
    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].path, "b.txt");
}

#[test]
fn remotes_detailed_parses_added_remote() {
    let (guard, repo) = temp_repo();
    let fake_remote_path = guard.path().join("does-not-need-to-exist");
    std::process::Command::new("git")
        .current_dir(guard.path())
        .args(&["remote", "add", "origin", &format!("https://github.com/AshwinJ127/resolve.git")])
        .status()
        .expect("failed to add remote");
    let _ = fake_remote_path; // path unused beyond documenting intent

    let remotes = core::remotes_detailed(&repo).expect("remotes_detailed failed");
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");
    assert_eq!(remotes[0].host, Some("github.com".to_string()));
    assert_eq!(remotes[0].owner, Some("AshwinJ127".to_string()));
    assert_eq!(remotes[0].repo, Some("resolve".to_string()));
}
```

- [ ] **Step 2: Run the new tests**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo test --test core_tests 2>&1`
Expected: all 14 tests pass. If `create_branch_then_switch_branch_round_trips` or similar fails because `create_branch`'s underlying `checkout -b` didn't leave you on the new branch, check the git version output in the failure — but `git checkout -b <name>` always switches, so this should pass as written.

- [ ] **Step 3: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add tests/core_tests.rs
git commit -m "Add integration tests for core:: git-shelling functions

Covers status, branch listing, commit creation, branch name validation,
create/switch branch, stash/pop, undo, and remote URL parsing end-to-end
against disposable temp git repos."
```

---

### Task 7: Core integration tests — push/pull/ahead-behind via fake origin

**Files:**
- Create: `tests/push_pull_tests.rs`

**Interfaces:**
- Consumes: `temp_repo()`/`bare_repo()` from `tests/common/mod.rs` (Task 5), `core::` functions from Task 3.

- [ ] **Step 1: Write `tests/push_pull_tests.rs`**

```rust
mod common;

use common::{bare_repo, temp_repo};
use rfx::core;
use std::process::Command;

fn add_origin(repo_path: &std::path::Path, origin_path: &std::path::Path) {
    let status = Command::new("git")
        .current_dir(repo_path)
        .args(&["remote", "add", "origin", origin_path.to_str().unwrap()])
        .status()
        .expect("failed to add origin");
    assert!(status.success());
}

#[test]
fn push_branch_establishes_upstream_tracking() {
    let (guard, repo) = temp_repo();
    let origin = bare_repo();
    add_origin(guard.path(), origin.path());

    let result = core::push_branch(&repo, "main");
    assert!(result.is_ok(), "push failed: {:?}", result);

    let (ahead, behind) = repo.ahead_behind("main").expect("ahead_behind failed");
    assert_eq!((ahead, behind), (0, 0));
}

#[test]
fn ahead_behind_reports_ahead_after_local_commit_not_yet_pushed() {
    let (guard, repo) = temp_repo();
    let origin = bare_repo();
    add_origin(guard.path(), origin.path());
    core::push_branch(&repo, "main").expect("initial push failed");

    std::fs::write(guard.path().join("c.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");
    core::create_commit(&repo, "Add c.txt").expect("create_commit failed");

    let (ahead, behind) = repo.ahead_behind("main").expect("ahead_behind failed");
    assert_eq!((ahead, behind), (1, 0));
}

#[test]
fn pull_specific_branch_brings_in_remote_commits() {
    let (guard_a, repo_a) = temp_repo();
    let origin = bare_repo();
    add_origin(guard_a.path(), origin.path());
    core::push_branch(&repo_a, "main").expect("push from repo_a failed");

    // A second clone of the same origin, simulating a teammate's machine.
    let guard_b = tempfile::TempDir::new().unwrap();
    let clone_status = Command::new("git")
        .args(&["clone", origin.path().to_str().unwrap(), guard_b.path().to_str().unwrap()])
        .status()
        .expect("clone failed");
    assert!(clone_status.success());
    Command::new("git")
        .current_dir(guard_b.path())
        .args(&["config", "user.name", "Test User"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(guard_b.path())
        .args(&["config", "user.email", "test@example.com"])
        .status()
        .unwrap();
    let repo_b = rfx::adapters::Repo::open_at(guard_b.path());

    // repo_a makes a new commit and pushes it.
    std::fs::write(guard_a.path().join("d.txt"), "content\n").unwrap();
    core::stage_all_files(&repo_a).expect("stage_all_files failed");
    core::create_commit(&repo_a, "Add d.txt").expect("create_commit failed");
    core::push_branch(&repo_a, "main").expect("second push failed");

    // repo_b pulls it.
    let pull_result = core::pull_specific_branch(&repo_b, "origin/main");
    assert!(pull_result.is_ok(), "pull failed: {:?}", pull_result);

    let commits = core::commits_detailed(&repo_b, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Add d.txt");
}
```

- [ ] **Step 2: Run the new tests**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo test --test push_pull_tests 2>&1`
Expected: all 3 tests pass. `pull_specific_branch_brings_in_remote_commits` exercises the full push→push→pull round trip across two independent clones of the same bare "origin".

- [ ] **Step 3: Run the full Rust test suite to confirm nothing regressed**

Run: `cargo test 2>&1`
Expected: all tests across `--lib` (4 `parse_remote_url` tests), `core_tests` (14 tests), and `push_pull_tests` (3 tests) pass — 21 total.

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add tests/push_pull_tests.rs
git commit -m "Add push/pull/ahead-behind integration tests via fake origin

Uses a local bare repo as a stand-in for 'origin' (no network needed) to
cover push_branch, ahead_behind, and pull_specific_branch for real,
including a full push-from-one-clone/pull-into-another round trip."
```

---

### Task 8: Update `rfx-desktop/src-tauri/src/lib.rs` to use `Repo`

**Files:**
- Modify: `rfx-desktop/src-tauri/src/lib.rs` (full rewrite)

**Interfaces:**
- Consumes: `Repo` (Task 2), `core::` functions taking `&Repo` (Task 3).
- Produces: every `#[tauri::command]` function constructs its own `let repo = rfx::adapters::Repo::open();` and passes `&repo` to the `core::`/`Repo` calls it makes. Public command signatures (names, parameters, return types) are unchanged — this only changes internals, so the frontend (`invoke(...)` calls) needs no changes.

- [ ] **Step 1: Write the new `rfx-desktop/src-tauri/src/lib.rs`**

```rust
use rfx::adapters::Repo;
use rfx::core::{self, FileChange};
use serde::Serialize;

#[derive(Serialize)]
struct RepoOverview {
    branches: Vec<UiBranchInfo>,
}

#[derive(Serialize)]
struct UiBranchInfo {
    name: String,
    current: bool,
    ahead: usize,
    behind: usize,
    author: String,
    date: String,
    message: String,
}

#[tauri::command]
fn create_commit(message: String) -> Result<String, String> {
    let repo = Repo::open();
    core::create_commit(&repo, &message).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_branch(name: String) -> Result<String, String> {
    let repo = Repo::open();
    core::create_branch(&repo, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_pending_changes() -> Result<Vec<FileChange>, String> {
    let repo = Repo::open();
    core::get_changed_files(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn commit_selection(message: String, files: Vec<String>) -> Result<String, String> {
    if files.is_empty() {
        return Err("No files selected.".to_string());
    }

    let repo = Repo::open();
    core::stage_files(&repo, &files).map_err(|e| e.to_string())?;
    core::create_commit(&repo, &message).map_err(|e| e.to_string())
}

#[tauri::command]
fn switch_branch(name: String) -> Result<String, String> {
    let repo = Repo::open();
    core::switch_branch(&repo, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_repo_overview() -> Result<RepoOverview, String> {
    let repo = Repo::open();
    let all_branches = core::branches_detailed(&repo).map_err(|e| e.to_string())?;

    let status = core::get_status(&repo).map_err(|e| e.to_string())?;

    let ui_branches = all_branches.into_iter().map(|b| {
        let is_current = b.name == status.branch;

        let (ahead, behind) = if is_current {
            (status.ahead.unwrap_or(0), status.behind.unwrap_or(0))
        } else {
            (0, 0)
        };

        UiBranchInfo {
            name: b.name,
            current: is_current,
            ahead,
            behind,
            author: b.author,
            date: b.last_change,
            message: b.last_commit,
        }
    }).collect();

    Ok(RepoOverview { branches: ui_branches })
}

#[tauri::command]
fn get_git_status() -> String {
    let repo = Repo::open();
    match core::get_status(&repo) {
        Ok(status) => {
            format!(
                "Branch: {}\nSync: +{} / -{}\nPending Changes: {}", 
                status.branch, 
                status.ahead.unwrap_or(0), 
                status.behind.unwrap_or(0), 
                status.changes.len()
            )
        },
        Err(e) => format!("Error: {}", e),
    }
}

#[tauri::command]
fn smart_sync() -> Result<String, String> {
    let repo = Repo::open();
    let status = core::get_status(&repo).map_err(|e| e.to_string())?;
    let branch = status.branch;

    match core::pull_specific_branch(&repo, &format!("origin/{}", branch)) {
        Ok(_) => {},
        Err(e) => return Err(format!("Pull failed: {}", e)),
    }

    match core::push_branch(&repo, &branch) {
        Ok(out) => Ok(format!("Sync complete!\n{}", out)),
        Err(e) => Err(format!("Push failed: {}", e)),
    }
}

#[tauri::command]
fn switch_remote(remote: String) -> Result<String, String> {
    let repo = Repo::open();
    let branch = core::get_status(&repo).map_err(|e| e.to_string())?.branch;
    core::switch_remote(&repo, &branch, &remote).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_remotes() -> Result<Vec<core::RemoteInfo>, String> {
    let repo = Repo::open();
    core::remotes_detailed(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn stash_changes() -> Result<String, String> {
    let repo = Repo::open();
    core::stash_changes(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn pop_stash() -> Result<String, String> {
    let repo = Repo::open();
    core::pop_stash(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_conflict_files() -> Result<Vec<String>, String> {
    let repo = Repo::open();
    let files = repo.status_porcelain().map_err(|e| e.to_string())?;
    let conflict_statuses = ["UU", "AA", "DD", "AU", "UA", "DU", "UD"];
    let conflicts = files
        .into_iter()
        .filter(|(status, _)| conflict_statuses.contains(&status.as_str()))
        .map(|(_, path)| path)
        .collect();
    Ok(conflicts)
}

#[tauri::command]
fn abort_merge() -> Result<String, String> {
    let repo = Repo::open();
    repo.merge_abort().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_git_status,
            smart_sync,
            get_repo_overview,
            create_commit,
            create_branch,
            get_pending_changes,
            commit_selection,
            switch_branch,
            switch_remote,
            get_remotes,
            stash_changes,
            pop_stash,
            get_conflict_files,
            abort_merge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Build the desktop backend**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop/src-tauri && cargo build 2>&1`
Expected: `Finished` with no errors or warnings.

- [ ] **Step 3: Build the whole workspace to confirm everything is green**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo build 2>&1`
Expected: `Finished`, zero warnings, for both the `rfx` crate and (if it's part of the same cargo workspace — check `Cargo.toml` for a `[workspace]` section; if `rfx-desktop/src-tauri` is a separate, non-workspace crate, this command only builds `rfx` and Step 2 already covered the desktop backend).

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src-tauri/src/lib.rs
git commit -m "Update desktop Tauri backend to use Repo

Each #[tauri::command] now constructs its own Repo::open() instead of
relying on adapters::'s removed global state. No change to command
names/signatures, so the frontend needs no changes."
```

---

### Task 9: Desktop backend tests

**Files:**
- Modify: `rfx-desktop/src-tauri/src/lib.rs` (add a `#[cfg(test)] mod tests` block at the end)
- Modify: `rfx-desktop/src-tauri/Cargo.toml` (add `tempfile` dev-dependency)

**Interfaces:**
- Consumes: the plain functions underneath `#[tauri::command]` in `lib.rs` (Task 8) — these are callable directly since the macro doesn't change the function's ABI, it just registers it for IPC dispatch separately.
- Note: `get_conflict_files`, `abort_merge`, and `get_repo_overview` all call `Repo::open()` internally rather than taking a `Repo` parameter, so these tests must run with the process's current working directory set to the temp repo (via `std::env::set_current_dir`) for the duration of each test — see Step 1 for why, and the serialization workaround.

- [ ] **Step 1: Add `tempfile` to `rfx-desktop/src-tauri/Cargo.toml`**

Edit `rfx-desktop/src-tauri/Cargo.toml`, adding after `[dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
serial_test = "3"
```

`serial_test` is needed because `get_conflict_files`/`abort_merge`/`get_repo_overview` call `Repo::open()` (cwd-based discovery) rather than accepting an injected `Repo` — unlike the `core`/`adapters` tests in Tasks 6–7, these three functions' signatures are fixed by the Tauri command contract (no arguments beyond what the frontend passes), so the test has to change the process's cwd, which is process-global and must be serialized across tests in this file to avoid races.

- [ ] **Step 2: Append tests to `rfx-desktop/src-tauri/src/lib.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::process::Command;
    use tempfile::TempDir;

    fn temp_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git").current_dir(dir.path()).args(args).status().unwrap();
            assert!(status.success());
        };
        run(&["init", "-b", "main"]);
        run(&["config", "user.name", "Test User"]);
        run(&["config", "user.email", "test@example.com"]);
        std::fs::write(dir.path().join("README.md"), "init\n").unwrap();
        run(&["add", "README.md"]);
        run(&["commit", "-m", "Initial commit"]);
        dir
    }

    #[test]
    #[serial]
    fn get_conflict_files_returns_files_with_conflict_markers() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };

        // Create a conflicting change on a second branch.
        run(&["checkout", "-b", "feature"]);
        std::fs::write(dir.path().join("README.md"), "feature version\n").unwrap();
        run(&["commit", "-am", "Feature change"]);
        run(&["checkout", "main"]);
        std::fs::write(dir.path().join("README.md"), "main version\n").unwrap();
        run(&["commit", "-am", "Main change"]);
        // This merge will conflict — ignore its exit status.
        let _ = run(&["merge", "feature"]);

        let conflicts = get_conflict_files().expect("get_conflict_files failed");
        assert_eq!(conflicts, vec!["README.md".to_string()]);
    }

    #[test]
    #[serial]
    fn abort_merge_clears_conflict_state() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };

        run(&["checkout", "-b", "feature"]);
        std::fs::write(dir.path().join("README.md"), "feature version\n").unwrap();
        run(&["commit", "-am", "Feature change"]);
        run(&["checkout", "main"]);
        std::fs::write(dir.path().join("README.md"), "main version\n").unwrap();
        run(&["commit", "-am", "Main change"]);
        let _ = run(&["merge", "feature"]);

        abort_merge().expect("abort_merge failed");

        let conflicts = get_conflict_files().expect("get_conflict_files failed");
        assert!(conflicts.is_empty());
    }

    #[test]
    #[serial]
    fn get_repo_overview_attributes_ahead_behind_only_to_current_branch() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };
        run(&["checkout", "-b", "other"]);
        run(&["checkout", "main"]);

        let overview = get_repo_overview().expect("get_repo_overview failed");
        let main_branch = overview.branches.iter().find(|b| b.name == "main").unwrap();
        let other_branch = overview.branches.iter().find(|b| b.name == "other").unwrap();

        assert!(main_branch.current);
        assert!(!other_branch.current);
        assert_eq!(other_branch.ahead, 0);
        assert_eq!(other_branch.behind, 0);
    }
}
```

- [ ] **Step 3: Run the desktop backend tests**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop/src-tauri && cargo test 2>&1`
Expected: 3 tests pass (`get_conflict_files_returns_files_with_conflict_markers`, `abort_merge_clears_conflict_state`, `get_repo_overview_attributes_ahead_behind_only_to_current_branch`).

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src-tauri/Cargo.toml rfx-desktop/src-tauri/Cargo.lock rfx-desktop/src-tauri/src/lib.rs
git commit -m "Add tests for desktop backend's Tauri-specific glue code

Covers get_conflict_files, abort_merge, and get_repo_overview's
current/ahead/behind branch-shaping logic — the parts of the desktop
backend that aren't already covered by the core:: test suite."
```

---

### Task 10: Frontend test tooling setup

**Files:**
- Modify: `rfx-desktop/package.json`
- Create: `rfx-desktop/vitest.config.ts`
- Create: `rfx-desktop/src/test/setup.ts`

**Interfaces:**
- Produces: `npm test` runs Vitest in jsdom mode with `@testing-library/jest-dom` matchers globally available. No component tests exist yet — this task only proves the tooling works with one placeholder test, which Task 11 replaces with the real suite.

- [ ] **Step 1: Install dependencies**

Run:
```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop
npm install --save-dev vitest @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom
```
Expected: packages added to `devDependencies` in `package.json` and `package-lock.json` updated.

- [ ] **Step 2: Add a `test` script to `package.json`**

Edit `rfx-desktop/package.json`'s `"scripts"` block, adding:

```json
    "test": "vitest run"
```

so the full `"scripts"` block reads:

```json
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run"
  },
```

- [ ] **Step 3: Write `rfx-desktop/vitest.config.ts`**

```typescript
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
  },
});
```

- [ ] **Step 4: Write `rfx-desktop/src/test/setup.ts`**

```typescript
import "@testing-library/jest-dom/vitest";
```

- [ ] **Step 5: Write a placeholder test to prove the tooling works**

Create `rfx-desktop/src/test/setup.test.tsx`:

```tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

describe("test tooling", () => {
  it("renders and asserts with jest-dom matchers", () => {
    render(<div>hello</div>);
    expect(screen.getByText("hello")).toBeInTheDocument();
  });
});
```

- [ ] **Step 6: Run it**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop && npm test 2>&1`
Expected: 1 test file, 1 test, passes.

- [ ] **Step 7: Delete the placeholder test**

Run: `rm /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop/src/test/setup.test.tsx`

It served only to prove the tooling works; Task 11 adds the real component tests.

- [ ] **Step 8: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/package.json rfx-desktop/package-lock.json rfx-desktop/vitest.config.ts rfx-desktop/src/test/setup.ts
git commit -m "Add Vitest + React Testing Library tooling to rfx-desktop

No component tests existed before this. jsdom environment,
@testing-library/jest-dom matchers wired up globally via setupFiles."
```

---

### Task 11: Frontend component tests — modals and RemoteSelector

**Files:**
- Create: `rfx-desktop/src/components/NewCommitModal.test.tsx`
- Create: `rfx-desktop/src/components/NewBranchModal.test.tsx`
- Create: `rfx-desktop/src/components/DirtyStateModal.test.tsx`
- Create: `rfx-desktop/src/components/SwitchBranchModal.test.tsx`
- Create: `rfx-desktop/src/components/RemoteSelector.test.tsx`

**Interfaces:**
- Consumes: the components as they exist today (Task 8's checkpoint commit — this task does not modify any component). All tests mock `@tauri-apps/api/core`'s `invoke` via `vi.mock`.

- [ ] **Step 1: Write `rfx-desktop/src/components/DirtyStateModal.test.tsx`**

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DirtyStateModal from "./DirtyStateModal";

const files = [
  { path: "a.txt", status: "M" },
  { path: "b.txt", status: "??" },
];

describe("DirtyStateModal", () => {
  it("renders nothing when closed", () => {
    const { container } = render(
      <DirtyStateModal open={false} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("lists each changed file when open", () => {
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    expect(screen.getByText("a.txt")).toBeInTheDocument();
    expect(screen.getByText("b.txt")).toBeInTheDocument();
  });

  it("calls onCommit when 'Commit changes, then sync' is clicked", () => {
    const onCommit = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={onCommit} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    fireEvent.click(screen.getByText("Commit changes, then sync"));
    expect(onCommit).toHaveBeenCalledOnce();
  });

  it("calls onStash when 'Stash changes and sync' is clicked", () => {
    const onStash = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={onStash} onCancel={vi.fn()} />
    );
    fireEvent.click(screen.getByText("Stash changes and sync"));
    expect(onStash).toHaveBeenCalledOnce();
  });

  it("calls onCancel when Cancel is clicked", () => {
    const onCancel = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={onCancel} />
    );
    fireEvent.click(screen.getByText("Cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 2: Write `rfx-desktop/src/components/NewBranchModal.test.tsx`**

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import NewBranchModal from "./NewBranchModal";

describe("NewBranchModal", () => {
  it("renders nothing when closed", () => {
    const { container } = render(<NewBranchModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("replaces spaces with dashes as the user types", () => {
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);
    const input = screen.getByPlaceholderText("feature/my-new-feature") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "my new branch" } });
    expect(input.value).toBe("my-new-branch");
  });

  it("calls onSubmit with the branch name when the form is submitted", () => {
    const onSubmit = vi.fn();
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);
    const input = screen.getByPlaceholderText("feature/my-new-feature");
    fireEvent.change(input, { target: { value: "feature-x" } });
    fireEvent.click(screen.getByText("Create Branch"));
    expect(onSubmit).toHaveBeenCalledWith("feature-x");
  });

  it("does not call onSubmit when the name is empty", () => {
    const onSubmit = vi.fn();
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);
    expect(screen.getByText("Create Branch")).toBeDisabled();
  });

  it("calls onClose when the X button is clicked", () => {
    const onClose = vi.fn();
    render(<NewBranchModal open={true} onClose={onClose} onSubmit={vi.fn()} />);
    fireEvent.click(screen.getByText("Cancel"));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 3: Write `rfx-desktop/src/components/NewCommitModal.test.tsx`**

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import NewCommitModal from "./NewCommitModal";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("NewCommitModal", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renders nothing when closed", () => {
    const { container } = render(<NewCommitModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("loads pending changes on open and selects them all by default", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { path: "a.txt", status: "M" },
      { path: "b.txt", status: "??" },
    ]);

    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);

    expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes");
    await waitFor(() => expect(screen.getByText("2 file(s) selected")).toBeInTheDocument());
  });

  it("invokes commit_selection with the message and selected files on commit", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce("commit ok"); // commit_selection

    const onSubmit = vi.fn();
    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);

    await waitFor(() => expect(screen.getByText("1 file(s) selected")).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText(/feat: implemented/), {
      target: { value: "Fix the thing" },
    });
    fireEvent.click(screen.getByText("Commit Changes"));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("commit_selection", {
        message: "Fix the thing",
        files: ["a.txt"],
      })
    );
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });

  it("disables the commit button when the message is empty", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]);
    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Commit Changes")).toBeDisabled());
  });
});
```

- [ ] **Step 4: Write `rfx-desktop/src/components/RemoteSelector.test.tsx`**

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import RemoteSelector from "./RemoteSelector";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("RemoteSelector", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("shows a loading state before remotes arrive", () => {
    mockedInvoke.mockReturnValueOnce(new Promise(() => {})); // never resolves
    render(<RemoteSelector />);
    expect(screen.getByText("Loading remotes...")).toBeInTheDocument();
  });

  it("lists only fetch-direction remotes, defaulting to origin", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { name: "origin", url: "https://github.com/a/b.git", direction: "fetch" },
      { name: "origin", url: "https://github.com/a/b.git", direction: "push" },
      { name: "upstream", url: "https://github.com/c/d.git", direction: "fetch" },
    ]);

    render(<RemoteSelector />);

    await waitFor(() => expect(screen.getByText("origin")).toBeInTheDocument());
    expect(screen.getByText("upstream")).toBeInTheDocument();
    expect(screen.getByDisplayValue("origin")).toBeInTheDocument();
  });

  it("invokes switch_remote when the selection changes", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { name: "origin", url: "https://github.com/a/b.git", direction: "fetch" },
      { name: "upstream", url: "https://github.com/c/d.git", direction: "fetch" },
    ]);
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_remote

    render(<RemoteSelector />);
    await waitFor(() => expect(screen.getByText("upstream")).toBeInTheDocument());

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "upstream" } });

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("switch_remote", { remote: "upstream" })
    );
  });
});
```

- [ ] **Step 5: Write `rfx-desktop/src/components/SwitchBranchModal.test.tsx`**

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import SwitchBranchModal from "./SwitchBranchModal";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const branches = [
  { name: "main", current: true, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "init" },
  { name: "feature", current: false, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "wip" },
];

describe("SwitchBranchModal", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renders nothing when closed", () => {
    const { container } = render(
      <SwitchBranchModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("excludes the current branch from the switch-to list", async () => {
    mockedInvoke.mockResolvedValueOnce([]); // get_pending_changes: clean

    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />);

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes"));
    expect(screen.queryByRole("option", { name: "main" })).not.toBeInTheDocument();
    expect(screen.getByRole("option", { name: "feature" })).toBeInTheDocument();
  });

  it("switches directly when the working directory is clean", async () => {
    mockedInvoke.mockResolvedValueOnce([]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_branch

    const onSubmit = vi.fn();
    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} branches={branches} />);

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes"));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    fireEvent.click(screen.getByText("Switch Branch"));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("switch_branch", { name: "feature" })
    );
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });

  it("requires picking stash before switching when there are uncommitted changes", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes: dirty

    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />);

    await waitFor(() => expect(screen.getByText("You have uncommitted changes")).toBeInTheDocument());
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    expect(screen.getByText("Switch Branch")).toBeDisabled();
  });

  it("stashes, switches, and pops when 'stash' is chosen with dirty changes", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // stash_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_branch
    mockedInvoke.mockResolvedValueOnce(undefined); // pop_stash

    const onSubmit = vi.fn();
    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} branches={branches} />);

    await waitFor(() => expect(screen.getByText("You have uncommitted changes")).toBeInTheDocument());
    fireEvent.click(screen.getByText("Stash changes and switch"));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    fireEvent.click(screen.getByText("Stash & Switch"));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("stash_changes"));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("switch_branch", { name: "feature" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("pop_stash"));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });
});
```

- [ ] **Step 6: Run all five test files**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop && npm test 2>&1`
Expected: all tests across `DirtyStateModal.test.tsx`, `NewBranchModal.test.tsx`, `NewCommitModal.test.tsx`, `RemoteSelector.test.tsx`, `SwitchBranchModal.test.tsx` pass.

- [ ] **Step 7: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src/components/DirtyStateModal.test.tsx \
  rfx-desktop/src/components/NewBranchModal.test.tsx \
  rfx-desktop/src/components/NewCommitModal.test.tsx \
  rfx-desktop/src/components/RemoteSelector.test.tsx \
  rfx-desktop/src/components/SwitchBranchModal.test.tsx
git commit -m "Add component tests for modals and RemoteSelector

Covers render/open-closed states, form validation, and the exact
invoke() calls each component fires on submit, with @tauri-apps/api/core
mocked throughout."
```

---

### Task 12: Frontend component tests — BranchList and FileSelector

**Files:**
- Create: `rfx-desktop/src/components/BranchList.test.tsx`
- Create: `rfx-desktop/src/components/FileSelector.test.tsx`

**Interfaces:**
- Consumes: the components as they exist today (Task 8's checkpoint commit — unmodified by this task).

- [ ] **Step 1: Write `rfx-desktop/src/components/BranchList.test.tsx`**

```tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import BranchList from "./BranchList";

const branches = [
  { name: "main", current: true, ahead: 2, behind: 1, author: "Ashwin", date: "2026-01-01", message: "Initial commit" },
  { name: "feature-x", current: false, ahead: 0, behind: 0, author: "Ashwin", date: "2026-01-02", message: "WIP" },
];

describe("BranchList", () => {
  it("shows the branch count", () => {
    render(<BranchList branches={branches} />);
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("marks the current branch with a HEAD badge", () => {
    render(<BranchList branches={branches} />);
    expect(screen.getByText("HEAD")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
    expect(screen.getByText("feature-x")).toBeInTheDocument();
  });

  it("shows ahead/behind counts for a branch that has them", () => {
    render(<BranchList branches={branches} />);
    expect(screen.getByText("2")).toBeInTheDocument(); // ahead count (also matches branch count coincidentally, both 2 here by design)
    expect(screen.getByText("1")).toBeInTheDocument(); // behind count
  });

  it("renders an empty table when there are no branches", () => {
    render(<BranchList branches={[]} />);
    expect(screen.getByText("0")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Write `rfx-desktop/src/components/FileSelector.test.tsx`**

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import FileSelector from "./FileSelector";

const files = [
  { path: "src/a.txt", status: "M" },
  { path: "src/b.txt", status: "??" },
  { path: "src/c.txt", status: "D" },
];

describe("FileSelector", () => {
  it("shows the changed file count", () => {
    render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={vi.fn()} />
    );
    expect(screen.getByText("3 Changed Files")).toBeInTheDocument();
  });

  it("calls onToggle with the file path when a row is clicked", () => {
    const onToggle = vi.fn();
    render(
      <FileSelector files={files} selected={new Set()} onToggle={onToggle} onToggleAll={vi.fn()} />
    );
    fireEvent.click(screen.getByText("a.txt"));
    expect(onToggle).toHaveBeenCalledWith("src/a.txt");
  });

  it("shows 'Select All' when nothing is selected, 'Deselect All' when everything is", () => {
    const { rerender } = render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={vi.fn()} />
    );
    expect(screen.getByText("Select All")).toBeInTheDocument();

    rerender(
      <FileSelector
        files={files}
        selected={new Set(files.map((f) => f.path))}
        onToggle={vi.fn()}
        onToggleAll={vi.fn()}
      />
    );
    expect(screen.getByText("Deselect All")).toBeInTheDocument();
  });

  it("calls onToggleAll when the select-all button is clicked", () => {
    const onToggleAll = vi.fn();
    render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={onToggleAll} />
    );
    fireEvent.click(screen.getByText("Select All"));
    expect(onToggleAll).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 3: Run both test files**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop && npm test 2>&1`
Expected: all tests across `BranchList.test.tsx` and `FileSelector.test.tsx` pass, alongside the five from Task 11 still passing.

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src/components/BranchList.test.tsx rfx-desktop/src/components/FileSelector.test.tsx
git commit -m "Add component tests for BranchList and FileSelector

Covers rendering given props (current-branch marker, ahead/behind
counts, empty state) and the file-selection interaction callbacks."
```

---

### Task 13: Frontend flow tests — `App.tsx` dirty-state sync gating

**Files:**
- Create: `rfx-desktop/src/App.test.tsx`

**Interfaces:**
- Consumes: `App` from `rfx-desktop/src/App.tsx` (Task 8's checkpoint commit — unmodified by this task). Mocks `@tauri-apps/api/core`'s `invoke`.

- [ ] **Step 1: Write `rfx-desktop/src/App.test.tsx`**

```tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const overview = {
  branches: [
    { name: "main", current: true, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "init" },
  ],
};

describe("App sync flow", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("syncs immediately when there are no pending changes", async () => {
    mockedInvoke.mockResolvedValueOnce(overview); // initial fetchStatus -> get_repo_overview
    mockedInvoke.mockResolvedValueOnce([]); // get_remotes for RemoteSelector
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockResolvedValueOnce([]); // get_pending_changes: clean
    mockedInvoke.mockResolvedValueOnce("Sync complete!"); // smart_sync
    mockedInvoke.mockResolvedValueOnce(overview); // get_repo_overview after sync

    fireEvent.click(screen.getByText("Sync"));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("smart_sync"));
    expect(screen.queryByText("Unsaved Changes")).not.toBeInTheDocument();
  });

  it("shows the dirty-state modal instead of syncing immediately when changes are pending", async () => {
    mockedInvoke.mockResolvedValueOnce(overview); // initial fetchStatus
    mockedInvoke.mockResolvedValueOnce([]); // get_remotes
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes: dirty

    fireEvent.click(screen.getByText("Sync"));

    await waitFor(() => expect(screen.getByText("Unsaved Changes")).toBeInTheDocument());
    expect(mockedInvoke).not.toHaveBeenCalledWith("smart_sync");
  });

  it("stashing from the dirty modal stashes, syncs, then pops the stash", async () => {
    mockedInvoke.mockResolvedValueOnce(overview); // initial fetchStatus
    mockedInvoke.mockResolvedValueOnce([]); // get_remotes
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes
    fireEvent.click(screen.getByText("Sync"));
    await waitFor(() => expect(screen.getByText("Unsaved Changes")).toBeInTheDocument());

    mockedInvoke.mockResolvedValueOnce(undefined); // stash_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // smart_sync
    mockedInvoke.mockResolvedValueOnce(overview); // get_repo_overview after sync
    mockedInvoke.mockResolvedValueOnce(undefined); // pop_stash
    mockedInvoke.mockResolvedValueOnce(overview); // get_repo_overview after pop (fetchStatus)

    fireEvent.click(screen.getByText("Stash changes and sync"));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("stash_changes"));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("smart_sync"));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("pop_stash"));
  });
});
```

- [ ] **Step 2: Run it**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop && npm test 2>&1`
Expected: all 3 tests in `App.test.tsx` pass, alongside every test from Tasks 10–12 still passing.

If `stashing from the dirty modal...` is flaky about call ordering because `runSync`'s internal `await invoke("smart_sync")` and the subsequent `get_repo_overview` race with test assertions, prefer the `waitFor` wrapping already shown — each assertion polls until the mock has actually been called, rather than asserting on a fixed timer.

- [ ] **Step 3: Run the full frontend suite one more time to confirm everything together is green**

Run: `npm test 2>&1`
Expected: all frontend test files (Tasks 11, 12, 13) pass together — 8 files total.

- [ ] **Step 4: Commit**

```bash
cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve
git add rfx-desktop/src/App.test.tsx
git commit -m "Add App.tsx flow tests for dirty-state sync gating

Covers the three sync paths: clean-tree immediate sync, dirty-tree
showing the DirtyStateModal instead of syncing, and the stash/sync/pop
sequence when the user picks 'stash' from that modal."
```

---

### Task 14: Final full-suite verification

**Files:**
- None — verification only.

**Interfaces:**
- Consumes: everything from Tasks 1–13.

- [ ] **Step 1: Run the full Rust test suite from the workspace root**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo test 2>&1`
Expected: all `rfx` crate tests pass (lib unit tests + `core_tests` + `push_pull_tests`), zero warnings.

- [ ] **Step 2: Run the desktop backend's test suite**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop/src-tauri && cargo test 2>&1`
Expected: all 3 backend tests pass.

- [ ] **Step 3: Run the frontend test suite**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve/rfx-desktop && npm test 2>&1`
Expected: all 8 test files pass.

- [ ] **Step 4: Confirm the CLI still works end-to-end by hand**

Run: `cd /Users/ashwinjoshi/Ashwin_Brain/Projects/resolve && cargo build --release && ./target/release/rfx status`
Expected: prints real status output for this repo, same shape as before the refactor started.

- [ ] **Step 5: Confirm the working tree is clean**

Run: `git status --short`
Expected: no output — everything from Tasks 1–13 has been committed.

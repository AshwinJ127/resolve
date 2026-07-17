# rfx: Repo refactor + test suite

## Context

`rfx` (CLI + `rfx-desktop` Tauri app) has zero automated test coverage despite
"safety first" being its core value proposition. This is the first of a
multi-pass effort; a later pass will cover feature hardening, docs, and a new
release. This spec covers only the refactor + test suite.

The blocker: `adapters::get_repo_root()` caches the repo root in a
process-global `OnceLock`, set once from whichever call happens first, for
the life of the process. Tests need to point at disposable temp git repos;
a process-global cache makes that impossible to do safely or in parallel.

## Checkpoint commit

Before any refactor work, commit the current uncommitted desktop WIP
(dirty-state/sync handling: `App.tsx`, `ActionBar.tsx`, `BranchList.tsx`,
`FileSelector.tsx`, `RemoteSelector.tsx`, `src-tauri/lib.rs`,
`src/adapters/mod.rs`, plus new `DirtyStateModal.tsx`/`SwitchBranchModal.tsx`)
as-is, so the refactor/test diff is reviewable independently.

## `Repo` struct

Replace the global `OnceLock<PathBuf>` in `src/adapters/mod.rs` with:

```rust
pub struct Repo {
    root: PathBuf,
}

impl Repo {
    /// Discover root from cwd via `git rev-parse --show-toplevel`. Production use.
    pub fn open() -> Self { ... }

    /// Use an explicit path with no discovery/caching. Test use.
    pub fn open_at(path: impl Into<PathBuf>) -> Self { ... }
}
```

All current free functions in `adapters` (`git_status_porcelain`,
`git_branch`, `git_ahead_behind`, etc.) become methods on `Repo`, operating
via `Command::new("git").current_dir(&self.root)`. No behavior change for
real usage — `Repo::open()` still discovers root the same way the old
`OnceLock` did, it's just no longer cached process-wide.

Every `core` function that currently calls `adapters::` free functions takes
`&Repo` as its first argument and calls the corresponding method instead.
This is a mechanical signature change — no logic changes.

Callers updated to construct and thread a `Repo`:
- `src/main.rs` / `src/ui/mod.rs`: one `Repo::open()` in `main()`, threaded
  as `&repo` through all `ui::` functions.
- `rfx-desktop/src-tauri/src/lib.rs`: one `Repo::open()` per `#[tauri::command]`
  invocation (cheap — just a `git rev-parse` call), passed to the `core::`
  functions it calls, including the two spots that currently call
  `adapters::` directly (`get_conflict_files`, `abort_merge`).

## Rust tests: `core` + `adapters`

New tests in `tests/` (currently empty, tracked but unused), using the
`tempfile` crate (new dev-dependency) to create disposable git repos per
test. Each test owns its own `Repo::open_at(tmp_path)` — fully parallel-safe,
no shared state.

**Pure logic (no repo I/O):**
- `parse_remote_url`: HTTPS, SSH, and malformed URL inputs.
- `validate_new_branch_name`: empty name, name containing whitespace.
- `create_commit`: empty message and too-short message rejection (these
  return before touching git).

**Against a real temp repo (init + configure user.name/user.email):**
- `get_status`: clean working directory, dirty working directory (staged +
  unstaged + untracked mixes).
- `branches_detailed`: single branch, multiple branches, author/date/message
  fields populated correctly.
- `create_commit` end-to-end: stage + commit, verify it lands in `git log`.
- `validate_new_branch_name`: existing-branch collision.
- `create_branch` / `switch_branch`: round-trip, verify `git_branch()`
  reflects the switch.
- `stash_changes` / `pop_stash`: dirty file survives a stash/pop round-trip.
- `undo_last_commit`: commit is gone from log, changes reappear as unstaged.
- `remotes_detailed`: add a remote via `git remote add`, verify
  host/owner/repo parsing end-to-end (not just the pure-parsing unit test).

**Push/pull/ahead-behind (needs a fake remote):**
- Create a second local **bare** repo as a fake "origin" (standard
  no-network technique), point the temp repo's `origin` at it.
- Cover `git_push_upstream`, `pull_specific_branch`,
  and `git_ahead_behind` (ahead-only, behind-only, diverged cases) for real.

## Desktop backend tests

`#[tauri::command]` functions are plain `fn`s under the macro, callable
directly from `#[cfg(test)]` without going through IPC. Add tests in
`rfx-desktop/src-tauri/src/lib.rs` (or a `tests/` module) covering:
- `get_conflict_files`: repo with a real merge conflict (create via
  diverging branches + merge) returns the right file list.
- `abort_merge`: conflict state is cleared after calling it.
- `get_repo_overview`: branch shaping — verify `current`/`ahead`/`behind`
  are attributed only to the current branch, zero for others.

Built the same way as the core tests: temp repos via `Repo::open_at`.

## Frontend tests

`rfx-desktop` has no test tooling today. Add Vitest + React Testing Library
+ jsdom + `@testing-library/jest-dom` as devDependencies, with a `test`
script in `package.json` and minimal Vitest config (jsdom environment).

Mock `@tauri-apps/api/core`'s `invoke` (`vi.mock`) in all component tests —
no real Tauri runtime in the test environment.

Coverage:
- `NewCommitModal`, `NewBranchModal`, `DirtyStateModal`, `SwitchBranchModal`,
  `RemoteSelector`: render, and verify the correct `invoke` call (name +
  args) fires on submit; cancel doesn't call `invoke`.
- `BranchList`, `FileSelector`: render correctly given props (current
  branch marker, file status labels, empty states).
- `App.tsx` flow tests (a few, not exhaustive): syncing with pending
  changes shows the dirty-state modal instead of syncing immediately;
  choosing "stash" in the dirty modal stashes then syncs then pops.

## Out of scope for this pass

Feature hardening (panic on missing `git` binary, fragile string-matching
in CLI `Select` menus), README rewrite, version bump, and cutting a new
GitHub release are deferred to a follow-up pass, per user request.

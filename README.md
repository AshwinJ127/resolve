# rfx: Git for Humans

rfx is a command-line tool that makes Git simpler and safer. It wraps standard Git commands in a guided workflow. It stops common mistakes like accidental pushes, confusing merge conflicts, and lost work.

It's built for developers who want a faster, safer way to manage their code without memorizing flags.

A desktop app (`rfx-desktop`) is also available. See [Desktop App](#desktop-app) below.

## Key Features

* **Dashboard (`rfx status`)**
  A plain-English summary of your repo. Shows your branch, how many commits you're ahead or behind, and lists unsaved changes.

* **Interactive Commits (`rfx new commit`)**
  Skip `git add`. This opens a wizard where you pick files with the spacebar and write your message. Staging happens automatically.

* **Safe Branching (`rfx new branch`)**
  Creates a branch and switches to it in one step. Warns you if you have uncommitted changes before doing so.

* **Branch Switching (`rfx switch branch`)**
  Lists your local branches with last-commit info and lets you pick one. If you have uncommitted changes, it offers to stash them first and restore them after the switch.

* **Smart Sync (`rfx pull` / `rfx push`)**
  * **Safety Lock:** blocks a pull if you have unsaved changes, so it never causes a surprise merge conflict.
  * **Interactive:** offers to commit your work before syncing.
  * **Auto-Tracking:** pushing a new branch sets its upstream automatically.

* **Remote Switching (`rfx switch remote`)**
  Changes which remote your current branch tracks.

* **Inspecting History (`rfx show branches` / `rfx show remotes` / `rfx show commits`)**
  Prints a table of branches, remotes, or recent commits. Add `--json` for machine-readable output.

* **Panic Button (`rfx undo`)**
  Reverts your last commit but keeps your files. They move back to staging so you can fix the mistake and try again.

## Installation

### Option 1: For Rust Developers (Recommended)
If you have Rust and Cargo installed, install rfx straight from source.

```bash
cargo install --git https://github.com/AshwinJ127/resolve
```

### Option 2: Prebuilt Binaries
Go to the [Releases page](https://github.com/AshwinJ127/resolve/releases) and download the binary for your platform (macOS, Linux, or Windows).

**Mac/Linux:**
```bash
chmod +x rfx
sudo mv rfx /usr/local/bin/
```

**Windows:**
Move `rfx.exe` to a folder on your PATH.

*Note: on macOS, you may need to allow the app in System Settings > Privacy & Security the first time you run it.*

## Usage Guide

**Check your status**
```bash
rfx status
```

**Save your work**
```bash
rfx new commit
```

**Sync with the team**
```bash
rfx pull
rfx push
```

**Create a feature branch**
```bash
rfx new branch
```

**Switch to another branch**
```bash
rfx switch branch
```

**Undo the last save (keeps your files)**
```bash
rfx undo
```

**Look at your branches, remotes, or commits**
```bash
rfx show branches
rfx show remotes
rfx show commits --branch main --count 20
```

## Desktop App

`rfx-desktop` is a Tauri and React GUI on top of the same core logic as the CLI. It shows your branches, lets you stage and commit files, switch branches, and sync with one click.

### Running it

```bash
cd rfx-desktop
npm install
npm run tauri dev
```

### Building it

```bash
cd rfx-desktop
npm run tauri build
```

## Philosophy

rfx follows a "Safety First" design. It assumes protecting your work matters more than speed. It blocks dangerous actions, like pulling into a dirty working directory, and asks for confirmation before rewriting history.

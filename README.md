# rfx: Git for Humans

rfx is a command-line interface designed to make Git simpler, safer, and more intuitive. It wraps standard Git commands in a user-friendly workflow, preventing common errors like accidental pushes, confusing merge conflicts, and lost work.

It is built for developers who want a faster, safer way to manage their code without memorizing complex flags.

## Key Features

* **Dashboard (rfx status)**
  A clean, plain-English summary of your repository. It tells you exactly which branch you are on, how many commits you are ahead or behind, and lists unsaved changes clearly.

* **Interactive Commits (rfx new commit)**
  Forget `git add`. This command opens an interactive wizard where you can select files with the spacebar and write your message. It handles staging automatically.

* **Smart Sync (rfx pull / rfx push)**
  * **Safety Lock:** rfx prevents you from pulling code if you have unsaved changes, stopping merge conflicts before they happen.
  * **Interactive Mode:** If you have work in progress, it asks if you want to commit it before syncing.
  * **Auto-Tracking:** Pushing a new branch automatically sets the upstream link.

* **Safe Branching (rfx new branch)**
  Creates a new branch and switches to it in one step. It also checks for uncommitted changes to ensure you don't accidentally carry messy work over to the new branch.

* **Panic Button (rfx undo)**
  Made a mistake? `rfx undo` reverts your last commit but **keeps your files**, moving them back to the staging area so you can fix the error and try again.

## Installation

### Option 1: For Rust Developers (Recommended)
If you have Rust and Cargo installed, you can install rfx directly from the source. This ensures you always have the latest version compiled for your machine.

```bash
cargo install --git https://github.com/AshwinJ127/resolve
```

### Option 2: Manual Installation (Mac/Linux)
1. Go to the [Releases Page](https://github.com/AshwinJ127/resolve/releases) and download the latest binary.
2. Open your terminal and navigate to your downloads folder.
3. Run the following commands to make it executable and move it to your path:

```bash
# Make the file executable
chmod +x rfx

# Move to a global bin directory (requires password)
sudo mv rfx /usr/local/bin/
```

*Note: On macOS, you may need to allow the application in **System Settings > Privacy & Security** the first time you run it.*

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

**Undo the last save (Keep files)**
```bash
rfx undo
```

## Philosophy
rfx follows a "Safety First" design philosophy. It assumes that preserving your work is more important than speed. It will block dangerous actions (like pulling into a dirty directory) and prompt you for confirmation before changing history.

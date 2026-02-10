# rfx: A Git Enhancement Tool

## Project Overview

`rfx` is a hybrid system designed to simplify and safeguard Git workflows. It consists of two main parts: a command-line interface (CLI) written in Rust and a Tauri-based desktop application with a React frontend. The project's core philosophy is "Safety First," aiming to prevent common Git errors by providing a more intuitive and interactive user experience.

The CLI wraps standard Git commands in user-friendly workflows, while the desktop application provides a graphical user interface (GUI) for visualizing repository status and managing changes.

## File Structure

The project is organized as a monorepo with a core Rust library, a Rust CLI, and a Tauri desktop application.

```
/
├── Cargo.toml        # Root workspace definition for the Rust projects
├── src/              # Source code for the core Rust library and CLI
│   ├── main.rs       # Entry point for the CLI application
│   └── lib.rs        # Core library code shared between the CLI and desktop app
├── rfx-desktop/      # Contains the Tauri desktop application
│   ├── src/          # Frontend source code (React, TypeScript)
│   │   ├── App.tsx   # Main React application component
│   │   └── main.tsx  # Entry point for the React application
│   ├── src-tauri/    # Backend source code for the Tauri application (Rust)
│   │   ├── Cargo.toml # Dependencies and configuration for the Tauri backend
│   │   ├── tauri.conf.json # Tauri application configuration
│   │   └── src/
│   │       ├── main.rs # Entry point for the Tauri backend
│   │       └── lib.rs  # Tauri-specific library code
│   └── package.json  # Frontend dependencies and scripts
└── README.md         # Project documentation
```

### Key Components:
- **`rfx` (CLI):** A command-line application that provides interactive commands like `rfx status`, `rfx new commit`, and `rfx push`. It uses the `clap` crate for argument parsing and `prettytable` for formatted output.
- **`rfx-desktop` (Desktop App):** A Tauri application that provides a GUI for the `rfx` tool.
    - **Frontend:** Built with React, TypeScript, and Vite. It uses Tailwind CSS for styling and communicates with the Rust backend via the `@tauri-apps/api`.
    - **Backend:** A Rust application that leverages the core `rfx` library and the `tauri` crate to create a webview-based desktop experience.

## Architecture and Design Decisions

### Monorepo Structure
The project is structured as a monorepo to facilitate code sharing between the CLI and the desktop application. The core logic is contained within the `rfx` crate at the root of the project, which is then used as a library by both the CLI (`src/main.rs`) and the Tauri backend (`rfx-desktop/src-tauri`). This approach ensures consistency and reduces code duplication.

### "Safety First" Philosophy
As stated in the `README.md`, the primary design principle is safety. The tool is designed to prevent common mistakes by blocking potentially destructive actions (e.g., pulling into a dirty working directory) and providing interactive prompts for confirmation.

### Technology Choices
- **Rust:** Chosen for its performance, safety, and suitability for both command-line tools and system-level programming, making it an excellent fit for the backend of the Tauri application.
- **Tauri:** Selected for building the desktop application. It allows for a lightweight and secure application by using the system's native webview. This is in contrast to other frameworks that bundle a full browser engine.
- **React and TypeScript:** A popular and robust combination for building modern, scalable user interfaces. TypeScript adds static typing to JavaScript, which helps in catching errors early and improving code quality.
- **Vite:** A fast and modern build tool for web projects. It offers a significantly faster development experience compared to older tools like Webpack.
- **Tailwind CSS:** A utility-first CSS framework that allows for rapid UI development without writing custom CSS.

## What Was Done

- A core Rust library (`rfx`) has been developed to handle the main Git-related logic.
- A CLI application has been built on top of the core library, providing several user-friendly commands.
- A Tauri-based desktop application (`rfx-desktop`) has been set up with a React and TypeScript frontend.
- The desktop application's backend is integrated with the core `rfx` library.
- The frontend includes components for displaying repository status, a file selector, and modals for creating new branches and commits.
- The project has a clear installation guide and usage instructions in the `README.md`.

# Custom Git Implementation

## About

This project is a custom implementation of Git, written in Rust, as part of the [CodeCrafters "Build Your Own Git" challenge](https://app.codecrafters.io/courses/git/). It replicates core Git functionality, including initializing a repository, handling objects, listing trees, writing trees, creating commits, and cloning repositories. The project follows the challenge's step-by-step progression to build a simplified but functional version of Git. The source code is available at [https://github.com/12Danish/custom_git](https://github.com/12Danish/custom_git).

## Project Overview

This Rust-based Git implementation supports essential Git commands to manage a version control system. It uses the `clap` crate for command-line argument parsing and `anyhow` for error handling, as defined in the `Cargo.toml`. The main entry point is `main.rs`, which parses command-line arguments and dispatches to respective command implementations.

### Features
- Initialize a Git repository (`init`)
- Display object contents (`cat-file`)
- Hash and optionally store file contents (`hash-object`)
- List tree object contents (`ls-tree`)
- Create a tree object from the working directory (`write-tree`)
- Create a commit object (`commit-tree`)
- Clone a repository from a URL (`clone`)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Git (for testing and comparison)

## Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/12Danish/custom_git
   cd custom_git
   ```

2. **Build the Project**
   ```bash
   cargo build
   ```

## Usage

The program supports the following commands, implemented as part of the CodeCrafters challenge. Run commands using `cargo run --` followed by the desired subcommand:

```bash
cargo run -- <subcommand> [options]
```

### Available Commands

1. **Initialize a Git Repository**
   Creates a new `.git` directory in the current working directory.
   ```bash
   cargo run -- init
   ```
   - Creates the basic Git directory structure (`.git/objects`, `.git/refs`, etc.).
   - Corresponds to `git init` command.

2. **Display Object Contents (`cat-file`)**
   Reads and displays the contents of a Git object (blob, tree, or commit) given its hash.
   ```bash
   cargo run -- cat-file -p <object-hash>
   ```
   - `-p`: Pretty-print the object contents.
   - Example: `cargo run -- cat-file -p 123abc...`
   - Corresponds to `git cat-file -p <object-hash>`.

3. **Hash a File (`hash-object`)**
   Computes the SHA-1 hash of a file and optionally stores it as a Git blob.
   ```bash
   cargo run -- hash-object [-w] <file-path>
   ```
   - `-w`: Write the blob to the Git object database.
   - Example: `cargo run -- hash-object -w example.txt`
   - Corresponds to `git hash-object [-w] <file>`.

4. **List Tree Contents (`ls-tree`)**
   Displays the contents of a tree object.
   ```bash
   cargo run -- ls-tree [--name-only] <tree-hash>
   ```
   - `--name-only`: Show only the names of objects in the tree.
   - Example: `cargo run -- ls-tree --name-only 456def...`
   - Corresponds to `git ls-tree [--name-only] <tree-hash>`.

5. **Create a Tree Object (`write-tree`)**
   Creates a tree object from the current working directory.
   ```bash
   cargo run -- write-tree
   ```
   - Captures the state of the working directory as a tree object.
   - Example: `cargo run -- write-tree`
   - Corresponds to `git write-tree`.

6. **Create a Commit Object (`commit-tree`)**
   Creates a commit object with a specified tree, optional parent commit, and message.
   ```bash
   cargo run -- commit-tree <tree-hash> [-p <parent-hash>] -m <message>
   ```
   - `-p`: Specify the parent commit hash (optional).
   - `-m`: Commit message.
   - Example: `cargo run -- commit-tree 789ghi... -m "Initial commit"`
   - Corresponds to `git commit-tree <tree-hash> [-p <parent-hash>] -m <message>`.

7. **Clone a Repository (`clone`)**
   Clones a Git repository from a URL to a specified directory (or current directory if not specified).
   ```bash
   cargo run -- clone <url> [<directory>]
   ```
   - Example: `cargo run -- clone https://github.com/user/repo.git my-repo`
   - Corresponds to `git clone <url> [<directory>]`.

## Project Structure

- **`main.rs`**: The main entry point, parsing command-line arguments using `clap` and dispatching to command implementations.
- **`commands/`**: Contains modules for each command (`init`, `cat_file`, `hash_object`, `ls_tree`, `write_tree`, `commit_tree`, `clone`).
- **`objects/`**: Handles Git object parsing and manipulation (blobs, trees, commits).
- **`Cargo.toml`**: Defines dependencies, including `clap` for argument parsing and `anyhow` for error handling.

## CodeCrafters Challenge

This project was built as part of the [CodeCrafters "Build Your Own Git" challenge](https://app.codecrafters.io/courses/git/). The challenge guides you through implementing Git’s core functionality step-by-step, starting with repository initialization and progressing to advanced features like cloning. Each command corresponds to a challenge step, ensuring a deep understanding of Git’s internals.

## Development

To extend or modify the project:
1. Add new commands to the `Command` enum in `main.rs`.
2. Implement the logic in the `commands/` directory.
3. Update `Cargo.toml` if additional dependencies are needed.
4. Run tests with `cargo test`.


![Completion Image](https://github.com/12Danish/custom_git/blob/main/completion-img.png)

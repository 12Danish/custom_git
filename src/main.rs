use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Debug, Subcommand)]

enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },

    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
    WriteTree {},

    CommitTree {
        tree_hash: String,

        #[clap(short = 'p')]
        parent: Option<String>,

        #[clap(short = 'm')]
        message: String,
    },

    Clone {
        url: String,

        dir_path: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            commands::init::init_invoke()?;
        }

        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::cat_file_invoke(&object_hash, pretty_print)?,
        Command::HashObject { write, file } => {
            commands::hash_object::hash_object_invoke(write, &file)?;
        }
        Command::LsTree {
            name_only,
            tree_hash,
        } => {
            commands::ls_tree::ls_tree_invoke(name_only, &tree_hash)?;
        }
        Command::WriteTree {} => {
            commands::write_tree::write_tree_invoke()?;
        }

        Command::CommitTree {
            tree_hash,
            parent,
            message,
        } => {
            commands::commit_tree::commit_tree_invoke(&tree_hash, parent.as_deref(), &message)?;
        }

        Command::Clone { url, dir_path } => {
            commands::clone::clone::clone_invoke(
                &url,
                Path::new(dir_path.as_deref().unwrap_or(".")),
            )?;
        }
    }
    Ok(())
}

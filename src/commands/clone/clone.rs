use crate::commands;
use crate::commands::clone::checkout_empty;
use crate::commands::clone::handle_delta;
use crate::commands::clone::unpack_objects;
use anyhow::Context;
use std::path::Path;

pub(crate) fn clone_invoke(url: &str, dir_path: &Path) -> anyhow::Result<()> {

    // Creating the target dir if it doesnt exist
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
    }
    // Shifting to the target directory
    if dir_path != Path::new(".") {
        std::env::set_current_dir(dir_path)
            .context("Changing the current dir to the specified dir by the user ")?;
    }

    // Calling git init
    commands::init::init_invoke()?;

    // Getting the hash for the latest commit on main/master
    let hash = commands::clone::ls_remote::ls_remote_invoke(url)
        .context("Attempting to get the master/main branch hash from remote git ")?;

    // Downloading pack files from git
    let pack = commands::clone::dowload_pack::download_pack(url, &hash)
        .context("Making request to get the binary pack file data ")?;

    let deltas = unpack_objects::unpack_objects_invoke(&pack)
        .context("Getting list of all delta objects0")?;

    if deltas.len() != 0 {
        handle_delta::process_delta(deltas).context("Processing delta objects")?;
    }

    checkout_empty::checkout_empty_invoke(hash.as_str())
        .context("creating actual directory structure")?;

    Ok(())
}

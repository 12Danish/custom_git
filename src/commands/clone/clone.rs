use std::path::Path;

use anyhow::Context;

use crate::commands;

pub(crate) fn clone_invoke(url: &str, dir_path: &Path) -> anyhow::Result<()> {
    // Shifting to the target directory
    if dir_path != Path::new(".") {
        std::env::set_current_dir(dir_path)
            .context("Changing the current dir to the specified dir by the user ")?;
    }

    // Calling git init
    // commands::init::init_invoke()?;

    let hash = commands::clone::ls_remote::ls_remote_invoke(url)
        .context("Attempting to get the master/main branch hash from remote git ")?;

    println!("{hash}");
    Ok(())
}

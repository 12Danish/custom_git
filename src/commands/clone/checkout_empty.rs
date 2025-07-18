use crate::objects::Object;
use anyhow::Context;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;
use std::{io::BufRead, path::Path};
pub(crate) fn checkout_empty_invoke(commit_hash: &str, path: &Path) -> anyhow::Result<()> {
    let commit_obj = Object::read(commit_hash).context("Parsing commit object")?;

    let mut hash_buf: Vec<u8> = Vec::new();

    let mut file_reader = commit_obj.reader;

    file_reader
        .read_until(b'\n', &mut hash_buf)
        .context("getting the tree entry line from the commit ")?;

    let tree_hash = hash_buf
        .split(|b| b.is_ascii_whitespace())
        .nth(1)
        .context("failed to read tree hash from commit")?;

    let tree_hash = std::str::from_utf8(tree_hash).context("converting the tree hash to &str")?;
    println!("Tree hash: {}", &tree_hash);

    recursively_populate_dir_structure(&tree_hash, path)
        .context("Calling the recursive function to establish the dir structure")?;

    Ok(())
}

fn recursively_populate_dir_structure(tree_hash: &str, parent_path: &Path) -> anyhow::Result<()> {
    let mut tree_obj = Object::read(tree_hash).context("Reading tree hash")?;
    let mut buf: Vec<u8> = Vec::new();
    let mut hashbuf = [0; 20];
    loop {
        buf.clear();
        let n = tree_obj
            .reader
            .read_until(0, &mut buf)
            .context("Reading tree header from file")?;

        if n == 0 {
            break;
        }
        tree_obj
            .reader
            .read_exact(&mut hashbuf[..])
            .context("read tree entry hash")?;

        let mode_and_name = CStr::from_bytes_with_nul(&buf).context("invalid tree entry ")?;

        let mut bits = mode_and_name.to_bytes().splitn(2, |b| *b == b' ');
        let mode = bits.next().expect("split needs to yield once");
        let name = bits
            .next()
            .ok_or_else(|| anyhow::anyhow!("tree entry has no filename"))?;

        let mode =
            std::str::from_utf8(mode).context("Attempting to convert the mode into a string")?;

        let name = std::str::from_utf8(name)
            .context("Attempting to covnvert the object name to string")?;

        let hash = hex::encode(&hashbuf);
        match mode {
            "40000" => {
                let path = parent_path.join(name);
                std::fs::create_dir(&path)
                    .context("Attempting to create a dir if object is a tree")?;
                recursively_populate_dir_structure(&hash, &path).context("Running for sub tree")?;
            }
            _ => handle_file_creation(mode, name, &hash, &parent_path)?,
        }
    }
    Ok(())
}
fn handle_file_creation(
    mode: &str,
    filename: &str,
    blob_hash: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let filepath = path.join(filename);

    match mode {
        "100755" | "100644" => {
            let mut file =
                File::create(&filepath).context("Attempting to create a file from blob object")?;

            let mut blob_obj = Object::read(blob_hash).context("Attempting to read blob hash")?;
            let n = std::io::copy(&mut blob_obj.reader, &mut file)
                .context("write .git/objects to file")?;

            anyhow::ensure!(
                n == blob_obj.expected_size,
                ".git/objects was not the expected size: expected {}, actual {}",
                blob_obj.expected_size,
                n
            );

            let perm = if mode == "100755" { 0o755 } else { 0o644 };
            std::fs::set_permissions(&filepath, std::fs::Permissions::from_mode(perm))
                .context("Setting file permissions")?;
        }

        "120000" => {
            // Symlink case
            let mut blob_obj = Object::read(blob_hash).context("Reading blob for symlink")?;
            let mut link_target = String::new();
            blob_obj
                .reader
                .read_to_string(&mut link_target)
                .context("Reading symlink target from blob")?;

            symlink(link_target.trim_end(), &filepath)
                .with_context(|| format!("Creating symlink {}", filepath.display()))?;
        }

        _ => {
            anyhow::bail!("Not a valid mode: {}", mode);
        }
    }

    Ok(())
}

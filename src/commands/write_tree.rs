use crate::objects::{Kind, Object};
use anyhow::Context;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::{ PermissionsExt};
use std::path::Path;

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir = fs::read_dir(path).with_context(|| format!("Open dir {}", path.display()))?;
    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("Bad dir entry in {}", path.display()))?;
        entries.push(entry);
    }
    entries.sort_unstable_by(|a, b| {
        let afn = a.file_name();
        let bfn = b.file_name();
        let mut afn = afn.into_encoded_bytes();
        let mut bfn = bfn.into_encoded_bytes();
        // Git sorts shorter strings before longer ones
        // This means that the terminating character has higher weight
        afn.push(0xff);
        bfn.push(0xff);
        afn.cmp(&bfn)
    });
    let mut tree_object = Vec::new();
    for entry in entries {
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
        let meta = entry.metadata().context("metadata for dir entry")?;
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if meta.permissions().mode() & 0o111 != 0 {
            "100755"
        } else {
            "100644"
        };

        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                // Empty directory ignore it
                continue;
            };
            hash
        } else {
            let tmp = "temporary";
            let writer = std::fs::File::create(tmp).context("construct temp file or blob")?;

            let hash = Object::blob_from_file(&path)
                .context(" open blob input file ")?
                .write(writer)
                .context("stream file into blob")?;

            let hex_hash = hex::encode(hash);

            fs::create_dir_all(format!(".git/objects/{}/", &hex_hash[..2]))
                .context("create subdir of git objects")?;
            fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hex_hash[..2], &hex_hash[2..]),
            )
            .context("renaming temp file to actual hashed name")?;

            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write tree object")?,
        ))
    }
}

pub(crate) fn write_tree_invoke() -> anyhow::Result<()> {
    if let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? {
        println!("{}", hex::encode(hash));
    } else {
        println!("Empty tree — no hash written.");
    }

    Ok(())
}

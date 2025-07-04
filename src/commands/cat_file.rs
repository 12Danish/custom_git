use anyhow::{Context, Result};

use crate::objects::{Kind, Object};

pub(crate) fn cat_file_invoke(object_hash: &str, pretty_print: bool) -> Result<()> {
    anyhow::ensure!(pretty_print, "-p must  be added");
    let mut obj = Object::read(object_hash).context("parsing blob file")?;
    match obj.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut obj.reader, &mut stdout)
                .context("write .git/objects to stdout")?;

            anyhow::ensure!(
                n == obj.expected_size,
                ".git/objects was not the expected size: expected {}, actual {}",
                obj.expected_size,
                n
            )
        }
        _ => {
            anyhow::bail!("Dont know how to print {}", obj.kind)
        }
    }
    Ok(())
}

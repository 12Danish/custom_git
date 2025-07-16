use crate::objects::{Kind, Object};
use anyhow::Context;
use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};
pub(crate) fn ls_tree_invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut obj = Object::read(tree_hash).context("parsing tree hash")?;
    match obj.kind {
        Kind::Tree => {
            let mut buf = Vec::new();
            let mut hashbuf = [0; 20];
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            loop {
                buf.clear();
                let n = obj
                    .reader
                    .read_until(0, &mut buf)
                    .context("Reading tree header from file")?;

                if n == 0 {
                    break;
                }
                obj.reader
                    .read_exact(&mut hashbuf[..])
                    .context("read tree entry hash")?;

                let mode_and_name =
                    CStr::from_bytes_with_nul(&buf).context("invalid tree entry ")?;

                let mut bits = mode_and_name.to_bytes().splitn(2, |b| *b == b' ');
                let mode = bits.next().expect("split needs to yield once");
                let name = bits
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("tree entry has no filename"))?;

                if name_only {
                    stdout.write_all(name).context("write tree entry name")?;
                } else {
                    let mode = std::str::from_utf8(mode).context("mode is valid utf-8")?;

                    let hash = hex::encode(&hashbuf);
                    let obj = Object::read(&hash)
                        .with_context(|| format!("read object for tree entry {hash}"))?;
                    
                    write!(stdout, "{mode:0>6} {} {hash} ", obj.kind)
                        .context("writing tree entry kind and hash")?;
                    stdout.write_all(name).context("write tree entry name")?;
                }
                writeln!(stdout, "").context("writing a newline to stdout")?;

                buf.clear();
            }
        }
        _ => {
            anyhow::bail!("Dont know how to print {}", obj.kind)
        }
    }
    Ok(())
}

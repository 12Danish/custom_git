use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fmt::{self, write};
use std::fs;
use std::io::BufRead;
use std::io::{BufReader, Read};
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit")
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn read(object_hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let f = fs::File::open(format!(
            ".git/objects/{}/{}",
            &object_hash[..2],
            &object_hash[2..]
        ))
        .context("Read object file from .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("Read header from file in .git/objects")?;

        let header = CStr::from_bytes_with_nul(&buf)
            .expect("There should exactly be one null and that at the end");

        let header = header
            .to_str()
            .context("Header in .git/objects is not valid UTF-8")?;

        let Some((kind, size)) = header.split_once(" ") else {
            anyhow::bail!(".git/objects file did not start with 'blob ' :  {header}")
        };

        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("Not handling that kind yet: {kind }"),
        };

        let size = size
            .parse::<u64>()
            .context(".git/objects file has invalid size")?;

        // NOTE: This will not return an error if file length exceeds size
        let mut z = z.take(size);

        Ok(Object {
            reader: z,
            kind,
            expected_size: size,
        })
    }
}

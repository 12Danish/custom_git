use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fmt::{self};
use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
    RefDelta,
    Unknown(u8),
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
            _ => write!(f, "unsupported"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();
        let stat = std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
        let file =
            std::fs::File::open(&file).with_context(|| format!("open {}", file.display()))?;

        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }
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
            anyhow::bail!(".git/objects file header did not have a ' ' :  {header}")
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
        let z = z.take(size);

        Ok(Object {
            reader: z,
            kind,
            expected_size: size,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        let writer = ZlibEncoder::new(writer, Compression::default());

        let mut writer = HashWriter {
            writer,
            hasher: Sha1::new(),
        };

        write!(writer, "{} {}\0", self.kind, self.expected_size)?;

        std::io::copy(&mut self.reader, &mut writer).context("stream file into blob ")?;
        let _ = writer.writer.finish()?;
        let hash = writer.hasher.finalize();

        Ok(hash.into())
    }

    pub(crate) fn write_to_objects(self) -> anyhow::Result<[u8; 20]> {
        let tmp = "temporary";
        let writer = std::fs::File::create(tmp).context("construct temp file for object")?;

        let hash = self
            .write(writer)
            .context("stream  object into  object file")?;

        let hex_hash = hex::encode(hash);

        fs::create_dir_all(format!(".git/objects/{}/", &hex_hash[..2]))
            .context("create subdir of git objects")?;
        fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &hex_hash[..2], &hex_hash[2..]),
        )
        .context("renaming temp file to actual hashed name")?;

        Ok(hash)
    }
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

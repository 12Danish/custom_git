use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
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
}

enum Kind {
    Blob,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }

        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "-p must  be added");

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
                _ => anyhow::bail!("Not handling that kind yet: {kind }"),
            };

            let size = size
                .parse::<u64>()
                .context(".git/objects file has invalid size")?;

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            // NOTE: This will not return an error if file length exceeds size
            let mut z = z.take(size);
            match kind {
                Kind::Blob => {
                    let n = std::io::copy(&mut z, &mut stdout)
                        .context("write .git/objects to stdout")?;

                    anyhow::ensure!(n == size, ".git.objects file size not as expected")
                }
            }
        }
        Command::HashObject { write, file } => {
            fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stat =
                    std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
                let writer = ZlibEncoder::new(writer, Compression::default());

                let mut writer = HashWriter {
                    writer,
                    hasher: Sha1::new(),
                };

                write!(writer, "blob ")?;

                write!(writer, "{}\0", stat.len())?;
                let mut file = std::fs::File::open(&file)
                    .with_context(|| format!("open {}", file.display()))?;

                std::io::copy(&mut file, &mut writer).context("stream file into blob ")?;
                let _ = writer.writer.finish()?;
                let hash = writer.hasher.finalize();

                Ok(hex::encode(hash))
            }

            let hash = if write {
                let tmp = "temporary";
                let hash = write_blob(
                    &file,
                    std::fs::File::create(tmp).context("construct temp file or blob")?,
                )
                .context("write out blob object")?;
                fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
                    .context("create subdir of git objects")?;
                fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("renaming temp file to actual hashed name")?;
                hash
            } else {
                write_blob(&file, std::io::sink()).context("write out blob object")?
            };
            println!("{hash}");
        }
    }
    Ok(())
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

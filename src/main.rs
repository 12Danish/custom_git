use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Read};
use std::{fs};
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
            ));
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
    }
    Ok(())
}

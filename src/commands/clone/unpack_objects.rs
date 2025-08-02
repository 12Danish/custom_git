use crate::objects::{Kind, Object};
use anyhow::Context;
use sha1::{Digest, Sha1};
use std::io::{Cursor, Read};
use flate2::read::ZlibDecoder;

impl Kind {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => Kind::Commit,
            2 => Kind::Tree,
            3 => Kind::Blob,
            7 => Kind::RefDelta,
            other => Kind::Unknown(other),
        }
    }
}

pub(crate) fn unpack_objects_invoke(mut pack: &[u8]) -> anyhow::Result<Vec<Deltas>> {
    println!("Searching for 'PACK' header...");
    let pack_start = pack
        .windows(4)
        .position(|window| window == b"PACK")
        .context("Could not find PACK header in response")?;
    println!("Found 'PACK' header at offset {}", pack_start);

    pack = &pack[pack_start..];

    if pack.len() < 4 || &pack[0..4] != b"PACK" {
        anyhow::bail!("Not a pack file");
    }

    let pack_data = &pack[..pack.len() - 20];
    let pack_hash = &pack[pack.len() - 20..];
    println!("Calculating SHA-1 hash of pack data...");

    let mut hasher = Sha1::new();
    hasher.update(pack_data);
    let calc_hash = hasher.finalize();

    if calc_hash.as_slice() != pack_hash {
        println!("Expected hash: {:x?}", pack_hash);
        println!("Calculated hash: {:x?}", calc_hash);
        anyhow::bail!("Hash mismatch");
    }

    println!("Hash matches. Proceeding with unpacking...");

    pack = &pack[8..]; // Skip "PACK" and version
    let objs = u32::from_be_bytes(pack[0..4].try_into()?);
    println!("Total objects in pack: {}", objs);
    pack = &pack[4..];

    let mut deltas = Vec::new();

    for i in 0..objs {
        println!("\nParsing object {}/{}", i + 1, objs);
        let mut c = pack[0];
        pack = &pack[1..];

        let obj_type_bits = (c >> 4) & 0x7;
        let obj_type = Kind::from_byte(obj_type_bits);
        println!("Object type: {:?}", obj_type);

        let mut size = (c & 0x0F) as usize;
        let mut shift = 4;

        while c & 0x80 != 0 {
            if pack.is_empty() {
                anyhow::bail!("Unexpected end of pack data while reading size");
            }
            c = pack[0];
            pack = &pack[1..];
            size += ((c & 0x7F) as usize) << shift;
            shift += 7;
        }
        println!("Expected decompressed size: {}", size);
        println!("Raw byte: 0x{:02x}", c);
        println!("Type bits: {}", obj_type_bits);

        if matches!(obj_type, Kind::Tree | Kind::Commit | Kind::Blob) {
            println!("Decompressing normal object...");
            println!("First 10 bytes of compressed data: {:x?}", &pack[..std::cmp::min(10, pack.len())]);
            let (decompressed_data, rem_pack) =
                decompress_data(pack, size).context("Getting the decompressed data")?;
            println!("Decompressed object size: {}", decompressed_data.len());

            Object {
                kind: obj_type,
                expected_size: decompressed_data.len() as u64,
                reader: Cursor::new(decompressed_data),
            }
            .write_to_objects()
            .context("Writing the new object")?;

            pack = rem_pack;
        } else {
            println!("Reading delta object...");
            if pack.len() < 20 {
                anyhow::bail!("Not enough data for delta base hash");
            }
            let delta_name = &pack[0..20];
            println!("Delta base object hash: {:x?}", delta_name);
            if size < 4 {
                anyhow::bail!("Delta object size too small: {}", size);
            }
            println!("First 10 bytes of delta compressed data: {:x?}", &pack[20..std::cmp::min(20 + 10, pack.len())]);
            let (decompressed_data, rem_pack) =
                decompress_data(&pack[20..], size).context("Getting the decompressed data for delta object")?;
            println!("Decompressed delta size: {}", decompressed_data.len());

            deltas.push(Deltas {
                delta_hash: delta_name.to_vec(),
                data: decompressed_data,
            });
            pack = rem_pack;
        }
    }

    println!("\nFinished unpacking all objects.");
    Ok(deltas)
}

fn decompress_data(pack: &[u8], size: usize) -> anyhow::Result<(Vec<u8>, &[u8])> {
    println!("Starting decompression... expected output size: {}", size);
    println!("Available compressed data: {} bytes", pack.len());

    let mut decoder = ZlibDecoder::new(pack);
    let mut output = Vec::with_capacity(size);
    let bytes_read = decoder
        .read_to_end(&mut output)
        .context("Failed to decompress data")?;

    if bytes_read != size {
        anyhow::bail!(
            "Decompressed size mismatch: expected {}, got {}",
            size,
            bytes_read
        );
    }

    let consumed = decoder.total_in() as usize;
    if consumed > pack.len() {
        anyhow::bail!("Decompressor consumed more data than available");
    }

    println!(
        "Decompression finished. Consumed {} bytes, produced {} bytes",
        consumed,
        bytes_read
    );

    Ok((output, &pack[consumed..]))
}

#[derive(Debug)]
pub(crate) struct Deltas {
    pub(crate) delta_hash: Vec<u8>,
    pub(crate) data: Vec<u8>,
}
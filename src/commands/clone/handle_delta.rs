use crate::commands::clone::unpack_objects::Deltas;
use crate::objects::{Kind, Object};
use anyhow::Context;
use std::io::{Cursor, Read};
#[derive(Debug, Clone)]
enum Instruction {
    Copy { size: u32, offset: u32 },
    Insert { size: u32, data: Vec<u8> },
}

pub(crate) fn process_delta(deltas: Vec<Deltas>) -> anyhow::Result<()> {
    for delta in deltas {
        println!(
            "Processing delta object with base {:x?}, size: {}",
            delta.delta_hash,
            delta.data.len()
        );

        let mut data_slice = delta.data.as_slice();

        let source_size = process_var_int(&mut data_slice);
        let target_size = process_var_int(&mut data_slice);

        println!("Source size: {}, Target size: {}", source_size, target_size);

        let instruction_list = parse_instructions(&mut data_slice);

        // Read the base object using the delta.delta_hash (base object hash)
        let (obj_type, base_content) = read_git_object(&delta.delta_hash)?;

        if base_content.len() != source_size {
            anyhow::bail!(
                "Base object size mismatch: expected {}, got {}",
                source_size,
                base_content.len()
            );
        }

        // Apply delta instructions to reconstruct the target object
        let mut target = Vec::new();

        for instruction in instruction_list {
            match instruction {
                Instruction::Copy { size, offset } => {
                    let end_offset = offset as usize + size as usize;
                    if end_offset > base_content.len() {
                        anyhow::bail!(
                            "Copy instruction out of bounds: offset {} + size {} > source size {}",
                            offset,
                            size,
                            base_content.len()
                        );
                    }
                    target.extend_from_slice(&base_content[offset as usize..end_offset]);
                }
                Instruction::Insert { data, .. } => {
                    target.extend_from_slice(&data);
                }
            }
        }

        if target.len() != target_size {
            anyhow::bail!(
                "Target size mismatch: expected {}, got {}",
                target_size,
                target.len()
            );
        }

        // Writing objects to .git file
        Object {
            kind: obj_type,
            expected_size: target.len() as u64,
            reader: Cursor::new(target),
        }
        .write_to_objects()
        .context("writing the new object ")?;

        println!("Successfully processed delta object");
    }
    Ok(())
}

fn process_var_int(data: &mut &[u8]) -> usize {
    let mut shift = 0;
    let mut var = 0;

    while !data.is_empty() && data[0] & 0x80 != 0 {
        let c = data[0];
        *data = &data[1..];
        var |= ((c & 0x7F) as usize) << shift;
        shift += 7;
    }

    if !data.is_empty() {
        let c = data[0];
        *data = &data[1..];
        var |= ((c & 0x7F) as usize) << shift;
    }

    var
}

fn parse_instructions(data: &mut &[u8]) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    while !data.is_empty() {
        let instruc_byte = data[0];
        *data = &data[1..];

        if instruc_byte & 0x80 != 0 {
            // COPY instruction
            let mut offset = 0u32;
            let mut size = 0u32;
            let mut shift = 0;
            let mut offset_bits = instruc_byte & 0xF;

            // Parse offset bytes
            while offset_bits != 0 {
                if offset_bits & 1 != 0 {
                    if !data.is_empty() {
                        offset |= (data[0] as u32) << shift;
                        *data = &data[1..];
                    }
                }
                shift += 8;
                offset_bits >>= 1;
            }

            // Parse size bytes
            shift = 0;
            let mut size_bits = (instruc_byte >> 4) & 0x7;
            while size_bits != 0 {
                if size_bits & 1 != 0 {
                    if !data.is_empty() {
                        size |= (data[0] as u32) << shift;
                        *data = &data[1..];
                    }
                }
                shift += 8;
                size_bits >>= 1;
            }

            // Default size if none specified
            if size == 0 {
                size = 0x1000;
            }

            instructions.push(Instruction::Copy { size, offset });
        } else {
            // INSERT instruction
            let size = (instruc_byte & 0x7F) as usize;

            if data.len() >= size {
                let insert_data = data[0..size].to_vec();
                *data = &data[size..];

                instructions.push(Instruction::Insert {
                    size: size as u32,
                    data: insert_data,
                });
            }
        }
    }

    instructions
}

fn read_git_object(hash: &[u8]) -> anyhow::Result<(Kind, Vec<u8>)> {
    // Convert the 20-byte hash to a hexadecimal string
    let hash_hex = hex::encode(hash);
    let mut obj = Object::read(&hash_hex).context("Reading the delta base object hash")?;
    let mut buf = Vec::new();
    obj.reader
        .read_to_end(&mut buf)
        .context("Reading the base object delta file into buf")?;
    Ok((obj.kind, buf))
}

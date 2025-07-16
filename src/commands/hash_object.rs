use crate::objects::Object;
use anyhow::Context;


use std::path::Path;

pub(crate) fn hash_object_invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    let object = Object::blob_from_file(file).context(" open blob input file ")?;
    let hash = if write {
        object
            .write_to_objects()
            .context("stream file into blob object file.dd ")?
    } else {
        object
            .write(std::io::sink())
            .context("stream file into blob")?
    };

    println!("{}", hex::encode(hash));
    Ok(())
}

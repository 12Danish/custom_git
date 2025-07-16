use crate::objects::{Kind, Object};
use anyhow::Context;
use chrono::{Local, Offset};
use std::io::Cursor;
use std::{
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};
pub(crate) fn commit_tree_invoke(
    tree_hash: &str,
    parent: Option<&str>,
    message: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !message.trim().is_empty(),
        "message must be provided with the commit"
    );
    let mut commit_object: Vec<u8> = Vec::new();

    writeln!(commit_object, "tree {}", tree_hash)
        .context("Writing `tree`  and its hash in commit object ")?;

    if let Some(parent) = parent {
        if !parent.trim().is_empty() {
            writeln!(commit_object, "parent {}", parent)
                .context("Writing `parent` tree and its hash in commit object if available")?;
        }
    }

    let author = "Danish Abbas <danishabbas200$@gmail.com>";
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let timezone = get_timezone();

    writeln!(
        commit_object,
        "author {} {} {}",
        author, timestamp, timezone
    )?;
    writeln!(
        commit_object,
        "committer {} {} {}",
        author, timestamp, timezone
    )?;
    writeln!(commit_object).context("Adding new line before message in hash object")?;
    writeln!(commit_object, "{message}").context("Adding user message in commit object")?;
    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit_object.len() as u64,
        reader: Cursor::new(commit_object),
    }
    .write_to_objects()
    .context("writing commit object")?;
    println!("{}", hex::encode(hash));
    Ok(())
}

fn get_timezone() -> String {
    let now = Local::now();
    let offset = now.offset().fix().local_minus_utc(); // in seconds
    let hours = offset / 3600;
    let minutes = (offset.abs() % 3600) / 60;
    let timezone = format!("{:+03}{}", hours, format!("{:02}", minutes));

    timezone
}

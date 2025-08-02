use anyhow::Context;
use reqwest::blocking::Client;
pub(crate) fn download_pack(url: &str, hash: &str) -> anyhow::Result<Vec<u8>> {
    let request_url = format!("{}.git/git-upload-pack", url);
    let mut content: Vec<u8> = Vec::new();
    let want_line = format!("want {}\n", hash);

    content.extend(b"0032");
    content.extend(want_line.as_bytes());

    content.extend(b"0000"); // flush packet
    content.extend(b"0009done\n"); // done packet

    let client = Client::new();

    let res = client
        .post(&request_url)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(content)
        .send()
        .context("Failed to send POST to git-upload-pack")?;

    let res = res
        .bytes()
        .context("Attempting to read the bytes in the response")?;
    println!("Response size: {}", res.len());
    println!(
        "First few bytes: {:?}",
        &res[..std::cmp::min(20, res.len())]
    );
    Ok(res.to_vec())
}

use anyhow::Context;
use reqwest::blocking::get;
use regex::Regex;
pub(crate) fn ls_remote_invoke(url: &str) -> anyhow::Result<String> {
    let request_url = format!("{}/info/refs?service=git-upload-pack", url);

    let resp = get(&request_url).context("sending get request to git")?;

    let body = resp
        .text()
        .context("attempting to convert the request url response to string")?;

    println!("{}", body);
    let re = Regex::new(r"^[a-z0-9]+ refs/heads/(main|master)$").unwrap();
    for line in body.lines() {
        
        if re.is_match(line) {
            if let Some(part) = line.split_whitespace().next() {
                return Ok(part[4..].to_string());
            };
        }
    }

    anyhow::bail!("No refs/heads/master or refs/heads/main found in response")
}

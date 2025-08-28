use reqwest::Error;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::utility::parse_github_body;

#[derive(Serialize, Deserialize)]
struct GithubRelease {
    name: String,
    body: String,
}

/// Uses GitHub API to get the latest release version number to check if the current version matches with it.
#[cfg(not(tarpaulin_include))]
pub fn check_version() -> Result<Option<Vec<String>>, Error> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Rex")
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(2))
        .build()?;

    let caller: GithubRelease = client
        .get("https://api.github.com/repos/TheRustyPickle/Rex/releases/latest")
        .send()?
        .json()?;

    let github_version = Version::parse(&caller.name.replace('v', "")).unwrap();

    if github_version > current_version {
        let updates = parse_github_body(&caller.body);
        Ok(Some(vec![caller.name, updates]))
    } else {
        Ok(None)
    }
}

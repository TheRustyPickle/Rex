use crate::utility::parse_github_body;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GithubRelease {
    name: String,
    body: String,
}

/// Uses Github API to get the latest release version number to check if the current version matches with it.
pub fn check_version() -> Result<Option<Vec<String>>, reqwest::Error> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    static APP_USER_AGENT: &str = "Rex";

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(std::time::Duration::new(2, 0))
        .build()?;

    let caller: GithubRelease = client
        .get("https://api.github.com/repos/TheRustyPickle/Rex/releases/latest")
        .send()?
        .json()?;

    let github_version = Version::parse(&format!("{}", caller.name.replace("v", ""))).unwrap();

    if github_version > current_version {
        let updates = parse_github_body(caller.body);
        Ok(Some(vec![caller.name, updates]))
    } else {
        Ok(None)
    }
}

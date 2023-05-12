use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    name: String,
}

/// Uses Github API to get the latest release version number to check if the current version matches with it.
pub fn check_version() -> Result<bool, reqwest::Error> {
    let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));
    static APP_USER_AGENT: &str = "Rex";

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(std::time::Duration::new(2, 0))
        .build()?;

    let caller: Version = client
        .get("https://api.github.com/repos/TheRustyPickle/Rex/releases/latest")
        .send()?
        .json()?;
    if current_version != caller.name {
        Ok(true)
    } else {
        Ok(false)
    }
}

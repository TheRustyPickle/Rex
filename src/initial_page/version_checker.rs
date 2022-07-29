use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    name: String,
}

/// Uses Github API to get the latest version number to check if the current version matches with it.
/// If not, we will start the new version pop up
pub fn check_version() -> Result<bool, reqwest::Error> {
    let cu_version = "v0.0".to_string();
    static APP_USER_AGENT: &str = "Testing";
    
    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(std::time::Duration::new(1, 0))
        .build()?;

    let caller: Version = client.get("https://api.github.com/repos/WaffleMixer/Rex/releases/latest").send()?.json()?;
    if cu_version != caller.name {
        return Ok(true)}
    else {
        return Ok(false)
    }
}
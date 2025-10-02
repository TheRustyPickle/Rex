use anyhow::{Result, anyhow};
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LocationInfo {
    location: String,
}

#[derive(Serialize, Deserialize)]
struct BackupPaths {
    locations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    location: PathBuf,
    pub backup_db_path: Option<Vec<PathBuf>>,
    pub new_location: Option<PathBuf>,
}

impl Config {
    pub fn get_config(original_db_path: &PathBuf) -> Result<Self> {
        let mut target_dir = original_db_path.to_owned();
        target_dir.pop();

        target_dir.push("rex.json");

        if !target_dir.exists() {
            return Ok(Config {
                backup_db_path: None,
                new_location: None,
                location: target_dir,
            });
        }

        let mut file = File::open(&target_dir)?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let mut config: Config = serde_json::from_str(&file_content)?;

        config.location = target_dir;
        Ok(config)
    }

    pub fn save_config(&self) -> Result<()> {
        let mut file = File::create(&self.location)?;
        serde_json::to_writer(&mut file, self)?;
        Ok(())
    }

    pub fn reset_new_location(&mut self) -> Result<()> {
        self.new_location = None;
        self.save_config()
    }

    pub fn reset_backup_db_path(&mut self) -> Result<()> {
        self.backup_db_path = None;
        self.save_config()
    }

    pub fn set_backup_db_path(&mut self, mut backup_db_path: Vec<PathBuf>) -> Result<()> {
        let mut original_db_path = self.location.clone();
        original_db_path.pop();

        original_db_path.push("rex.sqlite");

        if let Some(path) = self.new_location.as_ref() {
            backup_db_path.retain(|a| a != path || a != &original_db_path);
        }

        if backup_db_path.is_empty() {
            return Err(anyhow!(
                "After filtering out new location, backup path is empty"
            ));
        }

        self.backup_db_path = Some(backup_db_path);
        self.save_config()
    }

    pub fn set_new_location(&mut self, new_location: PathBuf) -> Result<()> {
        let mut original_db_path = self.location.clone();
        original_db_path.pop();

        original_db_path.push("rex.sqlite");

        if let Some(ref backups) = self.backup_db_path
            && backups
                .iter()
                .any(|p| p == &new_location || p == &original_db_path)
        {
            return Err(anyhow!(
                "New location conflicts with existing backup path or original db location: {}",
                new_location.display()
            ));
        }

        let mut og_db_path = self.location.clone();
        og_db_path.pop();

        og_db_path.push("rex.sqlite");

        let mut new_db_path = new_location.clone();
        new_db_path.push("rex.sqlite");

        fs::copy(og_db_path, new_db_path)?;

        self.new_location = Some(new_location);
        self.save_config()
    }

    pub fn save_backup(&self, db_path: &PathBuf) {
        let mut original_db_path = self.location.clone();
        original_db_path.pop();

        original_db_path.push("rex.sqlite");

        if let Some(paths) = &self.backup_db_path {
            for path in paths {
                let mut target_path = path.clone();
                if !target_path.exists() {
                    println!("Failed to find path {}", target_path.to_string_lossy());
                    continue;
                }
                target_path.push("rex.sqlite");

                if let Err(e) = fs::copy(db_path, &target_path) {
                    println!(
                        "Failed to copy DB to backup path {}. Error: {e:?}",
                        target_path.to_string_lossy()
                    );
                }
            }
        }

        if &original_db_path != db_path
            && let Err(e) = fs::copy(db_path, &original_db_path)
        {
            println!(
                "Failed to copy DB to original path {}. Error: {e:?}",
                original_db_path.to_string_lossy()
            );
        }
    }
}

pub fn migrate_config(config_path: &PathBuf) -> Result<()> {
    let mut config = Config {
        backup_db_path: None,
        new_location: None,
        location: PathBuf::new(),
    };

    let mut backup_path = config_path.to_owned();
    backup_path.pop();

    backup_path.push("backup_paths.json");

    let mut location_path = config_path.to_owned();
    location_path.pop();

    location_path.push("location.json");

    if !backup_path.exists() && !location_path.exists() {
        return Ok(());
    }

    if backup_path.exists() {
        let mut file = File::open(&backup_path)?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let location_info: BackupPaths = serde_json::from_str(&file_content)?;

        config.backup_db_path = Some(
            location_info
                .locations
                .into_iter()
                .map(PathBuf::from)
                .collect(),
        );

        fs::remove_file(backup_path)?;
    }

    if location_path.exists() {
        let mut file = File::open(&location_path)?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let location_info: LocationInfo = serde_json::from_str(&file_content)?;

        config.new_location = Some(PathBuf::from(location_info.location));

        fs::remove_file(location_path)?;

        let mut og_db_path = config_path.to_owned();
        og_db_path.pop();

        og_db_path.push("rex.sqlite");

        let mut new_db_path = config.new_location.clone().unwrap();
        new_db_path.push("rex.sqlite");

        fs::copy(og_db_path, new_db_path)?;
    }

    let mut target_dir = config_path.to_owned();
    target_dir.pop();

    target_dir.push("rex.json");

    let mut file = File::create(target_dir)?;
    serde_json::to_writer(&mut file, &config)?;

    Ok(())
}

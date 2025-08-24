use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub source_folder: PathBuf,
    pub destination_folder: PathBuf,
    pub first_level_folder: Option<String>,
    pub second_level_folder: Option<String>,
    pub subfolders: Option<Vec<String>>,
    pub allowed_extensions: Option<Vec<String>>,
    pub move_files: bool, 
    pub last_run: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub recipes: Vec<Recipe>, 
    #[serde(skip)]
    pub path: PathBuf
}

impl Settings {
    pub fn load_from_file(file_path: &PathBuf) -> anyhow::Result<Settings> {
        let settings_result = fs::read_to_string(file_path);
        if let Ok(settings_string) = settings_result {
            let recipes: Vec<Recipe> = serde_json::from_str(&settings_string.as_str())?;
            let to_return = Settings {
                recipes,
                path: file_path.clone()
            };
            return Ok(to_return);
        } else {
            return Err(anyhow::Error::msg("Error while loading the settings file"));
        }
    }


    // pub fn load() -> anyhow::Result<Settings> {
    //     if let Some(proj_dirs) = ProjectDirs::from("net", "prrpcht", "file_organizer") {
    //         log::debug!("Config dir: {}", proj_dirs.config_dir().to_str().unwrap());
    //         if Path::new(proj_dirs.config_dir()).is_dir() {
    //             let settings_path = proj_dirs.config_dir().join("settings.json");
    //             if Path::new(settings_path.as_path()).is_file() {
    //                 let settings_result = fs::read_to_string(settings_path.clone());
    //                 if let Ok(settings_string) = settings_result {
    //                     let to_return: Settings = serde_json::from_str(&settings_string.as_str())?;
    //                     return Ok(to_return);
    //                 } else {
    //                     return Self::create_new_settings_file(settings_path);
    //                 }
    //             } else {
    //                 return Self::create_new_settings_file(settings_path);
    //             }
    //         } else {
    //             log::warn!("Not settings file existing");
    //             fs::create_dir_all(proj_dirs.config_dir()).expect("Can't create new config folder");
    //             let settings_path = proj_dirs.config_dir().join("settings.json");
    //             return Self::create_new_settings_file(settings_path);
    //         }
    //     } else {
    //         Err(anyhow::Error::msg("No app data folder available"))
    //     }
    // }

    // fn create_new_settings_file(settings_path: std::path::PathBuf) -> anyhow::Result<Settings> {
    //     let mut file = fs::File::create(settings_path)?;
    //     let to_return: Settings = Default::default();
    //     let to_write = serde_json::to_string(&to_return)?;
    //     let _ = write!(file, "{}", to_write)?;
    //     return Ok(to_return);
    // }

    pub fn save(&self) -> anyhow::Result<()> {
        let to_write = serde_json::to_string(&self.recipes)?;
        let mut file = fs::File::create(self.path.clone())?;
        let _ = write!(file, "{}", to_write)?;
        Ok(())
    }
}
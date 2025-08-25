use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub enum DateComparator {
    CreationDate,
    ModificationDate,
}

impl Default for DateComparator {
    fn default() -> Self {
        DateComparator::ModificationDate
    }
}

/// Recipe is a struct that contains the settings for a recipe.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub source_folder: PathBuf,
    pub destination_folder: PathBuf,
    pub date_comparator: Option<DateComparator>,
    pub subfolders: Option<Vec<String>>,
    pub allowed_extensions: Option<Vec<String>>,
    pub move_files: bool,
    pub last_run: Option<String>,
}

/// Settings is a struct that contains the settings for the file organizer.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub recipes: Vec<Recipe>,
    #[serde(skip)]
    pub path: PathBuf,
}

impl Settings {
    /// Loads the settings from a file.
    ///
    /// ### Parameters
    /// - `file_path`: The path to the settings file.
    ///
    /// ### Returns
    /// - `Result<Settings, anyhow::Error>`: The settings.
    pub fn load_from_file(file_path: &PathBuf) -> anyhow::Result<Settings> {
        let settings_result = fs::read_to_string(file_path);
        if let Ok(settings_string) = settings_result {
            let recipes: Vec<Recipe> = serde_json::from_str(&settings_string.as_str())?;
            let to_return = Settings {
                recipes,
                path: file_path.clone(),
            };
            return Ok(to_return);
        } else {
            return Err(anyhow::Error::msg("Error while loading the settings file"));
        }
    }

    /// Saves the settings to a file.
    ///
    /// ### Parameters
    /// - `self`: The settings to save.
    ///
    /// ### Returns
    /// - `Result<(), anyhow::Error>`: The result of the save.
    pub fn save(&self) -> anyhow::Result<()> {
        let to_write = serde_json::to_string(&self.recipes)?;
        let mut file = fs::File::create(self.path.clone())?;
        let _ = write!(file, "{}", to_write)?;
        Ok(())
    }
}

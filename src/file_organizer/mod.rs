
use chrono::{DateTime, Utc};
use crate::file_organizer::settings::Settings;
use crate::file_organizer::settings::Recipe;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use colored::*;

pub mod settings;

pub struct FileOrganizer {
    settings: Settings,
}

impl FileOrganizer {
    pub fn new(settings_file_path:PathBuf) -> Self {
        let mut settings = Settings::load_from_file(&settings_file_path).expect("Error loading recipes file, cannot continue");
        for recipe in &mut settings.recipes {
            if let Some(allowed_extensions) = &mut recipe.allowed_extensions {
                for extension in allowed_extensions {
                    *extension = extension.to_lowercase();
                }
            }
        }
        Self { settings }
    }

    pub fn run(&mut self, dry_run: bool) -> anyhow::Result<()> {
        for recipe in &self.settings.recipes {
            self.run_recipe(&recipe, dry_run)?;
        }
        
        // Update last_run for all recipes if not in dry run mode
        if !dry_run {
            let last_run = Utc::now();
            let last_run = Some(last_run.format("%Y-%m-%d").to_string());
            for recipe in &mut self.settings.recipes {
                recipe.last_run = last_run.clone();
            }
        
            self.settings.save()?;
        }
        Ok(())
    }

    fn run_recipe(&self, recipe: &Recipe, dry_run: bool) -> anyhow::Result<()> {
        if !recipe.source_folder.is_dir()  {
            return Err(anyhow::Error::msg(format!("{} - Source folder not a directory: {}", recipe.name, recipe.source_folder.display())));
        }
        if !recipe.destination_folder.is_dir() {
            return Err(anyhow::Error::msg(format!("{} - Target folder not a directory: {}", recipe.name, recipe.destination_folder.display())));
        }
        let date_boundary = recipe.last_run.clone().unwrap_or_else(|| "1970-01-01".to_string());
        //let date_boundary = DateTime::parse_from_str(&date_boundary, "%Y-%m-%d")?;
        let date_boundary = DateTime::parse_from_str(&format!("{} 00:00:00 +0000", date_boundary), "%Y-%m-%d %H:%M:%S %z")?;

        let entries = fs::read_dir(&recipe.source_folder)?;
        for entry in entries {
            let entry = entry?;
            let from_file = entry.path();
            if from_file.is_file() {
                if let Some(filename) = from_file.file_name() {
                    if filename.to_str().unwrap().starts_with(".") {
                        continue;
                    }
                    if !is_extension_allowed(&from_file, &recipe.allowed_extensions) {
                        continue;
                    }
                    let last_modification_date = entry.metadata()?.modified()?;
                    let last_modification_date = DateTime::<Utc>::from(last_modification_date);
                    if last_modification_date.with_timezone(&date_boundary.timezone()) < date_boundary {
                        continue;
                    }
                    //let first_level_folder = recipe.first_level_folder.clone().unwrap_or_else(|| "".to_string());
                    let first_level_folder = date_to_folder_name(&last_modification_date, &recipe.first_level_folder);
                    let second_level_folder = date_to_folder_name(&last_modification_date, &recipe.second_level_folder);
                    let dest_folder = recipe.destination_folder.join(first_level_folder).join(second_level_folder);
                    if !dry_run {
                        if !dest_folder.exists() {
                            fs::create_dir_all(&dest_folder)?;  
                        }
                    }
                    let dest_file = dest_folder.join(filename.to_str().unwrap());
                    if recipe.move_files {
                        if !dry_run {    
                            fs::rename(&from_file, &dest_file)?;
                        } 
                        println!("{} {} {} - {}", "✅".green(), recipe.name.blue(), "File moved".green(), dest_file.to_str().unwrap());
                    } else {
                        if !dry_run {
                            fs::copy(&from_file, &dest_file)?;
                        } 
                        println!("{} {} {} - {}", "✅".green(), recipe.name.blue(), "File copied".green(), dest_file.to_str().unwrap());
                    }
                }
            }
        }
       
        Ok(())
    }
}

fn is_extension_allowed(file: &Path, allowed_extensions: &Option<Vec<String>>) -> bool {
    if let Some(allowed_extensions) = allowed_extensions {
        if allowed_extensions.is_empty() {
            return true;
        }
    if let Some(ext) = file.extension() {
        if let Some(ext_str) = ext.to_str() {
            return allowed_extensions.contains(&ext_str.to_string());
        } else {
            return false;
        }
        } else {
            return false;
        }
    }
    return false;
}

fn date_to_folder_name(date: &DateTime<Utc>, format: &Option<String>) -> String {
    if let Some(format) = format {
        return date.format(format.as_str()).to_string();
    } else {
        return "".to_string();
    }
}
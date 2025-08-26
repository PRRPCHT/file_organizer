use crate::file_organizer::settings::{DateComparator, Recipe, Settings};
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::*;
use rayon::prelude::*;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
pub mod settings;

/// FileOrganizer is a struct that contains the settings and the state of the file organizer.
pub struct FileOrganizer {
    settings: Settings,
    is_dry_run: bool,
    is_iterative: bool,
}

/// FileOrganizerStats is a struct that contains the statistics of the file organizer.
pub struct FileOrganizerStats {
    files_matched: u32,
    files_processed: u32,
    elapsed_time: i64,
}

impl FileOrganizer {
    /// Creates a new FileOrganizer.
    ///
    /// ### Parameters
    /// - `settings_file_path`: The path to the settings file.
    ///
    /// ### Returns
    /// - `FileOrganizer`: The FileOrganizer.
    pub fn new(settings_file_path: PathBuf, is_dry_run: bool, is_iterative: bool) -> Result<Self> {
        let mut settings = Settings::load_from_file(&settings_file_path)?;
        for recipe in &mut settings.recipes {
            if let Some(allowed_extensions) = &mut recipe.allowed_extensions {
                for extension in allowed_extensions {
                    *extension = extension.to_lowercase();
                }
            }
        }
        Ok(Self {
            settings,
            is_dry_run,
            is_iterative,
        })
    }

    /// Runs all recipes.
    ///
    /// ### Returns
    /// - `Result<(), anyhow::Error>`: The result of the recipes run.
    pub fn run(&mut self) -> anyhow::Result<()> {
        println!(
            "ℹ️ {} - Running {} recipe(s)",
            "file_organizer".blue(),
            self.settings.recipes.len()
        );
        for (i, recipe) in self.settings.recipes.iter().enumerate() {
            let stats = self.run_recipe(&recipe)?;
            println!(
                "{} {} {} - {}",
                "✅".green(),
                recipe.name.blue(),
                "Files matched".purple(),
                stats.files_matched
            );
            println!(
                "{} {} {} - {}",
                "✅".green(),
                recipe.name.blue(),
                "Files processed".purple(),
                stats.files_processed
            );
            println!(
                "{} {} {} - {}",
                "✅".green(),
                recipe.name.blue(),
                "Elapsed time".purple(),
                seconds_to_string(stats.elapsed_time / 1000)
            );
            if i < self.settings.recipes.len() - 1 {
                println!("{}", "----------------------------------------".blue());
            }
        }

        // Update last_run for all recipes if not in dry run mode
        if !self.is_dry_run {
            let last_run = Utc::now();
            let last_run = Some(last_run.format("%Y-%m-%d").to_string());
            for recipe in &mut self.settings.recipes {
                recipe.last_run = last_run.clone();
            }

            self.settings.save()?;
        }
        Ok(())
    }

    /// Runs a recipe.
    ///
    /// ### Parameters
    /// - `recipe`: The recipe to run.
    /// - `dry_run`: If true, the recipe will not be run.
    ///
    /// ### Returns
    /// - `Result<(), anyhow::Error>`: The result of the recipe run.
    fn run_recipe(&self, recipe: &Recipe) -> anyhow::Result<FileOrganizerStats> {
        if !recipe.source_folder.is_dir() {
            return Err(anyhow::Error::msg(format!(
                "{} - Source folder not a directory: {}",
                recipe.name,
                recipe.source_folder.display()
            )));
        }
        if !recipe.destination_folder.is_dir() {
            return Err(anyhow::Error::msg(format!(
                "{} - Target folder not a directory: {}",
                recipe.name,
                recipe.destination_folder.display()
            )));
        }
        print_recipe_info(recipe);

        let start_time = Utc::now().timestamp_millis();
        let date_boundary = get_date_boundary(recipe)?;
        let results: Vec<_> = if self.is_iterative {
            run_recipe_iterative(recipe, &date_boundary, self.is_dry_run)?
        } else {
            run_recipe_parallel(recipe, &date_boundary, self.is_dry_run)?
        };

        let files_processed = results.len() as u32;
        let mut files_matched = 0;

        for result in results {
            if let Ok(is_file_valid) = result {
                if is_file_valid {
                    files_matched += 1;
                }
            }
        }
        let elapsed_time = Utc::now().timestamp_millis() - start_time;
        Ok(FileOrganizerStats {
            files_matched,
            files_processed,
            elapsed_time,
        })
    }
}

/// Runs a recipe iteratively.
///
/// ### Parameters
/// - `recipe`: The recipe to run.
/// - `date_boundary`: The date boundary.
/// - `dry_run`: If true, the recipe will not be run.
///
/// ### Returns
/// - `Result<Vec<Result<bool>>>`: The results of the recipe run.
fn run_recipe_iterative(
    recipe: &Recipe,
    date_boundary: &DateTime<Utc>,
    dry_run: bool,
) -> Result<Vec<Result<bool>>> {
    let mut entries: Vec<_> =
        fs::read_dir(&recipe.source_folder)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    let results: Vec<_> = entries
        .iter()
        .map(|entry| run_for_file(entry, recipe, &date_boundary, dry_run))
        .collect();
    Ok(results)
}

/// Runs a recipe in parallel.
///
/// ### Parameters
/// - `recipe`: The recipe to run.
/// - `date_boundary`: The date boundary.
/// - `dry_run`: If true, the recipe will not be run.
///
/// ### Returns
/// - `Result<Vec<Result<bool>>>`: The results of the recipe run.
fn run_recipe_parallel(
    recipe: &Recipe,
    date_boundary: &DateTime<Utc>,
    dry_run: bool,
) -> Result<Vec<Result<bool>>> {
    let entries: Vec<_> = fs::read_dir(&recipe.source_folder)?.collect::<Result<Vec<_>, _>>()?;
    let results: Vec<_> = entries
        .par_iter()
        .map(|entry| run_for_file(entry, recipe, &date_boundary, dry_run))
        .collect();
    Ok(results)
}

/// Runs a recipe for a file.
/// The date boundary is passed as parameter in order to do not get recalculated for each call.
///
/// ### Parameters
/// - `entry`: The entry to run the recipe for.
/// - `recipe`: The recipe to run.
/// - `date_boundary`: The date boundary.
/// - `dry_run`: If true, the recipe will not be run.
///
/// ### Returns
/// - `bool`: True if the file is matched by the recipe and has been processed, false otherwise.
fn run_for_file(
    entry: &DirEntry,
    recipe: &Recipe,
    date_boundary: &DateTime<Utc>,
    dry_run: bool,
) -> anyhow::Result<bool> {
    let from_file = entry.path();
    if from_file.is_file() {
        if let Some(filename) = from_file.file_name() {
            if filename.to_str().unwrap().starts_with(".") {
                return Ok(false);
            }
            if !is_extension_allowed(&from_file, &recipe.allowed_extensions) {
                return Ok(false);
            }
            let file_date = match recipe
                .date_comparator
                .as_ref()
                .unwrap_or(&DateComparator::ModificationDate)
            {
                DateComparator::CreationDate => get_creation_date(&from_file).map_err(|e| {
                    anyhow::Error::msg(format!(
                        "{} - Error getting creation date: {}",
                        recipe.name, e
                    ))
                })?,
                DateComparator::ModificationDate => get_last_modification_date(&from_file)
                    .map_err(|e| {
                        anyhow::Error::msg(format!(
                            "{} - Error getting last modification date: {}",
                            recipe.name, e
                        ))
                    })?,
            };

            if file_date.with_timezone(&date_boundary.timezone()) < *date_boundary {
                return Ok(false);
            }
            let dest_folder = build_dest_folder(recipe, &file_date);

            if !dry_run {
                if !dest_folder.exists() {
                    fs::create_dir_all(&dest_folder)?;
                }
            }
            let dest_file = dest_folder.join(filename.to_str().unwrap());
            if recipe.move_files {
                if !dry_run {
                    if let Err(e) = fs::rename(&from_file, &dest_file) {
                        return Err(anyhow::Error::msg(format!(
                            "{} - Error moving file: {}",
                            recipe.name, e
                        )));
                    }
                }
                println!(
                    "{} {} {} - {}",
                    "✅".green(),
                    recipe.name.blue(),
                    "File moved".green(),
                    dest_file.to_str().unwrap()
                );
            } else {
                if !dry_run {
                    if let Err(e) = fs::copy(&from_file, &dest_file) {
                        return Err(anyhow::Error::msg(format!(
                            "{} - Error copying file: {}",
                            recipe.name, e
                        )));
                    }
                }
                println!(
                    "{} {} {} - {}",
                    "✅".green(),
                    recipe.name.blue(),
                    "File copied".green(),
                    dest_file.to_str().unwrap()
                );
            }
            return Ok(true);
        }
        return Ok(false);
    }
    Ok(false)
}

/// Prints the recipe info.
///
/// ### Parameters
/// - `recipe`: The recipe to print the info for.
fn print_recipe_info(recipe: &Recipe) {
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Source folder".purple(),
        recipe.source_folder.display()
    );
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Target folder".purple(),
        recipe.destination_folder.display()
    );
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Last run".purple(),
        recipe.last_run.as_ref().unwrap_or(&"Never".to_string())
    );
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Mode".purple(),
        if recipe.move_files { "Move" } else { "Copy" }
    );
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Allowed extensions".purple(),
        recipe
            .allowed_extensions
            .as_ref()
            .map(|v| v.join(", "))
            .as_ref()
            .unwrap_or(&"All".to_string())
    );
    println!(
        "{} {} {} - {}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Subfolders".purple(),
        recipe
            .subfolders
            .as_ref()
            .map(|v| v.join(", "))
            .as_ref()
            .unwrap_or(&"None".to_string())
    );
    println!(
        "{} {} {} - {:?}",
        "ℹ️".green(),
        recipe.name.blue(),
        "Date comparator".purple(),
        recipe
            .date_comparator
            .as_ref()
            .unwrap_or(&DateComparator::ModificationDate)
    );
}

/// Gets the date boundary for a recipe.
///
/// ### Parameters
/// - `recipe`: The recipe to get the date boundary for.
///
/// ### Returns
/// - `Result<DateTime<Utc>, anyhow::Error>`: The date boundary.
fn get_date_boundary(recipe: &Recipe) -> anyhow::Result<DateTime<Utc>> {
    let date_boundary = recipe
        .last_run
        .clone()
        .unwrap_or_else(|| "1970-01-01".to_string());
    let date_boundary = DateTime::parse_from_str(
        &format!("{} 00:00:00 +0000", date_boundary),
        "%Y-%m-%d %H:%M:%S %z",
    )?
    .to_utc();
    Ok(date_boundary)
}

/// Gets the last modification date of a file.
///
/// ### Parameters
/// - `file`: The file to get the last modification date of.
///
/// ### Returns
/// - `Result<DateTime<Utc>, anyhow::Error>`: The last modification date of the file.
fn get_last_modification_date(file: &Path) -> anyhow::Result<DateTime<Utc>> {
    let metadata = fs::metadata(file)?;
    let last_modification_date = metadata.modified()?;
    let last_modification_date = DateTime::<Utc>::from(last_modification_date);
    Ok(last_modification_date)
}

/// Gets the creation date of a file.
///
/// ### Parameters
/// - `file`: The file to get the creation date of.
///
/// ### Returns
/// - `Result<DateTime<Utc>, anyhow::Error>`: The creation date of the file.
fn get_creation_date(file: &Path) -> anyhow::Result<DateTime<Utc>> {
    let metadata = fs::metadata(file)?;
    let creation_date = metadata.created()?;
    let creation_date = DateTime::<Utc>::from(creation_date);
    Ok(creation_date)
}

/// Builds the destination folder.
///
/// ### Parameters
/// - `recipe`: The recipe to build the destination folder for.
/// - `last_modification_date`: The last modification date of the file.
///
/// ### Returns
/// - `PathBuf`: The destination folder.
/// - `Result<PathBuf, anyhow::Error>`: The destination folder.
fn build_dest_folder(recipe: &Recipe, last_modification_date: &DateTime<Utc>) -> PathBuf {
    let mut dest_folder = recipe.destination_folder.clone();
    if let Some(subfolders) = &recipe.subfolders {
        for subfolder in subfolders {
            let subfolder_name =
                date_to_folder_name(&last_modification_date, &Some(subfolder.clone()));
            dest_folder = dest_folder.join(subfolder_name);
        }
    }
    dest_folder
}

/// Checks if the extension of a file is allowed.
///
/// ### Parameters
/// - `file`: The file to check.
/// - `allowed_extensions`: The allowed extensions.
///
/// ### Returns
/// - `bool`: True if the extension is allowed, false otherwise.
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

/// Converts a date to a folder name.
///
/// ### Parameters
/// - `date`: The date to convert.
/// - `format`: The format to use.
///
/// ### Returns
/// - `String`: The folder name.
fn date_to_folder_name(date: &DateTime<Utc>, format: &Option<String>) -> String {
    if let Some(format) = format {
        return date.format(format.as_str()).to_string();
    } else {
        return "".to_string();
    }
}

/// Converts seconds to a string.
///
/// ### Parameters
/// - `seconds`: The seconds to convert.
///
/// ### Returns
/// - `String`: The string.
fn seconds_to_string(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    if seconds < 3600 {
        return format!("{}m {}s", seconds / 60, seconds % 60);
    }
    if seconds < 86400 {
        return format!(
            "{}h {}m {}s",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        );
    }
    return format!(
        "{}d {}h {}m {}s",
        seconds / 86400,
        (seconds % 86400) / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    );
}

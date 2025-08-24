use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, ArgMatches, Command};
use log::LevelFilter;
use simple_logger::SimpleLogger;
mod file_organizer;
use file_organizer::FileOrganizer;
use colored::*;

fn make_args() -> Command {
    command!()
        .about("Organize files into folders based on their extension")
        .arg(
            arg!(
                <RECIPES> "Path to the JSON file containing recipes"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                --dry_run "Performs a dry run test"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )

}

/// Gets the recipes path.
///
/// ### Return
/// A PathBuf with the recipes path.
fn get_recipes(matches: &ArgMatches) -> PathBuf {
    matches.get_one::<PathBuf>("RECIPES").unwrap().to_path_buf()
}


/// Gets the dry run flag.
///
/// ### Return
/// A boolean with the dry run flag.
fn get_dry_run_flag(matches: &ArgMatches) -> bool {
    matches.get_flag("dry_run")
}

fn main() {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Off
    };
    SimpleLogger::new()
        .with_colors(true) 
        .with_level(level)
        .init()
        .unwrap();
    let matches = make_args().get_matches();
    let recipes = get_recipes(&matches);
    let is_dry_run = get_dry_run_flag(&matches);
    if is_dry_run {
        println!("{} {}", "ℹ️  Dry run mode enabled".blue(), " - No files will be moved or copied");
    }

    let mut file_organizer = FileOrganizer::new(recipes);
    if let Err(e) = file_organizer.run(is_dry_run) {
        log::error!("{} {}", "❌ Error:".red().bold(), e);
    }
}

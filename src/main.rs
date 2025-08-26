use clap::{ArgAction, ArgMatches, Command, arg, command, value_parser};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
mod file_organizer;
use colored::*;
use file_organizer::FileOrganizer;

/// Makes the arguments.
///
/// ### Return
/// A Command with the arguments.
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
        .arg(
            arg!(
                --iterative "Runs iteratively instead of in parallel"
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

/// Gets the iterative flag.
///
/// ### Return
/// A boolean with the iterative flag.
fn get_iterative_flag(matches: &ArgMatches) -> bool {
    matches.get_flag("iterative")
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
    let is_iterative = get_iterative_flag(&matches);
    println!("{}", "----------------------------------------".blue());
    println!("{}", "- file_organizer                       -".blue());
    println!("{}", "----------------------------------------".blue());
    if is_dry_run {
        println!(
            "{} - No files will be moved or copied",
            "ℹ️ Dry run mode enabled".blue()
        );
    }
    if is_iterative {
        println!(
            "{} - Running iteratively instead of in parallel",
            "ℹ️ Iterative mode enabled".blue()
        );
    }

    let mut file_organizer = match FileOrganizer::new(recipes, is_dry_run, is_iterative) {
        Ok(file_organizer) => file_organizer,
        Err(e) => {
            println!("{} {}", "❌Error:".red().bold(), e);
            return;
        }
    };
    if let Err(e) = file_organizer.run() {
        println!("{} {}", "❌Error:".red().bold(), e);
    }
}

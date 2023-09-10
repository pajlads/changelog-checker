use std::{collections::HashSet, process::ExitCode};

use colored::Colorize;

mod args;
use args::parse;
pub use args::Args;

mod checker;

fn main() -> Result<(), ExitCode> {
    let args = match parse() {
        Ok(args) => args,
        Err(e) => {
            println!("{e}");
            args::print_usage();
            std::process::exit(1);
        }
    };

    let added_entries = match checker::check(&args.repo, &args.pr_number, &args.changelog_path) {
        Ok(added_entries) => added_entries,
        Err(e) => {
            println!("Error checking changelog entries: {e}");
            std::process::exit(1);
        }
    };

    let mut has_error = false;

    let unreleased_categories: HashSet<&'static str> = HashSet::from(["Unversioned", "Unreleased"]);

    for added_entry in added_entries {
        if args.strict && !unreleased_categories.contains(added_entry.category.as_str()) {
            println!(
                "Entry '{}' was added to already-released category '{}' (line {})",
                added_entry.text, added_entry.category, added_entry.line_number
            );
            if args.strict {
                has_error = true;
            }
        } else {
            println!(
                "Entry '{}' was added to category '{}' (line {})",
                added_entry.text, added_entry.category, added_entry.line_number
            );
        }
    }

    if has_error {
        println!(
            "{} At least one changelog entry was added in the wrong place",
            "ERROR:".red()
        );
        std::process::exit(1);
    }

    Ok(())
}

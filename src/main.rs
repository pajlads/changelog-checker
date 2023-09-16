use std::process::ExitCode;

use colored::Colorize;

mod args;
use args::parse_args;
pub use args::Args;

mod checker;
use checker::Checker;

fn main() -> Result<(), ExitCode> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            println!("{e}");
            args::print_usage();
            std::process::exit(1);
        }
    };

    let checker = Checker::new(args.clone());

    let added_entries = match checker.check() {
        Ok(added_entries) => added_entries,
        Err(e) => {
            println!("Error checking changelog entries: {e}");
            std::process::exit(1);
        }
    };

    let mut has_error = false;

    for added_entry in added_entries {
        if args.strict && added_entry.category != "Unversioned" {
            println!(
                "{} Entry '{}' was added to already-released category '{}'",
                "ERROR:".red(),
                added_entry.text,
                added_entry.category
            );
            has_error = true;
        } else {
            println!(
                "Entry '{}' was added to category '{}'",
                added_entry.text, added_entry.category
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

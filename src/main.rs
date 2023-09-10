use std::{
    path::Path,
    process::{ExitCode, Stdio},
};

use colored::Colorize;
use unidiff::PatchSet;

mod args;

use args::{parse_args, Args};

struct Checker {
    args: Args,
}

#[derive(Debug)]
struct AddedEntries {
    /// Category the entry was added under, e.g. "Unversioned" or "2.4.5"
    category: String,

    /// The full text entry (e.g. - Added cool feature. (#4770))
    text: String,
}

impl Checker {
    pub fn new(args: Args) -> anyhow::Result<Self> {
        Ok(Self { args })
    }

    pub fn check(&self) -> anyhow::Result<Vec<AddedEntries>> {
        let mut added_entries: Vec<AddedEntries> = Vec::new();

        let xd = std::process::Command::new("git")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-C")
            .arg(&self.args.repo_path)
            .arg("diff")
            .arg(&self.args.diff_ref)
            .arg(&self.args.changelog_path)
            .spawn()?
            .wait_with_output()?;

        let mut patch = PatchSet::new();
        patch.parse_bytes(&xd.stdout)?;

        let modified_files = patch.modified_files();

        if let Some(changelog_diff) = modified_files.first() {
            let mut additions: Vec<(usize, String)> = Vec::new();

            for hunk in changelog_diff.hunks() {
                for line in hunk.lines() {
                    if line.line_type != "+" {
                        continue;
                    }
                    additions.push((line.target_line_no.unwrap(), line.value.clone()));
                }
            }

            let contents = std::fs::read_to_string(Path::join(
                &self.args.repo_path,
                &self.args.changelog_path,
            ))?;
            let lines: Vec<&str> = contents.lines().collect();

            for (line_no, contents) in additions {
                let before = &lines[0..line_no];

                for xd in before.iter().rev() {
                    if xd.starts_with("## ") {
                        added_entries.push(AddedEntries {
                            category: xd.trim_start_matches("## ").to_owned(),
                            text: contents,
                        });
                        break;
                    }
                }
            }
        }

        Ok(added_entries)
    }
}

fn main() -> anyhow::Result<ExitCode> {
    let args = parse_args()?;

    let checker = Checker::new(args.clone())?;

    let added_entries = checker.check()?;

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
        Ok(1.into())
    } else {
        Ok(0.into())
    }
}

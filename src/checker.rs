use crate::Args;
use std::error::Error;
use std::fmt;
use std::{path::Path, process::Stdio};
use unidiff::PatchSet;

pub struct Checker {
    args: Args,
}

#[derive(Debug)]
pub struct AddedEntries {
    /// Category the entry was added under, e.g. "Unversioned" or "2.4.5"
    pub category: String,

    /// The full text entry (e.g. - Added cool feature. (#4770))
    pub text: String,
}

#[derive(Debug)]
pub enum CheckerError {
    Io(std::io::Error),
    Diff(unidiff::Error),
}

impl From<std::io::Error> for CheckerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<unidiff::Error> for CheckerError {
    fn from(e: unidiff::Error) -> Self {
        Self::Diff(e)
    }
}

impl fmt::Display for CheckerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => {
                write!(f, "io error during check: {e}")
            }
            Self::Diff(e) => {
                write!(f, "diff error during check: {e}")
            }
        }
    }
}

impl Error for CheckerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Diff(e) => Some(e),
        }
    }
}

impl Checker {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    pub fn check(&self) -> Result<Vec<AddedEntries>, CheckerError> {
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

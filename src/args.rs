use std::path::PathBuf;

#[derive(Debug, Clone)]
/// Check added changelog entries
pub struct Args {
    /// path to the changelog file, relative to the git repository root
    pub changelog_path: String,

    /// diff to compare head to. defaults to origin/master
    pub diff_ref: String,

    /// relative or absolute path to the repository
    pub repo_path: PathBuf,

    /// whether to error out if any changelog addition was made to a non-unversioned section
    pub strict: bool,
}

pub fn print_usage() {
    println!(
        "Usage: changelog-checker [OPTIONS] <repo-path>

Options:
  --changelog-path <changelog-path>     Path to the changelog path relative
                                        to repo-path.
                                        Default value is CHANGELOG.md
  --diff-ref <diff-ref>                 Ref to diff against.
                                        Default value is origin/master
  --strict                              Enables strict checks for added changelog
                                        entries, ensuring they are only added
                                        to the Unreleased section."
    );
}

pub(crate) fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut changelog_path: String = "CHANGELOG.md".to_owned();
    let mut diff_ref: String = "origin/master".to_owned();
    let mut repo_path: Option<PathBuf> = None;
    let mut parser = lexopt::Parser::from_env();
    let mut strict = false;
    while let Some(arg) = parser.next()? {
        match arg {
            Long("changelog-path") => {
                changelog_path = parser.value()?.parse()?;
            }
            Long("diff-ref") => {
                diff_ref = parser.value()?.parse()?;
            }
            Long("repo-path") => {}
            Long("strict") => {
                strict = true;
            }
            Long("help") => {
                print_usage();
                std::process::exit(0);
            }
            Value(v) if repo_path.is_none() => {
                repo_path = Some(v.try_into().unwrap());
            }
            Long(v) => return Err(lexopt::Error::UnexpectedOption(v.to_owned())),
            Short(v) => return Err(lexopt::Error::UnexpectedOption(v.into())),
            Value(v) => return Err(lexopt::Error::UnexpectedOption(v.into_string().unwrap())),
        }
    }

    Ok(Args {
        changelog_path,
        diff_ref,
        repo_path: repo_path.ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("repo-path".to_owned()),
        })?,
        strict,
    })
}

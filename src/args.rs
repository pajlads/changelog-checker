#[derive(Debug, Clone)]
/// Check added changelog entries
pub struct Args {
    /// path to the changelog file, relative to the git repository root
    /// defaults to CHANGELOG.md
    pub changelog_path: String,

    /// repo the PR belongs to, should be the organization name + the repository name
    /// (e.g. Chatterino/chatterino2)
    pub repo: String,

    /// PR to check
    /// (e.g. 4938)
    pub pr_number: String,

    /// whether to error out if any changelog addition was made to a non-unversioned section
    pub strict: bool,
}

pub fn print_usage() {
    println!(
        "Usage: changelog-checker [OPTIONS] <repo> <pr> (e.g. changelog-checker --strict Chatterino2/chatterino 4921)

Options:
  --changelog-path <changelog-path>     Path to the changelog path relative
                                        to repo-path.
                                        Default value is CHANGELOG.md
  --strict                              Enables strict checks for added changelog
                                        entries, ensuring they are only added
                                        to the Unreleased section."
    );
}

pub(crate) fn parse() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut changelog_path: String = "CHANGELOG.md".to_owned();
    let mut repo: Option<String> = None;
    let mut pr_number: Option<String> = None;
    let mut parser = lexopt::Parser::from_env();
    let mut strict = false;
    while let Some(arg) = parser.next()? {
        match arg {
            Long("changelog-path") => {
                changelog_path = parser.value()?.parse()?;
            }
            Long("repo-path") => {}
            Long("strict") => {
                if let Some(value) = parser.optional_value() {
                    strict = value.parse()?;
                } else {
                    strict = true;
                }
            }
            Long("help") => {
                print_usage();
                std::process::exit(0);
            }
            Value(v) if repo.is_none() => {
                repo = Some(v.into_string()?);
            }
            Value(v) if pr_number.is_none() => {
                pr_number = Some(v.into_string()?);
            }
            Long(v) => return Err(lexopt::Error::UnexpectedOption(v.to_owned())),
            Short(v) => return Err(lexopt::Error::UnexpectedOption(v.into())),
            Value(v) => return Err(lexopt::Error::UnexpectedOption(v.into_string().unwrap())),
        }
    }

    Ok(Args {
        changelog_path,
        repo: repo.ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("repo".to_owned()),
        })?,
        pr_number: pr_number.ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("pr".to_owned()),
        })?,
        strict,
    })
}

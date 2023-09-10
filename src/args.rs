use std::path::PathBuf;

#[derive(Debug, Clone)]
/// Check added changelog entries
pub(crate) struct Args {
    /// path to the changelog file, relative to the git repository root
    pub(crate) changelog_path: String,

    /// diff to compare head to. defaults to origin/master
    pub(crate) diff_ref: String,

    /// relative or absolute path to the repository
    pub(crate) repo_path: PathBuf,

    /// whether to error out if any changelog addition was made to a non-unversioned section
    pub(crate) strict: bool,
}

fn usage() -> String {
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
        .to_owned()
}

pub(crate) fn parse_args() -> anyhow::Result<Args> {
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
                println!("{}", usage());
                std::process::exit(0);
            }
            Value(v) if repo_path.is_none() => {
                repo_path = Some(v.try_into()?);
            }
            Long(v) => anyhow::bail!("unexpected argument --{v}"),
            Short(v) => anyhow::bail!("unexpected argument -{v}"),
            Value(v) => anyhow::bail!("unexpected positional argument {v:?}"),
        }
    }

    Ok(Args {
        changelog_path,
        diff_ref,
        repo_path: repo_path.ok_or_else(|| anyhow::anyhow!(usage()))?,
        strict,
    })
}

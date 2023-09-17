use anyhow::Result;
use serde::Deserialize;

#[derive(Debug)]
pub struct AddedEntry {
    /// Category the entry was added under, e.g. "Unversioned" or "2.4.5"
    pub category: String,

    /// The full text entry (e.g. - Added cool feature. (#4770))
    pub text: String,

    /// The 1-indexed line number of this new entry in the modified changelog file
    pub line_number: usize,
}

type GitHubPrFiles = Vec<Change>;

#[derive(Deserialize, Debug)]
struct Change {
    filename: String,
    #[serde(default)]
    patch: String,
    raw_url: String,
}

#[derive(Debug)]
struct Hunk {
    added_lines: Vec<(usize, String)>,
}

fn parse_hunks(patch: &str) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    // 0-based line numbers
    let mut line_no: usize = 0;
    let mut hunk: Option<Hunk> = None;

    for l in patch.lines() {
        if l.starts_with("@@ ") {
            if let Some(hunk) = hunk {
                hunks.push(hunk);
            }

            // Example line: @@ -90,7 +90,8 @@ HighlightingPage::HighlightingPage()

            let first_plus = l.find('+').unwrap() + 1;
            let first_comma = l[first_plus..].find(',').unwrap() + first_plus;
            // -1 to 0-index lines
            line_no = str::parse::<usize>(&l[first_plus..first_comma]).unwrap() - 1;
            // println!("line_no: {line_no}");
            hunk = Some(Hunk {
                added_lines: Vec::new(),
            });
        } else if let Some(l) = l.strip_prefix('+') {
            if let Some(hunk) = &mut hunk {
                hunk.added_lines.push((line_no, l.to_owned()));
            }
            line_no += 1;
        } else {
            line_no += 1;
        }
    }
    if let Some(hunk) = hunk {
        hunks.push(hunk);
    }

    hunks
}

/// Returns a list of added changelog entries in the given repo & PR
///
/// # Arguments
///
/// * `repo_name` - the full repo name, including its owner (e.g. `Chatterino/chatterino2`)
/// * `pr_number` - the PR number, as a string (e.g. `4919`)
/// * `changelog_path` - the path & filename of the changelog file, relative to the repo root (e.g.
/// CHANGELOG.md)
///
/// # Examples
///
/// ```
/// let entries = checker::check("Chatterino/chatterino2", "4631", "CHANGELOG.md");
/// if let Some(entries) = entries {
///     println!("Changelog entries: {entries:?}");
/// }
/// ```
pub fn check(repo_name: &str, pr_number: &str, changelog_path: &str) -> Result<Vec<AddedEntry>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("changelog checker https://github.com/pajlads/changelog-checker")
        .build()?;

    let url = reqwest::Url::parse(
        format!("https://api.github.com/repos/{repo_name}/pulls/{pr_number}/files").as_str(),
    )?;

    let resp: GitHubPrFiles = client.get(url).send()?.json()?;
    let mut added_entries: Vec<AddedEntry> = Vec::new();

    let changelog_diff_entry = resp.into_iter().find(|c| c.filename == changelog_path);
    if let Some(changelog_diff_entry) = changelog_diff_entry {
        let hunks = parse_hunks(&changelog_diff_entry.patch);
        let pr_additions = hunks.into_iter().flat_map(|h| h.added_lines);

        let changelog_contents = client.get(changelog_diff_entry.raw_url).send()?.text()?;
        let changelog_lines: Vec<&str> = changelog_contents.lines().collect();

        for (line_no, contents) in pr_additions {
            let before = &changelog_lines[0..line_no];

            for xd in before.iter().rev() {
                if xd.starts_with("## ") {
                    added_entries.push(AddedEntry {
                        category: xd.trim_start_matches("## ").to_owned(),
                        text: contents,
                        line_number: line_no + 1,
                    });
                    break;
                }
            }
        }
    }

    Ok(added_entries)
}

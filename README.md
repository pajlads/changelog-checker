# Changelog Checker

If you follow the same changelog format I use, this action will allow you to make sure PRs don't accidentally add their entry into an already-released changelog section.

## Usage

### Simple unskippable checker

This will always run, and fail if a changelog entry has been place in the wrong section.

```yaml
---
name: Changelog Checker

on: pull_request

jobs:
  changelog-check:
    runs-on: ubuntu-latest
    steps:
      - uses: pajlads/changelog-checker@v1.0.1
```

### Skippable checker

This can let you skip the changelog checker with the label `skip-changelog-checker`

```yaml
---
name: Changelog Checker

on:
  pull_request:
    types:
      - labeled
      - unlabeled
      - opened
      - synchronize
      - reopened

jobs:
  changelog-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v6
        id: label-checker
        with:
          result-encoding: "string"
          script: |
            const response = await github.rest.issues.listLabelsOnIssue({
              issue_number: context.payload.pull_request.number,
              owner: context.repo.owner,
              repo: context.repo.repo
            });
            if (new Set(response.data.map(label => label.name)).has("skip-changelog-checker")) {
              return "skip";
            }
            return "";

      - uses: pajlads/changelog-checker@v1.0.1
        if: steps.label-checker.outputs.result != 'skip'
```

## Contributing

When wanting to make a release, increase the version number in this file & in the `Dockerfile` in the same commit.

When the commit has landed in the main branch, tag it & the packages will be pushed automatically.

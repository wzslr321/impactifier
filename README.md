# Impactifier

Impactifier is a tool designed to analyze and visualize the impact of code changes, across the whole - possibly separated - codebase. 
It helps developers identify potential downstream effects of changes, allowing for more reliable and frequent releases.

## Table of Contents
1. [Overview](#overview)
2. [Key Features](#key-features)
3. [Roadmap](#roadmap)
4. [Getting Started](#getting-started)
   - [CI/CD](#cicd)
   - [CLI](#cli)
   - [Configuration File](#configuration-file)
6. [Contributing](#contributing)
7. [License](#license)

## Overview

Impactifier provides an automated approach to change impact analysis, initally designed to serve as CI/CD tool. 
Aims to improve frequency and reliability of releases, by generating impact reports for engineers to be able to push the changes with confidence.
(Eventually) Highly configurable, with possibility to add custom rules. Your contracts generator does weird stuff under the hood, yet you want to 
see what impact on front-end modifying its query handler have? Don't worry, *Impactifier* got your back.

## Key Features

- **Automated Impact Analysis:** Detect changes and their potential effects across the codebase.
- **Configurable rules specification:** Analyses the impact based on configurable set of rules, to support various tech-stacks.
- **Integration with CI/CD Pipelines:** Seamlessly integrate with GitHub Actions to provide impact reports in pull requests.
- **Performance:** Built in Rust for high performance and low latency. After all, it is all about faster releases.

## Roadmap

We want to support specifying more detailed context of analysis, such as:
- specific file and directory/file:
    `$ impactifier . features/auth --to-branch develop` - Analyse current file and its impact regarding 
    current state of `--to-branch`'s `./features/auth` directory


## Getting Started

### CI/CD

**Impactifier** main usage is designed to be CI/CD. 

It supports both `pull_request` and`push` trigger actions.

Simplified example:

```yaml
on: [push, pull_request]
jobs:
  run:
    runs-on: ubuntu-latest
    needs: extract-info
    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: run cli 
        run: |
          if [ "${{ github.event_name }}" = "push" ]; then
            impactifier --from-branch ${{ $BRANCH_NAME }} --commit ${{ $COMMIT_SHA }}
          elif [ "${{ github.event_name }}" = "pull_request" ]; then
            impactifier --from-branch ${{ $BRANCH_NAME }} --to-branch ${{ $TARGET_BRANCH }}
          fi
```

Flags information can easily be extracted inside the github action itself. 
Full example can be found [here](github.com/wzslr321/impactifier/example/.github/impactifier-action.yaml)


### CLI 

**Impactifier** can be used as a CLI tool.

It can either clone a repository or open an existing - in you local file path - one.

To clone a repository you have to specify its url via `--url` flag.
If repository is private, you need to pass `--access-token` flag.

Example:
```sh
$ impactifier --url github.com/wzslr321/foobar --access-token=very_private
```

The command above will actually allow you to perform your first impact analysis!
Since there is no further configuration, it will try to go with defaults, which are 
your local changes & current branch - so basically just output of plain `git diff`

To specify what exactly you want to analyze, you have a few options:
- `--from-branch`: branch to compare, defaults to the current branch
- `--to-branch`: branch to compare the one from `--from-branch` with
- `--commit-id`: specific commit to analyze
    To specify the branch where the commit is located, use `--from-branch`

To analyze local repository, you can specify `--path` flag. By default it checks both current directory,
and the one above it, to handle both local & ci/cd usage.

### Configuration File
All necessary config can be passed via flags - which take highest priority - but config file is also supported.

Below is an example of a impactifier-config.yaml file:

```yaml
repository:
  url: "https://github.com/example/repository.git"
  path: "/path/to/local/repository"
  access_token: "${GITHUB_ACCESS_TOKEN}"

options:
  on:
    - push
    - pull_request
  clone_into: "/tmp/clone"
```

#### Configuration Options
**repository:** Contains details about the repository to be analyzed.
- `url`: The URL of the repository to clone. This can be omitted if you provide a path.
- `path`: The path to a local repository. This can be used instead of cloning from a URL.
- `access_token`: An optional access token used for cloning private repositories. This can be set via an environment variable (e.g., `${GITHUB_ACCESS_TOKEN}`).

**options:** General options for Impactifier.
- `on`: A list of actions (push, pull_request) that trigger the analysis.
- `clone_into`: Specifies the directory where the repository should be cloned.

#### Overriding Configuration with CLI Flags 
While the configuration file provides a convenient way to manage settings, you can override any of these options directly from the command line using CLI flags. This allows for flexibility, especially when running Impactifier in different environments (e.g., local vs. CI/CD).

For example:
```sh
$ impactifier --config my-config.yaml --from-branch develop --to-branch main
```

In the above command:

- `--config my-config.yaml`: Specifies a custom configuration file.
- `--from-branch develop` and `--to-branch main`: Override the branches defined in the configuration file.

**Priority Order**
Impactifier follows a specific priority order when determining which settings to use:

- CLI Arguments/Flags: Highest priority. Values provided here override both default settings and those in the configuration file.
- Configuration File: If no CLI argument is provided for a setting, Impactifier looks for it in the configuration file.
- Default Configuration File: If you don’t specify a configuration file with the --config flag, Impactifier looks for a file 
named impactifier-config.yaml in the current working directory. If no file is found, it uses the default settings.
- Defaults: If neither a CLI argument nor a configuration file value is provided, Impactifier falls back to its default settings.

## Contributing
We welcome contributions to Impactifier! Please refer to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to contribute.

## License
Impactifier is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

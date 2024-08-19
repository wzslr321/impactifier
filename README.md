# Impactifier

Impactifier is a tool designed to analyze and visualize the impact of code changes in monorepos, with an initial focus on CQRS architectures. It helps developers identify potential downstream effects of changes, allowing for more confident and efficient releases.

## Table of Contents
1. [Overview](#overview)
2. [Key Features](#key-features)
3. [Roadmap](#roadmap)
4. [Getting Started](#getting-started)
   - [CI/CD](#cicd)
   - [CLI](#cli)
6. [Contributing](#contributing)
7. [License](#license)

## Overview

Impactifier provides an automated approach to change impact analysis, initally designed to serve as CI/CD tool. 
Aims to improve frequency and reliabilty of releases, by generating impact reports for engineers to be able to push the changes with confidance.
(Eventually) Higly configurable, with possibility to add custom rules. Your contracts generator does weird stuff under the hood, yet you want to 
see what impact on front-end modyfing its query handler have? Don't worry, *Impactifier* got your back.

## Key Features

- **Automated Impact Analysis:** Detect changes and their potential effects across the codebase.
- **Configurable rules specification:** Analyses the impact based on configurable set of rules, to support various tech-stacks.
- **Integration with CI/CD Pipelines:** Seamlessly integrate with GitHub Actions to provide impact reports in pull requests.
- **Performance:** Built in Rust for high performance and low latency. After all, it is all about faster releases.

## Roadmap

We want to support specyfing more detailed context of analysis, such as:
- specific file and directory/file:
    `$ impactifier . features/auth --to-branch develop` - Analyse current file and its impact regarding 
    current state of `--to-branch`'s `./feautres/auth` directory


## Getting Started

### CI/CD

### CLI 

**Impactifier** can be used as a CLI tool.

It can either clone a repository or open an existing - in you local file path - one.

To clone a repository you have to specify its url via `--url` flag.
If repository is private, you need to pass `--access-token` flag.

Example:
```sh
$ impactifier --url github.com/wzslr321/foobar --access-token=very_private
```

The commend above will actually allow you to perform your first impact analysis!
Since there is no further configuration, it will try to go with defaults, which are 
your local changes & current branch - so basically just output of plain `git diff`

To specify what exactly you want to analyze, you have a few options:
- `--from-branch`: branch to compare, defaults to the current branch
- `--to-branch`: branch to compare the one from `--from_branch` with
- `--commit-id`: specific commit to analze
    To specify the branch where the commit is located, use `--from-branch`

## Contributing
We welcome contributions to Impactifier! Please refer to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to contribute.

## License
Impactifier is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

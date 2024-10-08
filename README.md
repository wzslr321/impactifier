# Impactifier

Impactifier is a tool designed to analyze the impact of code changes, across the whole - possibly separated - codebase, and allow to take various actions based on that. It aims to help developers identify potential downstream effects of changes, allowing for more reliable and frequent releases. Can be used as either local tool or as part of CI/CD.  

> This project is build in public, therefore I strive to stream its development on [Twitch](https://www.twitch.tv/creatixd)

<br>

---

<p align = "center">
  <b> <i> Show your support by giving a :star: </b> </i>
</p>

---

<br>


## Table of Contents
1. [Overview](#overview)
2. [Key Features](#key-features)
3. [Getting Started](#getting-started)
   - [CI/CD](#cicd)
   - [CLI](#cli)
   - [Configuration File](#configuration-file)
4. [Contributing](#contributing)
5. [License](#license)

## Overview

> *Please note, that a lot of this README was created with help of ChatGPT, so it is far from perfect*

Impactifier provides an automated approach to change impact analysis, designed to serve as both CI/CD and local tool. 
Aims to improve frequency and reliability of releases, by generating impact reports for engineers to be able to push the changes with confidence. (Eventually) Highly configurable, with possibility to add custom rules and actions.

Your contracts generator does weird stuff under the hood, yet you want to see what impact on front-end modifying its query handler have? Don't worry, **Impactifier got your back.**

## Key Features

- **Automated Impact Analysis:** Detect changes and their potential effects across the codebase.
- **Configurable rules specification:** Analyses the impact based on configurable set of rules, to support various tech-stacks.
- **Integration with CI/CD Pipelines:** Seamlessly integrate with GitHub Actions to provide impact reports in pull requests.
- **Performance:** Built in Rust for high performance and low latency. After all, it is all about faster releases.

## Getting Started

> Please note, that it doesn't yet work at all. Stuff below is meant to show how it *hopefully* will be used.

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
Full example can be found [here](github.com/wzslr321/impactifier/.github/impactifier.yaml)


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
  url: "https://github.com/wzslr321/impactifier"
options:
  clone_into: "./repo"

rules:
  - name: "Detect API Changes"
    trigger:
      path: "api/"
      pattern: "func (\\w+)Handler"
    transform:
      name: "toApiEndpoint"
      steps:
        - name: "toLowerCase"
        - name: "replace"
          args:
            pattern: "Handler$"
            with: "_endpoint"
        - name: "prepend"
          args:
            value: "/api/"
        - name: "customFunction"
          args:
            script: |
              fn transform(context) {
                  if context.class_name == "SpecialClass" {
                      return "/special" + context.matched_string;
                  } else {
                      return context.matched_string;
                  }
              }
    matcher:
      path: "client/"
      pattern: "ApiClient.call('$transform')"
    action:
      alert_level: "Severe"
      message: "API changed"

```

## Contributing
We welcome contributions to Impactifier! Please refer to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to contribute.

## License
Impactifier is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

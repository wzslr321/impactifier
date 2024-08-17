# Impactifier

Impactifier is a tool designed to analyze and visualize the impact of code changes in monorepos, with an initial focus on CQRS architectures. It helps developers identify potential downstream effects of changes, allowing for more confident and efficient releases.

## Table of Contents
1. [Overview](#overview)
2. [Key Features](#key-features)
3. [Roadmap](#roadmap)
4. [Getting Started](#getting-started)
   - [Prerequisites](#prerequisites)
   - [Installation](#installation)
   - [Configuration](#configuration)
   - [Usage](#usage)
5. [Architecture](#architecture)
6. [Contributing](#contributing)
7. [License](#license)

## Overview

Impactifier provides an automated approach to change impact analysis, specifically tailored for monorepos and CQRS architectures. By analyzing the codebase, it can identify which parts of the system are affected by changes in handlers or contracts, generating detailed reports that can be integrated into CI/CD pipelines.

## Key Features

- **Automated Impact Analysis:** Detect changes and their potential effects across the codebase.
- **CQRS Support:** Special focus on handling changes in CQRS-based architectures.
- **Integration with CI/CD Pipelines:** Seamlessly integrate with GitHub Actions to provide impact reports in pull requests.
- **Future Expansion:** Plans to support REST API contracts and multi-repo setups.
- **Performance:** Built in Rust for high performance and low latency.

## Roadmap
We are continuously improving Impactifier. Here’s a look at our planned features:

Support for REST APIs: Analyze impact for RESTful services.
Multi-repo Support: Extend functionality to projects with multiple repositories.
Enhanced Reporting: Develop advanced graphical reports and visualization tools.
Check out our Roadmap for more details.

## Getting Started

TODO: 

## Contributing
We welcome contributions to Impactifier! Please refer to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to contribute.

## License
Impactifier is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

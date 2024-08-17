# Impactifier Architecture

## Overview
Impactifier consists of several core components:
1. **Static Analysis Engine:** Analyzes the codebase to build a dependency graph and tracks changes.
2. **Impact Analyzer:** Processes the diff between code changes to determine downstream impacts.
3. **Report Generator:** Produces detailed impact reports that integrate directly into CI/CD pipelines.
4. **CI/CD Integration Module:** Handles communication with CI tools like GitHub Actions, injecting reports into PRs.

## System Components
### 1. Static Analysis Engine
- **Purpose:** Parse and analyze the codebase to understand the structure, dependencies, and relationships between modules.
- **Key Sub-Components:**
  - **AST Parser:** Converts source code into an Abstract Syntax Tree (AST) for further analysis.
  - **Dependency Graph Builder:** Constructs a graph representing dependencies between functions, modules, and contracts.

### 2. Impact Analyzer
- **Purpose:** Detect and assess the impact of changes by analyzing diffs between commits.
- **Key Features:**
  - **Contract Change Detection:** Identify changes in API contracts (e.g., adding/removing parameters).
  - **Handler Logic Tracking:** Trace how changes in handler logic propagate through the system.

### 3. Report Generator
- **Purpose:** Generate human-readable reports for developers, summarizing what components might be impacted by a given change.
- **Report Formats:**
  - **Text-based Reports:** Summary in pull requests.
  - **Graphical Reports:** Dependency trees or flow diagrams (future implementation).

### 4. CI/CD Integration Module
- **Purpose:** Integrate seamlessly with CI/CD pipelines.
- **Initial Focus:** GitHub Actions (webhook integration for triggering analysis).

## Future Extensions
- Support for multi-repo architectures.
- REST API contract change analysis.
- Integration with other CI/CD platforms like Jenkins and GitLab CI.

# Code Guidelines

This document outlines the coding standards and best practices for the Impactifier project. Adhering to these guidelines will help ensure code quality, readability, and maintainability.

## Table of Contents
1. [General Principles](#general-principles)
2. [Code Style](#code-style)
3. [Documentation](#documentation)
4. [Testing](#testing)
5. [Commit Messages](#commit-messages)
6. [Code Reviews](#code-reviews)
7. [Branching and Merging](#branching-and-merging)
9. [Performance](#performance)

## General Principles
- **Clarity:** Write code that is easy to read and understand.
- **Simplicity:** Keep the code as simple as possible. Avoid overengineering.
- **Consistency:** Follow the conventions used throughout the project to maintain a consistent codebase.

## Code Style

### Rust Conventions
- **Format:** Use `rustfmt` to format your code. Ensure that the code adheres to the Rust style guidelines.
- **Naming Conventions:**
  - **Variables:** Use `snake_case` for variable names (e.g., `my_variable`).
  - **Functions:** Use `snake_case` for function names (e.g., `calculate_total`).
  - **Structs and Enums:** Use `CamelCase` for struct and enum names (e.g., `MyStruct`, `MyEnum`).
- **Line Length:** Keep lines to a maximum of 100 characters.

### Code Structure
- **File Organization:** Organize files logically in the directory structure. Group related functionality together.
- **Module Organization:** Use modules to encapsulate related functionality. Keep module files and directories named appropriately.
- **Error Handling:** Use Rust’s error handling patterns, such as `Result` and `Option`, to manage errors gracefully.

## Documentation

### Comments
- **Inline Comments:** Use comments to explain complex code logic or decisions. Place comments above the relevant code.
- **Doc Comments:** Use Rust doc comments (`///`) for public API documentation. Document functions, structs, and enums with descriptions of their purpose and usage.

## Testing

### Unit Tests
- Write unit tests for all core functionality.
- Place tests in the same module or file where the code is defined, using the `#[cfg(test)]` module.

### Test Coverage
- Aim for high test coverage but prioritize critical paths and functionality.

## Commit Messages

### Format
- Use the following format for commit messages:
[TYPE] [SCOPE]: [SHORT DESCRIPTION]

- **TYPE:** feat, fix, chore, docs, style, refactor, test
- **SCOPE:** A brief description of the affected area (e.g., "parser", "frontend")
- **SHORT DESCRIPTION:** A concise summary of the changes.

### Examples
- `feat(parser): add support for new syntax`
- `fix(backend): correct API endpoint response format`

## Code Reviews

### Process
- All code changes must be reviewed by at least one other team member before merging.
- Reviewers should check for code correctness, style adherence, and performance considerations.
- Address feedback and update the PR accordingly before merging.

## Branching and Merging

### Branching Strategy
- **Main Branches:**
- `main`: Production-ready code.
- `develop`: Integration branch for features before they go to `main`.
- **Feature Branches:** Use feature branches for new features or bug fixes. Name branches descriptively (e.g., `feature/add-login-page`, `bugfix/fix-header-layout`).

## Performance

### Optimization
- Optimize code for performance when necessary. Focus on optimizing critical paths.
- Use Rust’s profiling tools to identify and address performance bottlenecks.

---

Adhering to these guidelines will help maintain a high-quality codebase and make collaboration more effective. For any questions or clarifications, please reach out to the project maintainers.

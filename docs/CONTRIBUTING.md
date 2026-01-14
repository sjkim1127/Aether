# Contributing to Aether Codegen

First off, thanks for taking the time to contribute! ❤️

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [I Have a Question](#i-have-a-question)
- [I Want To Contribute](#i-want-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Your First Code Contribution](#your-first-code-contribution)
- [Styleguides](#styleguides)
  - [Commit Messages](#commit-messages)

## Code of Conduct

This project and everyone participating in it is governed by the [Aether Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## I Have a Question

> If you want to ask a question, we assume that you have read the available [Documentation](docs/README.md).

Before you ask a question, it is best to search for existing [Issues](https://github.com/sjkim1127/aether-codegen/issues) that might help you. In case you have found a suitable issue and still need clarification, you can write your question in this issue. It is also advisable to search the internet for answers first.

## I Want To Contribute

### Reporting Bugs

- Make sure that you are using the latest version.
- Perform a search to see if the problem has already been reported.
- Collect information about the bug:
  - Stack trace (Traceback)
  - OS, Platform and Version (Windows, Linux, macOS, x86, ARM)
  - Version of the interpreter, compiler, SDK, etc.
- Open an new [Issue](https://github.com/sjkim1127/aether-codegen/issues/new).

### Suggesting Enhancements

- Use the [Discussions](https://github.com/sjkim1127/aether-codegen/discussions) tab.
- Explain why this enhancement would be useful to most Aether users.

### Your First Code Contribution

1. Fork the project.
2. Create a branch (`git checkout -b feat/amazing-feature`).
3. Commit your changes (`git commit -m 'feat: Add some amazing feature'`).
4. Push to the branch (`git push origin feat/amazing-feature`).
5. Open a Pull Request.

#### Development Setup

**Prerequisites:**
- Rust 1.75+
- Node.js 18+
- Python 3.10+ (optional, for scripts)

**Setup:**
```bash
# Clone repository
git clone https://github.com/sjkim1127/aether-codegen.git
cd aether-codegen

# Build Rust crates
cargo build

# Build Node.js bindings
cd crates/aether-node
npm install
npm run build
```

**Testing:**
```bash
# Run Rust tests
cargo test

# Run Node.js tests
cd crates/aether-node
npm test
```

## Styleguides

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries

Example: `feat(core): Add support for nested templates`

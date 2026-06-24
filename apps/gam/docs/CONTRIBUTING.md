# Contributing to GAM

Thank you for your interest in contributing to GAM! This document provides guidelines and instructions for contributing.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Coding Standards](#coding-standards)
- [Bug Reports](#bug-reports)

---

## Code of Conduct

Please read and follow our [Code of Conduct](./CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive experience for everyone.

---

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
    ```bash
    git clone https://github.com/zintaen/gam.git
    cd gam
    ```
3. **Install dependencies** with pnpm:
    ```bash
    pnpm install
    ```
4. **Start the dev server:**
    ```bash
    pnpm dev
    ```

---

## Development Workflow

### Project Structure

| Directory         | Purpose                    |
| ----------------- | -------------------------- |
| `src-tauri/`      | Tauri backend (Rust)       |
| `src-tauri/src/`  | Rust commands and services |
| `src/`            | React frontend             |
| `src/components/` | Reusable UI components     |
| `src/hooks/`      | Custom React hooks         |
| `tests/`          | Unit tests (Vitest)        |

### Useful Commands

```bash
pnpm dev          # Start Tauri + Vite dev server
pnpm test         # Run all tests
pnpm test -- --coverage # Run tests with coverage report
pnpm test:watch   # Run tests in watch mode
pnpm build        # Build for production
```

### Adding a New Feature

1. Create a branch from `main`:
    ```bash
    git checkout -b feature/your-feature-name
    ```
2. Implement your changes.
3. Add or update tests in `tests/`.
4. Ensure all tests pass: `pnpm test`
5. Submit a pull request.

---

## Pull Request Guidelines

- **One feature per PR** — Keep changes focused and reviewable.
- **Write tests** — New core features should include unit tests. UI features should use `@testing-library/react`.
- **Update docs** — If your change affects the user-facing behavior, update the README.
- **Follow commit conventions** — Use clear, descriptive commit messages:
    ```
    feat: add alias drag-and-drop reordering
    fix: handle missing .gitconfig gracefully
    docs: update installation instructions
    test: add suggest service edge case tests
    ```

---

## Coding Standards

- **TypeScript** — All frontend code must be written in TypeScript with strict mode enabled.
- **Rust** — Backend commands and services are written in Rust in `src-tauri/src/`.
- **Naming** — Use camelCase for variables/functions, PascalCase for components/classes.
- **Components** — Each component gets its own file. Keep components focused and composable.
- **Security** — All Tauri commands must catch errors and return `{ success, error }` results.

---

## Bug Reports

When filing a bug report, please include:

1. **OS and version** (e.g., macOS 14.2, Windows 11, Ubuntu 24.04)
2. **Node.js version** (`node --version`)
3. **pnpm version** (`pnpm --version`)
4. **Steps to reproduce** the issue
5. **Expected vs. actual behavior**
6. **Error messages or screenshots** if applicable

Use the [GitHub Issues](../../issues) page to file reports.

---

Thank you for helping make GAM better! 🎉

---

## 🍌 Support

If GAM saves you time and brainpower, consider fueling its development with a banana!

[<img src="https://cdn.buymeacoffee.com/buttons/v2/default-violet.png" alt="Buy Me A Banana" height="50">](https://buymeacoffee.com/zintaen)

Or scan the QR Code:

<img src="docs/screenshots/buy-me-a-coffee.png" alt="Buy Me A Banana QR" width="200" style="border-radius: 12px;">

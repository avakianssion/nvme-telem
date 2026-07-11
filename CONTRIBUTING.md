# Contributing to nvme-telem

This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Submitting Changes](#submitting-changes)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitLab
2. **Clone your fork**:

```bash
   git clone https://gitlab.com/YOUR_USERNAME/nvme-telem.git
   cd nvme-telem
```

3. **Add upstream remote**:

```bash
   git remote add upstream https://gitlab.com/avakianssion/nvme-telem.git
```

## Development Setup

### Prerequisites

- Rust 1.91.0+ (see `rust-toolchain.toml`)
- Linux system with NVMe devices (for testing)
- Root/sudo access (required for NVMe device access)
- Development packages:

```bash
  # Rocky Linux / RHEL / Fedora
  sudo dnf install clang-devel llvm-devel

  # Ubuntu / Debian
  sudo apt install clang libclang-dev llvm-dev
```

### Building

```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test

# Run the sanity check example (build as user, run as root to access devices)
cargo build --example sanity_check
sudo ./target/debug/examples/sanity_check
```

## Submitting Changes

Merge requests target the **`rc`** branch (not `main`. `main` only receives
releases). Commit messages are never parsed by tooling, so write them however
you like; instead, every MR must include a **change file**.

### Change files

A change file is a small Markdown file in `.changeset/` that declares the
semver impact of your MR and provides the changelog text. These files
accumulate on `rc` and are consumed automatically when a release is cut, so
the directory is normally empty.

If you have [knope](https://knope.tech/installation) installed, it can
create the file interactively:

```bash
knope document-change
```

Or create `.changeset/<anything>.md` by hand (filename doesn't matter, must
be unique):

```markdown
---
default: minor
---

# Add parsing for vendor-specific telemetry log pages

Optional longer description in full Markdown — code blocks, links,
whatever helps a user of the crate understand the change.
```

The bump line must be exactly one of:

| line             | when                                    |
| ---------------- | --------------------------------------- |
| `default: major` | breaking API change                     |
| `default: minor` | new functionality, backwards compatible |
| `default: patch` | bug fix, no API change                  |

(`default` is the tooling's name for the single package in this repo, leave
it as-is, don't replace it with the crate name.)

While this crate is pre-1.0, cargo treats `0.x` minor bumps as breaking — if
in doubt whether something is `major` or `minor`, say so in the MR and we'll
sort it out in review.

CI fails the MR if the change file is missing or malformed. MRs that
shouldn't appear in the changelog or affect the version (CI tweaks, typo
fixes) can get the `skip-changelog` label from a maintainer instead.

### Checklist before opening an MR

- [ ] Change file added in `.changeset/`
- [ ] `cargo test --all-features` passes
- [ ] `cargo fmt --all` and `cargo clippy --all-targets --all-features` are clean

For how releases work end to end, see [RELEASING.md](RELEASING.md).

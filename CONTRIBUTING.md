# Contributing to nvme-telem

Thank you for your interest in contributing to nvme-telem. This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)

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

- Rust 1.85.0+ or latest
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

# Run the sanity check example
sudo cargo run --example sanity_check
```

# nvme-telem

[![crates.io](https://img.shields.io/crates/v/nvme-telem.svg)](https://crates.io/crates/nvme-telem)
[![docs.rs](https://docs.rs/nvme-telem/badge.svg)](https://docs.rs/nvme-telem)
[![license](https://img.shields.io/crates/l/nvme-telem.svg)](LICENSE)

A Rust library for collecting NVMe telemetry and S.M.A.R.T. data on Linux.

## Features

- Read NVMe SMART/Health logs
- Extract detailed controller identification data
- Organized structs for different metric categories (Identity, Capacity, Thermals, etc.)

## Installation

```bash
cargo add nvme-telem
```

## Quick start example

```rust
use nvme_telem::nvme::{self, Device};

fn main() {
    for ctrl in nvme::list_nvme_controllers() {
        let dev = format!("/dev/{}", ctrl);
        match Device::open(&dev).and_then(|device| device.smart_log()) {
            Ok(smart) => println!(
                "{} (S/N {}): {} K, {} power-on hours, critical warning {:#x}",
                smart.nvme_name,
                smart.serial_number,
                smart.temperature,
                smart.power_on_hours,
                smart.critical_warning,
            ),
            Err(e) => eprintln!("{}: {} (root access required?)", ctrl, e),
        }
    }
}
```

Programs using this library need root privileges at runtime to access
`/dev/nvme*` devices — build as a normal user, run the binary with `sudo`.

## Requirements

**Runtime**

- Linux
- One or more NVMe storage devices
- Root/sudo privileges (for `/dev/nvme*` access)

**Build**

- Rust 1.91.0+ (see `rust-toolchain.toml`)
- libclang, for generating the NVMe ioctl bindings:

```bash
# Ubuntu / Debian
sudo apt install clang libclang-dev llvm-dev

# Rocky Linux / RHEL / Fedora
sudo dnf install clang-devel llvm-devel
```

## Running the example

```bash
cargo build --example sanity_check
sudo ./target/debug/examples/sanity_check
```

(Build as your normal user, not under `sudo`, running cargo as root
leaves root-owned files in `target/` and may not find your toolchain.)

## Safety

This library uses `unsafe` code to make ioctl calls to NVMe devices. All
unsafe code is isolated in the device access functions.

## Contributing

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). MRs target
the `rc` branch and include a small change file; the rest is automated.

## License

MIT — see [LICENSE](LICENSE).

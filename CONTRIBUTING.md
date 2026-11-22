# Contributing to Rover Kernel

Thank you for your interest in contributing to `rover-kernel`! This document outlines our engineering standards, architecture rules, testing criteria, and development workflows.

---

## Core Principles

`rover-kernel` is engineered as a modern, modular, high-performance operating system kernel in native Rust. Every contribution must adhere to the following architectural rules:

1. **Pure `#![no_std]` Bare-Metal Execution:**
   The kernel operates on bare metal without any standard library runtime. All kernel code must remain `#![no_std]` and `#![no_main]`.

2. **Clean Modular Subsystems:**
   Every subsystem is strictly separated into its own module. **`mod.rs` files must contain only submodule declarations (`pub mod ...;`) and public API re-exports (`pub use ...;`).** No implementation logic or state may reside in `mod.rs` files.

3. **Rover Ecosystem Integration:**
   Firmware abstractions and pre-boot protocol specifications must come from [`rover-uefi`](https://github.com/muhammad-fiaz/rover-uefi) and [`rover-bootloader`](https://github.com/muhammad-fiaz/rover-bootloader). Boot information handoff must conform strictly to the Rover Boot Protocol v1.

4. **Safety and Defensive Design:**
   All `unsafe` blocks must be strictly justified with detailed `// SAFETY:` comments. Memory allocators and system calls must validate boundaries and verify pointer alignment before dereferencing.

5. **Zero Warnings Policy:**
   All source code must compile with zero warnings under `cargo clippy --target x86_64-unknown-none --all-targets -- -D warnings` and pass `cargo fmt --all -- --check`.

---

## Prerequisites

Ensure you have a recent Rust toolchain installed along with the bare-metal target triple and developer components:

```bash
rustup target add x86_64-unknown-none
rustup component add clippy rustfmt
```

For virtualization and boot testing, install QEMU with OVMF firmware:
- QEMU (`qemu-system-x86_64`)
- EDK II OVMF x86_64 firmware

---

## Development Workflow

### 1. Host Unit Tests

Host tests exercise memory management, sync primitives, data structures, and protocol parsing:

```bash
cargo test --tests --no-default-features
```

### 2. Bare-Metal Kernel Compilation

Compile the statically linked kernel executable:

```bash
cargo build --release --target x86_64-unknown-none
```

### 3. Linting and Formatting

Run the strict linter and code formatter:

```bash
cargo fmt --all -- --check
cargo clippy --target x86_64-unknown-none --all-targets -- -D warnings
```

### 4. QEMU Boot Testing

Prepare an EFI System Partition directory with `BOOTX64.EFI`, `rover.cfg`, and `kernel.elf`, and execute QEMU:

```bash
qemu-system-x86_64 \
    -machine q35 \
    -m 512M \
    -cpu qemu64 \
    -drive if=pflash,format=raw,readonly=on,file=path/to/OVMF_CODE.fd \
    -drive format=raw,file=fat:rw:esp \
    -serial stdio \
    -display none \
    -net none
```

---

## Pull Request Process

1. Fork the repository and create a feature branch (`git checkout -b feature/my-feature`).
2. Implement your changes following our modular architecture guidelines.
3. Ensure all tests pass (`cargo test --tests --no-default-features`) and clippy passes with zero warnings.
4. Verify boot handoff and serial log output under QEMU.
5. Submit a pull request with a descriptive title and detailed summary of changes.

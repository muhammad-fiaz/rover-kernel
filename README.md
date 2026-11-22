<div align="center">

# Rover Kernel

<a href="https://github.com/muhammad-fiaz/rover-kernel"><img src="https://img.shields.io/badge/Rust-2024%20Edition-orange.svg?logo=rust" alt="Rust Edition"></a>
<a href="https://github.com/muhammad-fiaz/rover-kernel"><img src="https://img.shields.io/github/stars/muhammad-fiaz/rover-kernel" alt="GitHub stars"></a>
<a href="https://github.com/muhammad-fiaz/rover-kernel/issues"><img src="https://img.shields.io/github/issues/muhammad-fiaz/rover-kernel" alt="GitHub issues"></a>
<a href="https://github.com/muhammad-fiaz/rover-kernel/pulls"><img src="https://img.shields.io/github/issues-pr/muhammad-fiaz/rover-kernel" alt="GitHub pull requests"></a>
<a href="https://github.com/muhammad-fiaz/rover-kernel/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License"></a>

<p align="center">
  A modern, modular, high-performance operating system kernel implemented natively in Rust.
</p>

</div>

---

## Overview

**Rover Kernel** is an independent, modular, high-performance operating system kernel written natively in `#![no_std]` Rust (`Rust 2024` edition). Engineered as the core operating system component within the Rover ecosystem, it consumes the authoritative [Rover Boot Protocol v1](https://github.com/muhammad-fiaz/rover-bootloader) handed off by **Rover Bootloader** and **Rover UEFI** to initialize platform hardware, establish virtual memory paging, manage system heap allocations, configure hardware interrupts, dispatch system calls, and schedule preemptive kernel tasks.

---

## Subsystem Architecture

Rover Kernel is architected with strict single-responsibility separation:

```text
kernel/
├── linker/
│   └── x86_64.ld              # Physical address 0x200000 linker script
├── src/
│   ├── arch/                  # Architecture-specific hardware abstraction
│   │   └── x86_64/            # CPUID, GDT, IDT, MSRs, CR0..CR4, Port I/O
│   ├── block/                 # Block device layer and RAM block devices
│   ├── boot/                  # Rover Boot Protocol v1 parser & validator
│   ├── diagnostics/           # Memory diagnostics and CPU feature reporting
│   ├── drivers/               # Hardware device drivers
│   │   ├── fbcon.rs           # Direct GOP linear framebuffer text console
│   │   ├── pci.rs             # PCI configuration space enumerator
│   │   ├── ps2.rs             # PS/2 keyboard controller & scancode translator
│   │   └── serial.rs          # 16550 UART serial driver (COM1 115200 8N1)
│   ├── fs/                    # Virtual File System (VFS), Inodes, RamFS, FD table
│   ├── interrupts/            # Local APIC timer, legacy 8259 PIC, IRQ dispatch
│   ├── logging/               # ANSI terminal color logger and kprint/kinfo macros
│   ├── mm/                    # Memory management
│   │   ├── address.rs         # Strongly typed PhysAddr and VirtAddr
│   │   ├── frame.rs           # Usable RAM bitmap physical frame allocator
│   │   ├── heap.rs            # Linked-list kernel heap allocator (GlobalAlloc)
│   │   └── paging.rs          # 4-level x86-64 page tables and translations
│   ├── process/               # Process Control Block (PCB) and task isolation
│   ├── sched/                 # Preemptive cooperative task scheduler & context switch
│   ├── security/              # Capability-based security, SMEP, SMAP, NXE, usercopy
│   ├── sync/                  # Concurrency: SpinLock, IrqSafeSpinLock, Mutex, RwLock
│   ├── syscall/               # Fast system call entry (LSTAR, STAR, FMASK) and ABI
│   ├── time/                  # TSC cycle timer, monotonic clock, sleep ticks
│   ├── lib.rs                 # Library client-side API re-exports
│   ├── main.rs                # Entry point (_start naked thunk & kernel_main)
│   └── panic.rs               # Diagnostic panic handler
└── tests/                     # Foundation, memory, and subsystem host unit tests
```

---

## Features

- **Pure Native Rust (`Rust 2024`):** Strictly `#![no_std]` and `#![no_main]`, enforcing zero reliance on external host C runtimes.
- **Rover Boot Protocol v1 Compliance:** Seamless handoff from `rover-bootloader` and `rover-uefi`, receiving normalized memory maps, linear framebuffer geometry, ACPI RSDP, and SMBIOS entry points.
- **Robust Memory Management:** Usable physical frame allocator dynamically constructed from UEFI memory descriptors, 4-level identity/virtual paging, and an embedded linked-list kernel heap allocator implementing `core::alloc::GlobalAlloc`.
- **Preemptive Multitasking:** Calibrated Local APIC periodic timer generating hardware interrupts, preemptive task scheduling, and cooperative context switching.
- **Hardware Security Mitigations:** Detection and activation of No-Execute (`NXE` in `EFER`), Supervisor Mode Execution Prevention (`SMEP`), Supervisor Mode Access Prevention (`SMAP`), and fault-checked usercopy boundaries.
- **Fast System Call ABI:** Direct x86-64 `SYSCALL`/`SYSRET` acceleration via MSR registers (`STAR`, `LSTAR`, `FMASK`), offering low-latency kernel dispatch.
- **Modular Hardware Drivers:** Polled 16550 UART serial communication, full PCI bus hierarchy enumeration, 8x16 VGA bitmap font graphical framebuffer console (`fbcon`), and PS/2 keyboard scancode translation.

---

## Building and Verification

### Prerequisites

Install the `x86_64-unknown-none` target:

```bash
rustup target add x86_64-unknown-none
rustup component add clippy rustfmt
```

### Build Kernel Executable

```bash
cargo build --release --target x86_64-unknown-none
```

The resulting statically linked kernel ELF executable is placed at:
`target/x86_64-unknown-none/release/rover-kernel`

### Run Host Test Suite

Unit and integration tests run directly on the host development environment:

```bash
cargo test --tests --no-default-features
```

### Static Analysis and Code Formatting

```bash
cargo fmt --all -- --check
cargo clippy --target x86_64-unknown-none --all-targets -- -D warnings
```

---

## Booting Under QEMU

Rover Kernel is booted via Rover Bootloader in an EFI System Partition (ESP):

1. **Prepare the ESP Directory:**

```text
esp/
├── EFI/
│   └── BOOT/
│       └── BOOTX64.EFI       # Built rover-bootloader.efi
├── rover.cfg                 # Boot configuration file
└── kernel.elf                # Built rover-kernel executable
```

2. **Boot Configuration (`rover.cfg`):**

```ini
default = 0
timeout = 0

entry "Rover Kernel" {
    kernel = \kernel.elf
    resolution = 1024x768
    cmdline = console=ttyS0 loglevel=debug root=/dev/ram0
}
```

3. **Launch QEMU with OVMF Firmware:**

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

## Contributing

Contributions are welcome! Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) guide for details on our engineering standards, architecture rules, `#![no_std]` constraints, and pull request workflow.

---

## Security Policy

Security is a primary design goal of the Rover operating system. Please consult [SECURITY.md](SECURITY.md) for vulnerability reporting procedures and our security architecture guidelines.

---

## Acknowledgment

The Linux kernel source code served as an architectural reference for hardware registers, protocols, and standard subsystem architecture during the design of Rover Kernel.

---

## License

Rover Kernel is licensed under the [Apache License, Version 2.0](LICENSE).

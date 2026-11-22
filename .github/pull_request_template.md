## Description
<!-- Provide a brief description of the changes introduced by this pull request. -->

## Motivation & Context
<!-- Why is this change required? What problem does it solve? -->
<!-- If it fixes an open issue, link to it here: e.g. Fixes #123 -->

## Type of Change
<!-- Mark the relevant options with an 'x': -->
- [ ] Bug fix (non-breaking change fixing an issue or CPU exception)
- [ ] New feature (non-breaking change adding a new driver, syscall, or capability)
- [ ] Breaking change (change causing existing kernel APIs or ABIs to alter)
- [ ] Documentation update
- [ ] Architecture / hardware backend enhancement

## Subsystems Affected
- [ ] Architecture & CPU (`arch::x86_64`)
- [ ] Memory Management (`mm::frame`, `mm::paging`, `mm::heap`, `mm::address`)
- [ ] Interrupts & Timing (`interrupts::apic`, `interrupts::pic8259`, `time`)
- [ ] Hardware Device Drivers (`drivers::serial`, `drivers::pci`, `drivers::fbcon`, `drivers::ps2`)
- [ ] Task Scheduling (`sched::scheduler`, `sched::task`, `sched::context`)
- [ ] System Call Infrastructure (`syscall::abi`, `syscall::entry`, `syscall::dispatch`)
- [ ] Filesystem & Storage (`fs::vfs`, `fs::ramfs`, `fs::fd`, `block::device`)
- [ ] Security & Protections (`security::protection`, `security::usercopy`, `security::capability`)
- [ ] Boot Protocol & Diagnostics (`boot::info`, `diagnostics`)

## Checklist
- [ ] My code adheres to the project's `#![no_std]` bare-metal architecture.
- [ ] All `mod.rs` files contain ONLY module declarations and public API re-exports.
- [ ] I have run `cargo fmt --all -- --check` and confirmed formatting passes cleanly.
- [ ] I have run `cargo clippy --target x86_64-unknown-none --all-targets -- -D warnings` with 0 warnings.
- [ ] I have run `cargo test --tests --no-default-features` and all host tests pass.
- [ ] I have verified kernel compilation (`cargo build --release --target x86_64-unknown-none`).
- [ ] I have validated kernel execution under QEMU with OVMF firmware.
- [ ] I have updated relevant documentation where appropriate.

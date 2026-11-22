# Security Policy

The `rover-kernel` project takes kernel-level security, memory protection, and execution isolation seriously. This document outlines our vulnerability reporting process and security posture.

---

## Supported Versions

Security updates and vulnerability patches are maintained for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability or security defect in `rover-kernel`, please report it responsibly so that we can address it before public disclosure.

**Please DO NOT report security issues via public GitHub issues, discussions, or pull requests.**

### Reporting Channels

You can report vulnerabilities securely via:

1. **Email:** Send your report directly to **[contact@muhammadfiaz.com](mailto:contact@muhammadfiaz.com)** with the subject line `[SECURITY] rover-kernel: <Brief Summary>`.
2. **GitHub Security Advisory:** Submit a private vulnerability report via [GitHub Private Vulnerability Reporting](https://github.com/muhammad-fiaz/rover-kernel/security/advisories/new).

### Information to Include

To assist in timely triage and resolution, please provide:

- **Component:** The affected kernel subsystem (e.g., `mm::paging`, `syscall::dispatch`, `security::usercopy`, `sched::scheduler`).
- **Severity & Impact:** Assessment of vulnerability impact (e.g., privilege escalation, kernel memory corruption, arbitrary code execution, denial of service).
- **Reproduction Steps:** A minimal reproducible scenario, user-space binary, or configuration demonstrating the issue.
- **Environment:** Target architecture (`x86_64-unknown-none`), Rust toolchain version, and QEMU configuration.

---

## Security Architecture

`rover-kernel` implements defense-in-depth security principles:

- **Hardware Protections:**
  - **No-Execute (NX):** The `EFER.NXE` bit is enabled during early boot to enforce execution prevention on data and stack pages.
  - **Supervisor Mode Execution Prevention (SMEP):** Configured in `CR4` on supported processors to prevent the kernel from executing user-space memory.
  - **Supervisor Mode Access Prevention (SMAP):** Configured in `CR4` to block unintentional kernel reads/writes from/to user-space addresses.

- **Boundary Validation:**
  - All data transfers between user-space and kernel-space must pass through `copy_from_user` and `copy_to_user` routines, which verify pointer canonicality and address bounds against user address space limits.

- **Capability-Based Access Control:**
  - Operations requiring elevated privileges are verified against fine-grained process capabilities before execution.

- **Memory Safety:**
  - Written in `#![no_std]` Rust, guaranteeing memory safety and data race protection at compile time, with explicit isolation of unsafe hardware operations.

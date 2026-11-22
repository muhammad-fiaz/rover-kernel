//! # Subsystem Unit & Integration Tests
//!
//! Validates VFS, `RamFS`, Block Device, Capabilities, Mutex, `RwLock`, `OnceLock`,
//! and PS/2 scancode decoding.

#![allow(
    clippy::unwrap_used,
    clippy::unreadable_literal,
    clippy::borrow_as_ptr,
    clippy::doc_markdown
)]

extern crate alloc;
use alloc::string::String;
use alloc::sync::Arc;

use rover_kernel::block::{BlockDevice, RamBlockDevice};
use rover_kernel::drivers::ps2::Ps2Keyboard;
use rover_kernel::fs::{FileSystem, Inode, NodeType, RamFile, RamFs};
use rover_kernel::security::{Capability, CapabilitySet, USER_VIRTUAL_MAX, is_valid_user_range};
use rover_kernel::sync::{Mutex, OnceLock, RwLock};

#[test]
fn test_capabilities() {
    let mut caps = CapabilitySet::empty();
    assert!(!caps.has(Capability::SysAdmin));
    assert!(!caps.has(Capability::NetAdmin));

    caps.grant(Capability::SysAdmin);
    assert!(caps.has(Capability::SysAdmin));
    assert!(!caps.has(Capability::NetAdmin));

    caps.grant(Capability::NetAdmin);
    assert!(caps.has(Capability::SysAdmin));
    assert!(caps.has(Capability::NetAdmin));

    caps.revoke(Capability::SysAdmin);
    assert!(!caps.has(Capability::SysAdmin));
    assert!(caps.has(Capability::NetAdmin));
}

#[test]
fn test_user_range_validation() {
    assert!(!is_valid_user_range(0, 100)); // Null
    assert!(!is_valid_user_range(0x1000, 0)); // Zero len

    // Valid lower half user addresses
    assert!(is_valid_user_range(0x0000_0000_0040_0000, 4096));
    assert!(is_valid_user_range(USER_VIRTUAL_MAX - 100, 50));

    // Higher half kernel addresses must be rejected
    assert!(!is_valid_user_range(0xFFFF_8000_0000_0000, 4096));
    assert!(!is_valid_user_range(USER_VIRTUAL_MAX, 100)); // Crosses boundary
}

#[test]
fn test_ram_block_device() {
    let mut dev = RamBlockDevice::new(512, 16);
    assert_eq!(dev.block_size(), 512);
    assert_eq!(dev.total_blocks(), 16);

    let write_buf = [0xABu8; 512];
    assert!(dev.write_blocks(2, 1, &write_buf).is_ok());

    let mut read_buf = [0u8; 512];
    assert!(dev.read_blocks(2, 1, &mut read_buf).is_ok());
    assert_eq!(read_buf, write_buf);

    // Reading unwritten block returns zeroes
    let mut zero_buf = [0xFFu8; 512];
    assert!(dev.read_blocks(0, 1, &mut zero_buf).is_ok());
    assert_eq!(zero_buf, [0u8; 512]);

    // Out of bounds read returns error
    let mut err_buf = [0u8; 512];
    assert!(dev.read_blocks(16, 1, &mut err_buf).is_err());
}

#[test]
fn test_ramfs_hierarchy_and_io() {
    let fs = RamFs::default();
    let root = fs.root();
    assert_eq!(root.stat().node_type, NodeType::Directory);

    let file = Arc::new(RamFile::with_data(b"Hello from Rover Kernel RamFS!"));
    let dir = fs.root_dir();
    dir.add_child(String::from("hello.txt"), file);

    let looked_up = dir.lookup("hello.txt").unwrap();
    assert_eq!(looked_up.stat().node_type, NodeType::File);
    assert_eq!(looked_up.stat().size, 30);

    let mut read_buf = [0u8; 30];
    assert_eq!(looked_up.read(0, &mut read_buf).unwrap(), 30);
    assert_eq!(&read_buf, b"Hello from Rover Kernel RamFS!");

    let entries = dir.read_dir().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0], "hello.txt");
}

#[test]
fn test_sync_mutex() {
    let mutex = Mutex::new(10);
    assert!(!mutex.is_locked());

    {
        let mut guard = mutex.lock();
        assert!(mutex.is_locked());
        *guard += 5;
    }

    assert!(!mutex.is_locked());
    assert_eq!(*mutex.lock(), 15);
}

#[test]
fn test_sync_rwlock() {
    let rwlock = RwLock::new(42);

    {
        let r1 = rwlock.read();
        let r2 = rwlock.read();
        assert_eq!(*r1, 42);
        assert_eq!(*r2, 42);
    }

    {
        let mut w = rwlock.write();
        *w = 100;
    }

    assert_eq!(*rwlock.read(), 100);
}

#[test]
fn test_sync_oncelock() {
    let cell: OnceLock<u32> = OnceLock::default();
    assert!(cell.get().is_none());

    assert!(cell.set(999).is_ok());
    assert_eq!(cell.get(), Some(&999));
    assert!(cell.set(111).is_err()); // Cannot overwrite
}

#[test]
fn test_ps2_keyboard_scancode_translation() {
    let mut kbd = Ps2Keyboard::default();

    // Key 'A' make code = 0x1E
    assert_eq!(kbd.process_scancode(0x1E), Some('a'));

    // Key 'A' break code = 0x9E (bit 7 set) -> ignored
    assert_eq!(kbd.process_scancode(0x9E), None);

    // Left shift press = 0x2A
    assert_eq!(kbd.process_scancode(0x2A), None);

    // Key 'A' with shift -> 'A'
    assert_eq!(kbd.process_scancode(0x1E), Some('A'));

    // Left shift release = 0xAA
    assert_eq!(kbd.process_scancode(0xAA), None);

    // Key 'A' without shift -> 'a'
    assert_eq!(kbd.process_scancode(0x1E), Some('a'));
}

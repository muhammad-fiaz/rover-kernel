//! # x86-64 Global Descriptor Table (GDT)
//!
//! Configures 64-bit kernel code/data segments and user-mode segments,
//! loads the GDTR register, and reloads segment selectors.

/// Kernel code segment selector (index 1, RPL 0).
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

/// Kernel data segment selector (index 2, RPL 0).
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;

/// User data segment selector (index 3, RPL 3).
pub const USER_DATA_SELECTOR: u16 = 0x18 | 3;

/// User code segment selector (index 4, RPL 3).
pub const USER_CODE_SELECTOR: u16 = 0x20 | 3;

/// A single 64-bit segment descriptor in the GDT.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct GdtEntry(pub u64);

impl GdtEntry {
    /// The null descriptor (must be first entry in GDT).
    pub const NULL: Self = Self(0);

    /// Constructs a 64-bit code segment descriptor.
    ///
    /// Sets: Present (bit 47), DPL (bits 45..46), Descriptor Type (bit 44 = code/data),
    /// Executable (bit 43), Readable (bit 41), Long Mode (bit 53).
    #[must_use]
    pub const fn code_segment(dpl: u8) -> Self {
        let dpl_bits = ((dpl & 3) as u64) << 45;
        // Present (1 << 47) | DPL | Code/Data (1 << 44) | Executable (1 << 43) | Readable (1 << 41) | Long (1 << 53)
        let val = (1 << 47) | dpl_bits | (1 << 44) | (1 << 43) | (1 << 41) | (1 << 53);
        Self(val)
    }

    /// Constructs a 64-bit data segment descriptor.
    ///
    /// Sets: Present (bit 47), DPL (bits 45..46), Descriptor Type (bit 44 = code/data),
    /// Writable (bit 41).
    #[must_use]
    pub const fn data_segment(dpl: u8) -> Self {
        let dpl_bits = ((dpl & 3) as u64) << 45;
        // Present (1 << 47) | DPL | Code/Data (1 << 44) | Writable (1 << 41)
        let val = (1 << 47) | dpl_bits | (1 << 44) | (1 << 41);
        Self(val)
    }
}

/// The Global Descriptor Table structure for early kernel initialization.
#[repr(C, align(16))]
pub struct GlobalDescriptorTable {
    entries: [GdtEntry; 5],
}

impl GlobalDescriptorTable {
    /// Creates the standard early kernel GDT.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [
                GdtEntry::NULL,
                GdtEntry::code_segment(0), // 0x08: Kernel Code
                GdtEntry::data_segment(0), // 0x10: Kernel Data
                GdtEntry::data_segment(3), // 0x18: User Data
                GdtEntry::code_segment(3), // 0x20: User Code
            ],
        }
    }
}

impl Default for GlobalDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Pointer structure passed to the `lgdt` instruction.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct GdtPointer {
    /// Size of the table in bytes minus 1.
    pub limit: u16,
    /// Physical/virtual linear address of the GDT.
    pub base: u64,
}

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

/// Initializes and loads the early Global Descriptor Table.
///
/// # Safety
///
/// Must be called during single-threaded early initialization before interrupts are enabled.
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn init_early_gdt() {
    let ptr = GdtPointer {
        limit: (core::mem::size_of::<GlobalDescriptorTable>() - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u64,
    };

    // SAFETY: Loading valid static GDT and reloading CS/DS/SS selectors.
    unsafe {
        core::arch::asm!(
            "lgdt [{ptr}]",
            "push {cs}",
            "lea {tmp}, [2f + rip]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ds, {ds:x}",
            "mov es, {ds:x}",
            "mov ss, {ds:x}",
            "mov fs, {zero:x}",
            "mov gs, {zero:x}",
            ptr = in(reg) &raw const ptr,
            cs = in(reg) u64::from(KERNEL_CODE_SELECTOR),
            ds = in(reg) KERNEL_DATA_SELECTOR,
            zero = in(reg) 0u16,
            tmp = out(reg) _,
            options(preserves_flags)
        );
    }
}

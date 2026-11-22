//! # x86-64 Interrupt Descriptor Table (IDT)
//!
//! Configures 256 interrupt gates, default CPU exception handlers,
//! and loads the IDTR register via `lidt`.

use core::fmt;

/// An entry in the 64-bit Interrupt Descriptor Table.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    /// Creates a missing/non-present IDT gate entry.
    #[must_use]
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Sets the handler address, code selector, and gate attributes.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_handler(&mut self, handler: usize) {
        self.selector = super::gdt::KERNEL_CODE_SELECTOR;
        self.ist = 0;
        // Present (0x80) | 64-bit Interrupt Gate (0x0E)
        self.type_attr = 0x8E;
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

/// Pointer structure passed to the `lidt` instruction.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct IdtPointer {
    /// Limit in bytes of the IDT minus 1.
    pub limit: u16,
    /// Linear base address of the IDT array.
    pub base: u64,
}

/// The Interrupt Descriptor Table containing 256 gates.
#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    entries: [IdtEntry; 256],
}

impl InterruptDescriptorTable {
    /// Creates a new IDT with all gates initially missing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }

    /// Sets the handler function for a specific interrupt vector.
    pub fn set_gate(&mut self, vector: usize, handler: usize) {
        if vector < 256 {
            self.entries[vector].set_handler(handler);
        }
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Stack frame automatically saved by the CPU when an exception occurs.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    /// Instruction pointer where exception occurred.
    pub rip: u64,
    /// Code segment selector.
    pub cs: u64,
    /// CPU flags at exception time.
    pub rflags: u64,
    /// Stack pointer at exception time.
    pub rsp: u64,
    /// Stack segment selector.
    pub ss: u64,
}

impl fmt::Display for ExceptionStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RIP: 0x{:016x}, CS: 0x{:04x}, RFLAGS: 0x{:016x}\nRSP: 0x{:016x}, SS: 0x{:04x}",
            self.rip, self.cs, self.rflags, self.rsp, self.ss
        )
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Registers a gate handler for the specified interrupt vector in the system IDT.
pub fn register_idt_gate(vector: usize, handler: usize) {
    // SAFETY: Initialized early and serialized across kernel initialization.
    unsafe {
        (*core::ptr::addr_of_mut!(IDT)).set_gate(vector, handler);
    }
}

/// Human-readable names for CPU exceptions (vectors 0 to 31).
pub const EXCEPTION_NAMES: [&str; 32] = [
    "Divide Error (#DE)",
    "Debug (#DB)",
    "Non-Maskable Interrupt (#NMI)",
    "Breakpoint (#BP)",
    "Overflow (#OF)",
    "Bound Range Exceeded (#BR)",
    "Invalid Opcode (#UD)",
    "Device Not Available (#NM)",
    "Double Fault (#DF)",
    "Coprocessor Segment Overrun",
    "Invalid TSS (#TS)",
    "Segment Not Present (#NP)",
    "Stack-Segment Fault (#SS)",
    "General Protection Fault (#GP)",
    "Page Fault (#PF)",
    "Reserved",
    "x87 Floating-Point Exception (#MF)",
    "Alignment Check (#AC)",
    "Machine Check (#MC)",
    "SIMD Floating-Point Exception (#XM)",
    "Virtualization Exception (#VE)",
    "Control Protection Exception (#CP)",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

// Macro to generate naked assembly stubs for exceptions with and without error codes.
macro_rules! exception_handler_no_error {
    ($name:ident, $vec:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push 0", // dummy error code
                concat!("push ", $vec),
                "jmp default_exception_entry"
            );
        }
    };
}

macro_rules! exception_handler_with_error {
    ($name:ident, $vec:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(concat!("push ", $vec), "jmp default_exception_entry");
        }
    };
}

exception_handler_no_error!(exc_0, 0);
exception_handler_no_error!(exc_1, 1);
exception_handler_no_error!(exc_2, 2);
exception_handler_no_error!(exc_3, 3);
exception_handler_no_error!(exc_4, 4);
exception_handler_no_error!(exc_5, 5);
exception_handler_no_error!(exc_6, 6);
exception_handler_no_error!(exc_7, 7);
exception_handler_with_error!(exc_8, 8);
exception_handler_no_error!(exc_9, 9);
exception_handler_with_error!(exc_10, 10);
exception_handler_with_error!(exc_11, 11);
exception_handler_with_error!(exc_12, 12);
exception_handler_with_error!(exc_13, 13);
exception_handler_with_error!(exc_14, 14);
exception_handler_no_error!(exc_15, 15);
exception_handler_no_error!(exc_16, 16);
exception_handler_with_error!(exc_17, 17);
exception_handler_no_error!(exc_18, 18);
exception_handler_no_error!(exc_19, 19);
exception_handler_no_error!(exc_20, 20);
exception_handler_with_error!(exc_21, 21);

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn default_exception_entry() {
    core::arch::naked_asm!(
        "push rax",
        "push rcx",
        "push rdx",
        "push rbx",
        "push rbp",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov rdi, [rsp + 15 * 8]", // vector
        "mov rsi, [rsp + 16 * 8]", // error code
        "lea rdx, [rsp + 17 * 8]", // ExceptionStackFrame
        "call handle_exception_dispatch",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rbp",
        "pop rbx",
        "pop rdx",
        "pop rcx",
        "pop rax",
        "add rsp, 16", // pop vector and error code
        "iretq"
    );
}

#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation)]
extern "C" fn handle_exception_dispatch(
    vector: u64,
    error_code: u64,
    frame: *const ExceptionStackFrame,
) {
    let frame_ref = unsafe { &*frame };
    let name = if (vector as usize) < EXCEPTION_NAMES.len() {
        EXCEPTION_NAMES[vector as usize]
    } else {
        "Unknown Exception"
    };

    crate::kerror!("=================================================");
    crate::kerror!(
        "!!! CPU EXCEPTION OCCURRED: {} (vector {}) !!!",
        name,
        vector
    );
    crate::kerror!("Error Code: 0x{:016x}", error_code);
    crate::kerror!("{}", frame_ref);

    if vector == 14 {
        let cr2 = super::cpu::read_cr2();
        crate::kerror!("Faulting Address (CR2): 0x{:016x}", cr2);
    }
    crate::kerror!("=================================================");

    super::cpu::halt_loop();
}

/// Initializes and loads the early Interrupt Descriptor Table.
///
/// # Safety
///
/// Must be called during single-threaded early initialization.
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn init_early_idt() {
    // Populate exception handlers 0..21
    unsafe {
        let idt = &mut *core::ptr::addr_of_mut!(IDT);
        idt.set_gate(0, exc_0 as *const () as usize);
        idt.set_gate(1, exc_1 as *const () as usize);
        idt.set_gate(2, exc_2 as *const () as usize);
        idt.set_gate(3, exc_3 as *const () as usize);
        idt.set_gate(4, exc_4 as *const () as usize);
        idt.set_gate(5, exc_5 as *const () as usize);
        idt.set_gate(6, exc_6 as *const () as usize);
        idt.set_gate(7, exc_7 as *const () as usize);
        idt.set_gate(8, exc_8 as *const () as usize);
        idt.set_gate(9, exc_9 as *const () as usize);
        idt.set_gate(10, exc_10 as *const () as usize);
        idt.set_gate(11, exc_11 as *const () as usize);
        idt.set_gate(12, exc_12 as *const () as usize);
        idt.set_gate(13, exc_13 as *const () as usize);
        idt.set_gate(14, exc_14 as *const () as usize);
        idt.set_gate(15, exc_15 as *const () as usize);
        idt.set_gate(16, exc_16 as *const () as usize);
        idt.set_gate(17, exc_17 as *const () as usize);
        idt.set_gate(18, exc_18 as *const () as usize);
        idt.set_gate(19, exc_19 as *const () as usize);
        idt.set_gate(20, exc_20 as *const () as usize);
        idt.set_gate(21, exc_21 as *const () as usize);

        let ptr = IdtPointer {
            limit: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        core::arch::asm!("lidt [{0}]", in(reg) &raw const ptr, options(preserves_flags));
    }
}

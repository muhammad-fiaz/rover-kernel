/*
 * Rover Kernel Linker Script for x86_64
 *
 * Places the kernel at physical address 0x200000 (2 MiB), leaving low physical
 * memory untouched. All loadable sections are aligned to 4 KiB boundaries.
 */

ENTRY(_start)

SECTIONS
{
    . = 0x00200000;

    __kernel_start = .;

    .text : ALIGN(4K) {
        __text_start = .;
        *(.text.entry)
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : ALIGN(4K) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : ALIGN(4K) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4K);
    __kernel_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame)
    }
}

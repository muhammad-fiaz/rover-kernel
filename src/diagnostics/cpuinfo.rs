//! # CPU Diagnostics Reporting
//!
//! Generates detailed diagnostic logs of processor topology and features.

#[cfg(not(target_arch = "x86_64"))]
use crate::arch::current::CpuCapabilities;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::CpuCapabilities;

/// Emits detailed CPU feature diagnostics to the kernel log.
pub fn dump_cpu_diagnostics(caps: &CpuCapabilities) {
    crate::kinfo!("--- CPU Hardware Diagnostics ---");
    crate::kinfo!("Vendor String:    {}", caps.vendor_str());
    #[cfg(target_arch = "x86_64")]
    {
        crate::kinfo!(
            "Local APIC:       {}",
            if caps.has_apic { "Present" } else { "Missing" }
        );
        crate::kinfo!(
            "Long Mode (64-bit): {}",
            if caps.has_long_mode {
                "Supported"
            } else {
                "Unsupported"
            }
        );
        crate::kinfo!(
            "No-Execute (NX):  {}",
            if caps.has_nx {
                "Supported"
            } else {
                "Unsupported"
            }
        );
        crate::kinfo!(
            "1 GiB Huge Pages: {}",
            if caps.has_page_1gb {
                "Supported"
            } else {
                "Unsupported"
            }
        );
        crate::kinfo!("SIMD Capabilities:");
        crate::kinfo!("  SSE:   {}, SSE2:   {}", caps.has_sse, caps.has_sse2);
        crate::kinfo!(
            "  SSE3:  {}, SSE4.1: {}, SSE4.2: {}",
            caps.has_sse3,
            caps.has_sse4_1,
            caps.has_sse4_2
        );
        crate::kinfo!("  AVX:   {}, AVX2:   {}", caps.has_avx, caps.has_avx2);
    }
    crate::kinfo!("--------------------------------");
}

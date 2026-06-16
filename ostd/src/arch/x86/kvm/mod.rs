// SPDX-License-Identifier: MPL-2.0

//! x86 KVM hardware backend detection.

use crate::{arch::cpu::cpuid::cpuid, kvm::KvmAvailability};

mod vmx;

const CPUID_VENDOR_LEAF: u32 = 0;

pub(crate) fn availability() -> KvmAvailability {
    match cpu_vendor() {
        CpuVendor::Intel => vmx::availability(),
        CpuVendor::Amd | CpuVendor::Other => KvmAvailability::UnsupportedVendor,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CpuVendor {
    Intel,
    Amd,
    Other,
}

fn cpu_vendor() -> CpuVendor {
    let Some(result) = cpuid(CPUID_VENDOR_LEAF, 0) else {
        return CpuVendor::Other;
    };

    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&result.ebx.to_ne_bytes());
    vendor[4..8].copy_from_slice(&result.edx.to_ne_bytes());
    vendor[8..12].copy_from_slice(&result.ecx.to_ne_bytes());

    match &vendor {
        b"GenuineIntel" => CpuVendor::Intel,
        b"AuthenticAMD" => CpuVendor::Amd,
        _ => CpuVendor::Other,
    }
}

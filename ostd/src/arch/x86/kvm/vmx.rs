// SPDX-License-Identifier: MPL-2.0

//! Intel VMX availability detection.

use x86::msr::{IA32_FEATURE_CONTROL, rdmsr};

use crate::{arch::cpu::cpuid::cpuid, kvm::KvmAvailability};

const CPUID_FEATURE_LEAF: u32 = 1;
const CPUID_FEATURE_SUBLEAF: u32 = 0;
const CPUID_ECX_VMX_BIT: u32 = 5;

const FEATURE_CONTROL_LOCKED: u64 = 1 << 0;
const FEATURE_CONTROL_VMXON_OUTSIDE_SMX: u64 = 1 << 2;

pub(crate) fn availability() -> KvmAvailability {
    if !has_vmx_cpuid_bit() {
        return KvmAvailability::MissingVmx;
    }

    classify_feature_control(read_feature_control())
}

fn has_vmx_cpuid_bit() -> bool {
    let Some(result) = cpuid(CPUID_FEATURE_LEAF, CPUID_FEATURE_SUBLEAF) else {
        return false;
    };

    result.ecx & (1 << CPUID_ECX_VMX_BIT) != 0
}

fn read_feature_control() -> u64 {
    // SAFETY: Intel documents `IA32_FEATURE_CONTROL` as available when CPUID
    // reports VMX support, which is checked before this function is called.
    unsafe { rdmsr(IA32_FEATURE_CONTROL) }
}

fn classify_feature_control(feature_control: u64) -> KvmAvailability {
    let locked = feature_control & FEATURE_CONTROL_LOCKED != 0;
    let vmxon_outside_smx = feature_control & FEATURE_CONTROL_VMXON_OUTSIDE_SMX != 0;

    if locked && vmxon_outside_smx {
        KvmAvailability::Available
    } else {
        KvmAvailability::VmxDisabledByFirmware
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_control_must_be_locked_and_enable_vmxon_outside_smx() {
        assert_eq!(
            classify_feature_control(FEATURE_CONTROL_LOCKED | FEATURE_CONTROL_VMXON_OUTSIDE_SMX),
            KvmAvailability::Available
        );
        assert_eq!(
            classify_feature_control(FEATURE_CONTROL_LOCKED),
            KvmAvailability::VmxDisabledByFirmware
        );
        assert_eq!(
            classify_feature_control(FEATURE_CONTROL_VMXON_OUTSIDE_SMX),
            KvmAvailability::VmxDisabledByFirmware
        );
        assert_eq!(
            classify_feature_control(0),
            KvmAvailability::VmxDisabledByFirmware
        );
    }
}

// SPDX-License-Identifier: MPL-2.0

//! KVM hardware availability detection.

/// KVM hardware availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KvmAvailability {
    /// KVM can use the current hardware backend.
    Available,
    /// The architecture has no supported KVM backend.
    UnsupportedArch,
    /// The CPU vendor is not supported by the current KVM backend.
    UnsupportedVendor,
    /// The CPU does not report Intel VMX support.
    MissingVmx,
    /// Intel VMX is disabled by firmware.
    VmxDisabledByFirmware,
}

impl KvmAvailability {
    /// Returns whether KVM can use the current hardware backend.
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Returns KVM hardware availability.
pub fn availability() -> KvmAvailability {
    arch_availability()
}

#[cfg(target_arch = "x86_64")]
fn arch_availability() -> KvmAvailability {
    crate::arch::kvm::availability()
}

#[cfg(not(target_arch = "x86_64"))]
fn arch_availability() -> KvmAvailability {
    KvmAvailability::UnsupportedArch
}

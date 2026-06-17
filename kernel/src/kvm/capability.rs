// SPDX-License-Identifier: MPL-2.0

//! KVM capability reporting.

use super::{
    memory::KVM_NR_MEMSLOTS,
    uapi::{KVM_CAP_MAX_VCPUS, KVM_CAP_NR_MEMSLOTS, KVM_CAP_NR_VCPUS, KVM_CAP_USER_MEMORY},
    vcpu::KVM_MAX_VCPUS,
};

/// Returns the `KVM_CHECK_EXTENSION` value for a capability.
pub(crate) fn check_extension(raw_capability: usize) -> i32 {
    let Ok(capability) = i32::try_from(raw_capability) else {
        return 0;
    };

    match capability {
        KVM_CAP_USER_MEMORY => 1,
        KVM_CAP_NR_VCPUS | KVM_CAP_MAX_VCPUS => i32::try_from(KVM_MAX_VCPUS).unwrap_or(i32::MAX),
        KVM_CAP_NR_MEMSLOTS => i32::try_from(KVM_NR_MEMSLOTS).unwrap_or(i32::MAX),
        _ => 0,
    }
}

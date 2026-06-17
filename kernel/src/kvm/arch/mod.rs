// SPDX-License-Identifier: MPL-2.0

//! Architecture-specific KVM support.

use super::vcpu::KvmVcpu;
use crate::{prelude::*, util::ioctl::RawIoctl};

#[cfg(target_arch = "x86_64")]
pub(crate) mod x86;

#[cfg(target_arch = "x86_64")]
pub(crate) use x86::vcpu::KvmArchVcpu;

#[cfg(not(target_arch = "x86_64"))]
pub(crate) struct KvmArchVcpu;

#[cfg(not(target_arch = "x86_64"))]
impl KvmArchVcpu {
    /// Creates architecture-specific vCPU state.
    pub(crate) fn new() -> Self {
        Self
    }
}

/// Handles architecture-specific vCPU ioctls.
pub(crate) fn handle_vcpu_ioctl(vcpu: &KvmVcpu, raw_ioctl: RawIoctl) -> Result<i32> {
    #[cfg(target_arch = "x86_64")]
    {
        x86::vcpu::handle_ioctl(vcpu, raw_ioctl)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (vcpu, raw_ioctl);
        return_errno_with_message!(Errno::ENOTTY, "the ioctl command is unknown")
    }
}

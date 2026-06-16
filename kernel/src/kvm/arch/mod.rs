// SPDX-License-Identifier: MPL-2.0

//! Architecture-specific KVM support.

#[cfg(target_arch = "x86_64")]
pub(crate) mod x86;

// SPDX-License-Identifier: MPL-2.0

//! x86 KVM virtual CPU ioctl support.

use super::uapi::{KvmRegs, KvmSegment, KvmSregs, ioctl_defs};
use crate::{
    kvm::vcpu::KvmVcpu,
    prelude::*,
    util::ioctl::{RawIoctl, dispatch_ioctl},
};

const X86_CR0_PE: u64 = 1 << 0;
const X86_CR0_ET: u64 = 1 << 4;
const X86_CR0_PG: u64 = 1 << 31;
const X86_CR8_MAX: u64 = 15;
const X86_RFLAGS_FIXED: u64 = 1 << 1;
const X86_RESET_RIP: u64 = 0xfff0;

/// Architecture-specific vCPU state.
pub(crate) struct KvmArchVcpu {
    inner: Mutex<KvmArchVcpuInner>,
}

impl KvmArchVcpu {
    /// Creates x86 vCPU state.
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(KvmArchVcpuInner::new()),
        }
    }

    fn regs(&self) -> KvmRegs {
        self.inner.lock().regs
    }

    fn set_regs(&self, mut regs: KvmRegs) {
        regs.rflags |= X86_RFLAGS_FIXED;
        self.inner.lock().regs = regs;
    }

    fn sregs(&self) -> KvmSregs {
        self.inner.lock().sregs
    }

    fn set_sregs(&self, mut sregs: KvmSregs) -> Result<()> {
        validate_sregs(&sregs)?;
        sregs.cr0 |= X86_CR0_ET;
        self.inner.lock().sregs = sregs;
        Ok(())
    }
}

struct KvmArchVcpuInner {
    regs: KvmRegs,
    sregs: KvmSregs,
}

impl KvmArchVcpuInner {
    fn new() -> Self {
        Self {
            regs: default_regs(),
            sregs: default_sregs(),
        }
    }
}

/// Handles x86-specific vCPU ioctls.
pub(crate) fn handle_ioctl(vcpu: &KvmVcpu, raw_ioctl: RawIoctl) -> Result<i32> {
    use ioctl_defs::*;

    dispatch_ioctl!(match raw_ioctl {
        cmd @ GetRegs => {
            let regs = vcpu.arch().regs();
            cmd.write(&regs)?;
            Ok(0)
        }
        cmd @ SetRegs => {
            let regs = cmd.read()?;
            vcpu.arch().set_regs(regs);
            Ok(0)
        }
        cmd @ GetSregs => {
            let sregs = vcpu.arch().sregs();
            cmd.write(&sregs)?;
            Ok(0)
        }
        cmd @ SetSregs => {
            let sregs = cmd.read()?;
            vcpu.arch().set_sregs(sregs)?;
            Ok(0)
        }
        _cmd @ SetCpuid2 => {
            unsupported_vcpu_ioctl("KVM_SET_CPUID2")
        }
        _cmd @ GetCpuid2 => {
            unsupported_vcpu_ioctl("KVM_GET_CPUID2")
        }
        _ => return_errno_with_message!(Errno::ENOTTY, "the ioctl command is unknown"),
    })
}

fn unsupported_vcpu_ioctl(name: &str) -> Result<i32> {
    debug!("{} is not supported yet", name);
    return_errno_with_message!(Errno::ENOTTY, "the vCPU ioctl is not supported yet")
}

fn default_regs() -> KvmRegs {
    KvmRegs {
        rip: X86_RESET_RIP,
        rflags: X86_RFLAGS_FIXED,
        ..Default::default()
    }
}

fn default_sregs() -> KvmSregs {
    KvmSregs {
        cs: real_mode_code_segment(0xf000, 0xffff0000),
        ds: real_mode_data_segment(0, 0),
        es: real_mode_data_segment(0, 0),
        fs: real_mode_data_segment(0, 0),
        gs: real_mode_data_segment(0, 0),
        ss: real_mode_data_segment(0, 0),
        tr: unusable_segment(),
        ldt: unusable_segment(),
        cr0: X86_CR0_ET,
        ..Default::default()
    }
}

fn real_mode_code_segment(selector: u16, base: u64) -> KvmSegment {
    KvmSegment {
        base,
        limit: 0xffff,
        selector,
        type_: 0xb,
        present: 1,
        s: 1,
        ..Default::default()
    }
}

fn real_mode_data_segment(selector: u16, base: u64) -> KvmSegment {
    KvmSegment {
        base,
        limit: 0xffff,
        selector,
        type_: 0x3,
        present: 1,
        s: 1,
        ..Default::default()
    }
}

fn unusable_segment() -> KvmSegment {
    KvmSegment {
        unusable: 1,
        ..Default::default()
    }
}

fn validate_sregs(sregs: &KvmSregs) -> Result<()> {
    validate_segment(&sregs.cs)?;
    validate_segment(&sregs.ds)?;
    validate_segment(&sregs.es)?;
    validate_segment(&sregs.fs)?;
    validate_segment(&sregs.gs)?;
    validate_segment(&sregs.ss)?;
    validate_segment(&sregs.tr)?;
    validate_segment(&sregs.ldt)?;

    if sregs.cr8 > X86_CR8_MAX {
        return_errno_with_message!(Errno::EINVAL, "the CR8 value is invalid");
    }

    if sregs.cr0 & X86_CR0_PG != 0 && sregs.cr0 & X86_CR0_PE == 0 {
        return_errno_with_message!(Errno::EINVAL, "paging requires protected mode");
    }

    Ok(())
}

fn validate_segment(segment: &KvmSegment) -> Result<()> {
    validate_bit(segment.present)?;
    validate_bit(segment.db)?;
    validate_bit(segment.s)?;
    validate_bit(segment.l)?;
    validate_bit(segment.g)?;
    validate_bit(segment.avl)?;
    validate_bit(segment.unusable)?;

    if segment.dpl > 3 {
        return_errno_with_message!(Errno::EINVAL, "the segment DPL is invalid");
    }

    if segment.type_ > 0xf {
        return_errno_with_message!(Errno::EINVAL, "the segment type is invalid");
    }

    if segment.unusable == 0 && segment.l != 0 && segment.db != 0 {
        return_errno_with_message!(
            Errno::EINVAL,
            "a usable segment cannot be both 64-bit and default-operand-size"
        );
    }

    Ok(())
}

fn validate_bit(bit: u8) -> Result<()> {
    if bit > 1 {
        return_errno_with_message!(Errno::EINVAL, "the segment bit field is invalid");
    }

    Ok(())
}

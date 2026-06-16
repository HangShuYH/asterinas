// SPDX-License-Identifier: MPL-2.0

//! Linux x86 KVM userspace ABI definitions.

#![allow(unused)]

/// Checks whether `KVM_SET_TSS_ADDR` is supported.
pub(crate) const KVM_CAP_SET_TSS_ADDR: i32 = 4;

/// The CPUID entry has a significant index.
pub(crate) const KVM_CPUID_FLAG_SIGNIFCANT_INDEX: u32 = 1 << 0;
/// Alias for the misspelled Linux `KVM_CPUID_FLAG_SIGNIFCANT_INDEX` macro.
pub(crate) const KVM_CPUID_FLAG_SIGNIFICANT_INDEX: u32 = KVM_CPUID_FLAG_SIGNIFCANT_INDEX;
/// The CPUID entry describes a stateful function.
pub(crate) const KVM_CPUID_FLAG_STATEFUL_FUNC: u32 = 1 << 1;
/// The next read should use CPUID state.
pub(crate) const KVM_CPUID_FLAG_STATE_READ_NEXT: u32 = 1 << 2;

/// General-purpose registers; `struct kvm_regs` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmRegs {
    /// The `rax` register.
    pub rax: u64,
    /// The `rbx` register.
    pub rbx: u64,
    /// The `rcx` register.
    pub rcx: u64,
    /// The `rdx` register.
    pub rdx: u64,
    /// The `rsi` register.
    pub rsi: u64,
    /// The `rdi` register.
    pub rdi: u64,
    /// The `rsp` register.
    pub rsp: u64,
    /// The `rbp` register.
    pub rbp: u64,
    /// The `r8` register.
    pub r8: u64,
    /// The `r9` register.
    pub r9: u64,
    /// The `r10` register.
    pub r10: u64,
    /// The `r11` register.
    pub r11: u64,
    /// The `r12` register.
    pub r12: u64,
    /// The `r13` register.
    pub r13: u64,
    /// The `r14` register.
    pub r14: u64,
    /// The `r15` register.
    pub r15: u64,
    /// The instruction pointer.
    pub rip: u64,
    /// The flags register.
    pub rflags: u64,
}

/// A segment descriptor; `struct kvm_segment` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmSegment {
    /// The segment base address.
    pub base: u64,
    /// The segment limit.
    pub limit: u32,
    /// The segment selector.
    pub selector: u16,
    /// The descriptor type.
    pub type_: u8,
    /// Whether the segment is present.
    pub present: u8,
    /// The descriptor privilege level.
    pub dpl: u8,
    /// The default operand size bit.
    pub db: u8,
    /// The descriptor type bit.
    pub s: u8,
    /// The long mode bit.
    pub l: u8,
    /// The granularity bit.
    pub g: u8,
    /// The available bit.
    pub avl: u8,
    /// Whether the segment is unusable.
    pub unusable: u8,
    /// Padding reserved by Linux KVM.
    pub padding: u8,
}

/// A descriptor table; `struct kvm_dtable` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmDtable {
    /// The table base address.
    pub base: u64,
    /// The table limit.
    pub limit: u16,
    /// Padding reserved by Linux KVM.
    pub padding: [u16; 3],
}

/// Special registers; `struct kvm_sregs` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmSregs {
    /// The code segment.
    pub cs: KvmSegment,
    /// The data segment.
    pub ds: KvmSegment,
    /// The extra segment.
    pub es: KvmSegment,
    /// The FS segment.
    pub fs: KvmSegment,
    /// The GS segment.
    pub gs: KvmSegment,
    /// The stack segment.
    pub ss: KvmSegment,
    /// The task register.
    pub tr: KvmSegment,
    /// The local descriptor table register.
    pub ldt: KvmSegment,
    /// The global descriptor table register.
    pub gdt: KvmDtable,
    /// The interrupt descriptor table register.
    pub idt: KvmDtable,
    /// The `cr0` register.
    pub cr0: u64,
    /// The `cr2` register.
    pub cr2: u64,
    /// The `cr3` register.
    pub cr3: u64,
    /// The `cr4` register.
    pub cr4: u64,
    /// The `cr8` register.
    pub cr8: u64,
    /// The `efer` MSR.
    pub efer: u64,
    /// The local APIC base MSR.
    pub apic_base: u64,
    /// The interrupt bitmap.
    pub interrupt_bitmap: [u64; 4],
}

/// A CPUID entry; `struct kvm_cpuid_entry2` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmCpuidEntry2 {
    /// The CPUID function.
    pub function: u32,
    /// The CPUID index.
    pub index: u32,
    /// CPUID entry flags.
    pub flags: u32,
    /// The CPUID `eax` result.
    pub eax: u32,
    /// The CPUID `ebx` result.
    pub ebx: u32,
    /// The CPUID `ecx` result.
    pub ecx: u32,
    /// The CPUID `edx` result.
    pub edx: u32,
    /// Padding reserved by Linux KVM.
    pub padding: [u32; 3],
}

/// A variable-length CPUID array header; `struct kvm_cpuid2` in Linux x86 KVM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmCpuid2 {
    /// The number of CPUID entries that follow this header.
    pub nent: u32,
    /// Padding reserved by Linux KVM.
    pub padding: u32,
    /// The trailing CPUID entries.
    pub entries: [KvmCpuidEntry2; 0],
}

pub(crate) mod ioctl_defs {
    //! x86 KVM ioctl command definitions.

    use super::{KvmCpuid2, KvmRegs, KvmSregs};
    use crate::util::ioctl::{InData, InOutData, NoData, OutData, ioc};

    pub(crate) type GetSupportedCpuid =
        ioc!(KVM_GET_SUPPORTED_CPUID, 0xAE, 0x05, InOutData<KvmCpuid2>);
    pub(crate) type SetTssAddr = ioc!(KVM_SET_TSS_ADDR, 0xAE, 0x47, NoData);
    pub(crate) type GetRegs = ioc!(KVM_GET_REGS, 0xAE, 0x81, OutData<KvmRegs>);
    pub(crate) type SetRegs = ioc!(KVM_SET_REGS, 0xAE, 0x82, InData<KvmRegs>);
    pub(crate) type GetSregs = ioc!(KVM_GET_SREGS, 0xAE, 0x83, OutData<KvmSregs>);
    pub(crate) type SetSregs = ioc!(KVM_SET_SREGS, 0xAE, 0x84, InData<KvmSregs>);
    pub(crate) type SetCpuid2 = ioc!(KVM_SET_CPUID2, 0xAE, 0x90, InData<KvmCpuid2>);
    pub(crate) type GetCpuid2 = ioc!(KVM_GET_CPUID2, 0xAE, 0x91, InOutData<KvmCpuid2>);
}

const _: () = assert!(size_of::<KvmRegs>() == 144);
const _: () = assert!(size_of::<KvmSegment>() == 24);
const _: () = assert!(size_of::<KvmDtable>() == 16);
const _: () = assert!(size_of::<KvmSregs>() == 312);
const _: () = assert!(size_of::<KvmCpuidEntry2>() == 40);
const _: () = assert!(size_of::<KvmCpuid2>() == 8);

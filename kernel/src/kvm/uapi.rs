// SPDX-License-Identifier: MPL-2.0

//! Linux KVM userspace ABI definitions.

#![allow(unused)]

/// The Linux KVM API version implemented by Asterinas.
pub(crate) const KVM_API_VERSION: i32 = 12;

/// The ioctl magic used by Linux KVM.
pub(crate) const KVMIO: u8 = 0xAE;

/// Logs writes to memory pages.
pub(crate) const KVM_MEM_LOG_DIRTY_PAGES: u32 = 1 << 0;
/// Maps a userspace memory region as read-only guest memory.
pub(crate) const KVM_MEM_READONLY: u32 = 1 << 1;

/// Checks whether userspace memory regions are supported.
pub(crate) const KVM_CAP_USER_MEMORY: i32 = 3;
/// Returns the recommended maximum number of vCPUs per VM.
pub(crate) const KVM_CAP_NR_VCPUS: i32 = 9;
/// Returns the maximum number of memory slots per VM.
pub(crate) const KVM_CAP_NR_MEMSLOTS: i32 = 10;
/// Returns the maximum number of vCPUs per VM.
pub(crate) const KVM_CAP_MAX_VCPUS: i32 = 66;

/// The guest exit reason is unknown.
pub(crate) const KVM_EXIT_UNKNOWN: u32 = 0;
/// The guest exited due to an exception.
pub(crate) const KVM_EXIT_EXCEPTION: u32 = 1;
/// The guest exited due to port I/O.
pub(crate) const KVM_EXIT_IO: u32 = 2;
/// The guest executed `hlt`.
pub(crate) const KVM_EXIT_HLT: u32 = 5;
/// The guest exited due to MMIO.
pub(crate) const KVM_EXIT_MMIO: u32 = 6;
/// The guest shut down.
pub(crate) const KVM_EXIT_SHUTDOWN: u32 = 8;
/// VM entry failed.
pub(crate) const KVM_EXIT_FAIL_ENTRY: u32 = 9;
/// KVM encountered an internal error.
pub(crate) const KVM_EXIT_INTERNAL_ERROR: u32 = 17;

/// Port I/O reads from userspace into the guest.
pub(crate) const KVM_EXIT_IO_IN: u8 = 0;
/// Port I/O writes from the guest to userspace.
pub(crate) const KVM_EXIT_IO_OUT: u8 = 1;

/// The internal error was caused by emulation failure.
pub(crate) const KVM_INTERNAL_ERROR_EMULATION: u32 = 1;
/// The internal error was caused by unexpected simultaneous exceptions.
pub(crate) const KVM_INTERNAL_ERROR_SIMUL_EX: u32 = 2;
/// The internal error was caused by unexpected event delivery.
pub(crate) const KVM_INTERNAL_ERROR_DELIVERY_EV: u32 = 3;
/// The internal error was caused by an unexpected VM exit reason.
pub(crate) const KVM_INTERNAL_ERROR_UNEXPECTED_EXIT_REASON: u32 = 4;

/// A userspace memory region; `struct kvm_userspace_memory_region` in Linux.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
pub(crate) struct KvmUserspaceMemoryRegion {
    /// The memory slot id.
    pub slot: u32,
    /// Memory slot flags.
    pub flags: u32,
    /// The guest physical base address.
    pub guest_phys_addr: u64,
    /// The size of the memory region in bytes.
    pub memory_size: u64,
    /// The userspace virtual base address.
    pub userspace_addr: u64,
}

/// The `KVM_EXIT_UNKNOWN` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitUnknown {
    /// The hardware exit reason.
    pub hardware_exit_reason: u64,
}

/// The `KVM_EXIT_FAIL_ENTRY` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitFailEntry {
    /// The hardware entry failure reason.
    pub hardware_entry_failure_reason: u64,
    /// The CPU id associated with the failure.
    pub cpu: u32,
}

/// The `KVM_EXIT_EXCEPTION` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitException {
    /// The exception vector.
    pub exception: u32,
    /// The exception error code.
    pub error_code: u32,
}

/// The `KVM_EXIT_IO` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitIo {
    /// The I/O direction.
    pub direction: u8,
    /// The access size in bytes.
    pub size: u8,
    /// The I/O port.
    pub port: u16,
    /// The number of accesses.
    pub count: u32,
    /// The data offset relative to the start of `KvmRun`.
    pub data_offset: u64,
}

/// The `KVM_EXIT_MMIO` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitMmio {
    /// The guest physical address.
    pub phys_addr: u64,
    /// The MMIO data bytes.
    pub data: [u8; 8],
    /// The access length.
    pub len: u32,
    /// Whether this is a write.
    pub is_write: u8,
}

/// The `KVM_EXIT_INTERNAL_ERROR` payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KvmRunExitInternalError {
    /// The internal error subtype.
    pub suberror: u32,
    /// The number of valid data entries.
    pub ndata: u32,
    /// Internal error data.
    pub data: [u64; 16],
}

/// The exit payload union in `struct kvm_run`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union KvmRunExit {
    /// The `KVM_EXIT_UNKNOWN` payload.
    pub unknown: KvmRunExitUnknown,
    /// The `KVM_EXIT_FAIL_ENTRY` payload.
    pub fail_entry: KvmRunExitFailEntry,
    /// The `KVM_EXIT_EXCEPTION` payload.
    pub exception: KvmRunExitException,
    /// The `KVM_EXIT_IO` payload.
    pub io: KvmRunExitIo,
    /// The `KVM_EXIT_MMIO` payload.
    pub mmio: KvmRunExitMmio,
    /// The `KVM_EXIT_INTERNAL_ERROR` payload.
    pub internal: KvmRunExitInternalError,
    /// Padding that fixes the Linux ABI union size.
    pub padding: [u8; 256],
}

/// The vCPU run page; `struct kvm_run` in Linux.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct KvmRun {
    /// Requests an interrupt-window exit.
    pub request_interrupt_window: u8,
    /// Requests immediate exit before entering the guest.
    pub immediate_exit: u8,
    /// Padding reserved by Linux KVM.
    pub padding1: [u8; 6],
    /// The guest exit reason.
    pub exit_reason: u32,
    /// Whether the guest is ready for interrupt injection.
    pub ready_for_interrupt_injection: u8,
    /// The guest interrupt flag.
    pub if_flag: u8,
    /// Run flags.
    pub flags: u16,
    /// The task priority register shadow.
    pub cr8: u64,
    /// The local APIC base MSR.
    pub apic_base: u64,
    /// The exit payload.
    pub exit: KvmRunExit,
    /// The valid synchronized register bitmap.
    pub kvm_valid_regs: u64,
    /// The dirty synchronized register bitmap.
    pub kvm_dirty_regs: u64,
    /// The architecture-specific synchronized register area.
    pub sync_regs: [u8; 2048],
}

impl Default for KvmRun {
    fn default() -> Self {
        Self {
            request_interrupt_window: 0,
            immediate_exit: 0,
            padding1: [0; 6],
            exit_reason: 0,
            ready_for_interrupt_injection: 0,
            if_flag: 0,
            flags: 0,
            cr8: 0,
            apic_base: 0,
            exit: KvmRunExit { padding: [0; 256] },
            kvm_valid_regs: 0,
            kvm_dirty_regs: 0,
            sync_regs: [0; 2048],
        }
    }
}

pub(crate) mod ioctl_defs {
    //! KVM ioctl command definitions.

    use super::KvmUserspaceMemoryRegion;
    use crate::util::ioctl::{InData, NoData, ioc};

    pub(crate) type GetApiVersion = ioc!(KVM_GET_API_VERSION, 0xAE, 0x00, NoData);
    pub(crate) type CreateVm = ioc!(KVM_CREATE_VM, 0xAE, 0x01, NoData);
    pub(crate) type CheckExtension = ioc!(KVM_CHECK_EXTENSION, 0xAE, 0x03, NoData);
    pub(crate) type GetVcpuMmapSize = ioc!(KVM_GET_VCPU_MMAP_SIZE, 0xAE, 0x04, NoData);
    pub(crate) type SetUserMemoryRegion = ioc!(
        KVM_SET_USER_MEMORY_REGION,
        0xAE,
        0x46,
        InData<KvmUserspaceMemoryRegion>
    );
    pub(crate) type CreateVcpu = ioc!(KVM_CREATE_VCPU, 0xAE, 0x41, NoData);
    pub(crate) type Run = ioc!(KVM_RUN, 0xAE, 0x80, NoData);
}

const _: () = assert!(size_of::<KvmUserspaceMemoryRegion>() == 32);
const _: () = assert!(size_of::<KvmRunExit>() == 256);
const _: () = assert!(size_of::<KvmRun>() == 2352);

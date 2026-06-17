// SPDX-License-Identifier: MPL-2.0

//! KVM virtual CPU file support.

use alloc::format;
use core::fmt::Display;

use super::{uapi::KVM_VCPU_MMAP_SIZE, vm::KvmVm};
use crate::{
    events::IoEvents,
    fs::{
        file::{AccessMode, CreationFlags, FileLike, InodeHandle, Mappable, file_table::FdFlags},
        ramfs::memfd::{MemfdFlags, MemfdInodeHandle},
        vfs::path::Path,
    },
    prelude::*,
    process::signal::{PollHandle, Pollable},
};

/// The maximum number of vCPUs supported by one VM.
pub(crate) const KVM_MAX_VCPUS: u32 = 1;

/// A vCPU file handle created by `KVM_CREATE_VCPU`.
pub(crate) struct KvmVcpuFile {
    id: u32,
    #[expect(dead_code, reason = "keeps VM state alive for the vCPU fd lifetime")]
    vm: Arc<KvmVm>,
    run_file: InodeHandle,
}

impl KvmVcpuFile {
    /// Creates a KVM vCPU file handle.
    pub(crate) fn new(id: u32, vm: Arc<KvmVm>) -> Result<Self> {
        // The generic mmap path expects the mapped VMO to be the page cache of
        // the file path. A private memfd gives the vCPU run page such a backing
        // without changing the shared VMAR/VFS code.
        let run_file =
            InodeHandle::new_memfd(format!("kvm-vcpu:{}", id), MemfdFlags::MFD_NOEXEC_SEAL)?;
        run_file.resize(KVM_VCPU_MMAP_SIZE as usize)?;

        Ok(Self { id, vm, run_file })
    }
}

impl Pollable for KvmVcpuFile {
    fn poll(&self, mask: IoEvents, _poller: Option<&mut PollHandle>) -> IoEvents {
        mask & (IoEvents::IN | IoEvents::OUT)
    }
}

impl FileLike for KvmVcpuFile {
    fn access_mode(&self) -> AccessMode {
        AccessMode::O_RDWR
    }

    fn mappable(&self) -> Result<Mappable> {
        self.run_file.mappable()
    }

    fn path(&self) -> &Path {
        self.run_file.path()
    }

    fn dump_proc_fdinfo(self: Arc<Self>, fd_flags: FdFlags) -> Box<dyn Display> {
        struct FdInfo {
            flags: u32,
            id: u32,
            path: Path,
        }

        impl Display for FdInfo {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                writeln!(f, "pos:\t{}", 0)?;
                writeln!(f, "flags:\t0{:o}", self.flags)?;
                writeln!(f, "mnt_id:\t{}", self.path.mount_node().id())?;
                writeln!(f, "ino:\t{}", self.path.inode().ino())?;
                writeln!(f, "vcpu_id:\t{}", self.id)
            }
        }

        let mut flags = self.status_flags().bits() | self.access_mode() as u32;
        if fd_flags.contains(FdFlags::CLOEXEC) {
            flags |= CreationFlags::O_CLOEXEC.bits();
        }

        Box::new(FdInfo {
            flags,
            id: self.id,
            path: self.path().clone(),
        })
    }
}

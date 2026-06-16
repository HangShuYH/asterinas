// SPDX-License-Identifier: MPL-2.0

//! KVM virtual machine file support.

use core::fmt::Display;

use crate::{
    events::IoEvents,
    fs::{
        file::{AccessMode, CreationFlags, FileLike, file_table::FdFlags},
        pseudofs::AnonInodeFs,
        vfs::path::Path,
    },
    prelude::*,
    process::signal::{PollHandle, Pollable},
};

/// A VM file handle created by `KVM_CREATE_VM`.
pub(crate) struct KvmVmFile {
    pseudo_path: Path,
}

impl KvmVmFile {
    /// Creates a KVM VM file handle.
    pub(crate) fn new() -> Self {
        let pseudo_path = AnonInodeFs::new_path(|_| "anon_inode:[kvm-vm]".to_string());

        Self { pseudo_path }
    }
}

impl Pollable for KvmVmFile {
    fn poll(&self, _mask: IoEvents, _poller: Option<&mut PollHandle>) -> IoEvents {
        IoEvents::empty()
    }
}

impl FileLike for KvmVmFile {
    fn access_mode(&self) -> AccessMode {
        AccessMode::O_RDWR
    }

    fn path(&self) -> &Path {
        &self.pseudo_path
    }

    fn dump_proc_fdinfo(self: Arc<Self>, fd_flags: FdFlags) -> Box<dyn Display> {
        struct FdInfo {
            flags: u32,
        }

        impl Display for FdInfo {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                writeln!(f, "pos:\t{}", 0)?;
                writeln!(f, "flags:\t0{:o}", self.flags)?;
                writeln!(f, "mnt_id:\t{}", AnonInodeFs::mount_node().id())?;
                writeln!(f, "ino:\t{}", AnonInodeFs::shared_inode().ino())
            }
        }

        let mut flags = self.status_flags().bits() | self.access_mode() as u32;
        if fd_flags.contains(FdFlags::CLOEXEC) {
            flags |= CreationFlags::O_CLOEXEC.bits();
        }

        Box::new(FdInfo { flags })
    }
}

// SPDX-License-Identifier: MPL-2.0

//! KVM virtual machine file support.

use core::fmt::Display;

use ostd::task::Task;

use super::{
    memory::MemorySlotTable,
    uapi::ioctl_defs,
    vcpu::{KVM_MAX_VCPUS, KvmVcpuFile},
};
use crate::{
    events::IoEvents,
    fs::{
        file::{AccessMode, CreationFlags, FileLike, file_table::FdFlags},
        pseudofs::AnonInodeFs,
        vfs::path::Path,
    },
    prelude::*,
    process::signal::{PollHandle, Pollable},
    util::ioctl::{RawIoctl, dispatch_ioctl},
};

/// A VM file handle created by `KVM_CREATE_VM`.
pub(crate) struct KvmVmFile {
    vm: Arc<KvmVm>,
    pseudo_path: Path,
}

impl KvmVmFile {
    /// Creates a KVM VM file handle.
    pub(crate) fn new() -> Self {
        let pseudo_path = AnonInodeFs::new_path(|_| "anon_inode:[kvm-vm]".to_string());

        Self {
            vm: Arc::new(KvmVm::new()),
            pseudo_path,
        }
    }
}

/// A KVM virtual machine object shared by VM and vCPU file handles.
pub(crate) struct KvmVm {
    inner: Mutex<KvmVmInner>,
}

impl KvmVm {
    fn new() -> Self {
        Self {
            inner: Mutex::new(KvmVmInner::new()),
        }
    }

    fn set_user_memory_region(&self, region: super::uapi::KvmUserspaceMemoryRegion) -> Result<()> {
        self.inner
            .lock()
            .memory_slots
            .set_user_memory_region(region)
    }

    fn register_vcpu_id(&self, id: u32) -> Result<()> {
        if id >= KVM_MAX_VCPUS {
            return_errno_with_message!(Errno::EINVAL, "the vCPU id is out of range");
        }

        let mut inner = self.inner.lock();
        if !inner.vcpu_ids.insert(id) {
            return_errno_with_message!(Errno::EINVAL, "the vCPU id already exists");
        }

        Ok(())
    }
}

struct KvmVmInner {
    memory_slots: MemorySlotTable,
    vcpu_ids: BTreeSet<u32>,
}

impl KvmVmInner {
    fn new() -> Self {
        Self {
            memory_slots: MemorySlotTable::new(),
            vcpu_ids: BTreeSet::new(),
        }
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

    fn ioctl(&self, raw_ioctl: RawIoctl) -> Result<i32> {
        use ioctl_defs::*;

        dispatch_ioctl!(match raw_ioctl {
            cmd @ SetUserMemoryRegion => {
                let region = cmd.read()?;
                self.vm.set_user_memory_region(region)?;
                Ok(0)
            }
            _cmd @ CreateVcpu => {
                let id = u32::try_from(raw_ioctl.arg())
                    .map_err(|_| Error::with_message(Errno::EINVAL, "the vCPU id is too large"))?;
                let vcpu_file = Arc::new(KvmVcpuFile::new(id, self.vm.clone())?);
                self.vm.register_vcpu_id(id)?;

                let current_task = Task::current().unwrap();
                let thread_local = current_task.as_thread_local().unwrap();
                let fd = {
                    let file_table = thread_local.borrow_file_table();
                    let mut file_table_locked = file_table.unwrap().write();
                    file_table_locked.insert(vcpu_file, FdFlags::empty())
                };

                Ok(fd.into())
            }
            _ => return_errno_with_message!(Errno::ENOTTY, "the ioctl command is unknown"),
        })
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

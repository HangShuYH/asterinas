// SPDX-License-Identifier: MPL-2.0

//! Hardware KVM misc-device support.
//!
//! This module registers the `/dev/kvm` device.

use device_id::{DeviceId, MinorId};
use ostd::task::Task;

use crate::{
    device::{Device, DeviceType, DevtmpfsInodeMeta, registry::char},
    events::IoEvents,
    fs::{
        file::{PerOpenFileOps, StatusFlags, file_table::FdFlags},
        vfs::inode::FileOps,
    },
    kvm::{
        capability::check_extension,
        uapi::{KVM_API_VERSION, KVM_VCPU_MMAP_SIZE, ioctl_defs},
        vm::KvmVmFile,
    },
    prelude::*,
    process::signal::{PollHandle, Pollable},
    util::ioctl::{RawIoctl, dispatch_ioctl},
};

const KVM_MINOR: u32 = 232;

/// The '/dev/kvm' device.
#[derive(Debug)]
struct KvmDevice {
    id: DeviceId,
}

impl KvmDevice {
    fn new() -> Arc<Self> {
        let major = super::MISC_MAJOR.get().unwrap().get();
        let minor = MinorId::new(KVM_MINOR);

        let id = DeviceId::new(major, minor);
        Arc::new(Self { id })
    }
}

impl Device for KvmDevice {
    fn type_(&self) -> DeviceType {
        DeviceType::Char
    }

    fn id(&self) -> DeviceId {
        self.id
    }

    fn devtmpfs_meta(&self) -> Option<DevtmpfsInodeMeta<'_>> {
        Some(DevtmpfsInodeMeta::new("kvm"))
    }

    fn open(&self) -> Result<Box<dyn PerOpenFileOps>> {
        Ok(Box::new(KvmFile))
    }
}

/// A file handle opened from `/dev/kvm`.
struct KvmFile;

impl Pollable for KvmFile {
    fn poll(&self, mask: IoEvents, _poller: Option<&mut PollHandle>) -> IoEvents {
        mask & (IoEvents::IN | IoEvents::OUT)
    }
}

impl FileOps for KvmFile {
    fn read_at(
        &self,
        _offset: usize,
        _writer: &mut VmWriter,
        _status_flags: StatusFlags,
    ) -> Result<usize> {
        return_errno_with_message!(Errno::EINVAL, "the KVM device does not support reading");
    }

    fn write_at(
        &self,
        _offset: usize,
        _reader: &mut VmReader,
        _status_flags: StatusFlags,
    ) -> Result<usize> {
        return_errno_with_message!(Errno::EINVAL, "the KVM device does not support writing");
    }
}

impl PerOpenFileOps for KvmFile {
    fn check_seekable(&self) -> Result<()> {
        return_errno_with_message!(Errno::ESPIPE, "seek is not supported")
    }

    fn is_offset_aware(&self) -> bool {
        false
    }

    fn ioctl(&self, raw_ioctl: RawIoctl) -> Result<i32> {
        use ioctl_defs::*;

        dispatch_ioctl!(match raw_ioctl {
            _cmd @ GetApiVersion => {
                Ok(KVM_API_VERSION)
            }
            _cmd @ CreateVm => {
                if raw_ioctl.arg() != 0 {
                    return_errno_with_message!(Errno::EINVAL, "the VM type is not supported");
                }

                let vm_file = Arc::new(KvmVmFile::new());
                let current_task = Task::current().unwrap();
                let thread_local = current_task.as_thread_local().unwrap();
                let fd = {
                    let file_table = thread_local.borrow_file_table();
                    let mut file_table_locked = file_table.unwrap().write();
                    file_table_locked.insert(vm_file, FdFlags::empty())
                };
                Ok(fd.into())
            }
            _cmd @ CheckExtension => {
                Ok(check_extension(raw_ioctl.arg()))
            }
            _cmd @ GetVcpuMmapSize => {
                Ok(KVM_VCPU_MMAP_SIZE)
            }
            _ => return_errno_with_message!(Errno::ENOTTY, "the ioctl command is unknown"),
        })
    }
}

pub(super) fn init_in_first_kthread() {
    let availability = ostd::kvm::availability();
    if !availability.is_available() {
        warn!("not registering /dev/kvm: {:?}", availability);
        return;
    }

    char::register(KvmDevice::new()).unwrap();
    info!("registered /dev/kvm");
}

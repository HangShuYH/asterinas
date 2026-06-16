// SPDX-License-Identifier: MPL-2.0

//! Hardware KVM misc-device support.
//!
//! This module registers the `/dev/kvm` device.

use device_id::{DeviceId, MinorId};

use crate::{
    device::{Device, DeviceType, DevtmpfsInodeMeta, registry::char},
    events::IoEvents,
    fs::{
        file::{PerOpenFileOps, StatusFlags},
        vfs::inode::FileOps,
    },
    prelude::*,
    process::signal::{PollHandle, Pollable},
    util::ioctl::RawIoctl,
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
        let _ = raw_ioctl;
        return_errno_with_message!(Errno::ENOTTY, "the ioctl command is unknown");
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

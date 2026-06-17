// SPDX-License-Identifier: MPL-2.0

//! KVM guest memory slot management.

use core::ops::Range;

use super::uapi::KvmUserspaceMemoryRegion;
use crate::{
    prelude::*,
    vm::vmar::{VMAR_CAP_ADDR, VMAR_LOWEST_ADDR},
};

/// The maximum number of memory slots supported by one VM.
pub(crate) const KVM_NR_MEMSLOTS: usize = 32;

/// A validated guest memory slot.
#[derive(Clone, Debug)]
struct MemorySlot {
    id: usize,
    guest_range: Range<u64>,
    #[expect(dead_code, reason = "the hardware mapping backend will consume it")]
    userspace_addr: u64,
}

impl MemorySlot {
    fn new(id: usize, region: &KvmUserspaceMemoryRegion) -> Result<Self> {
        validate_region_alignment(region)?;

        let guest_end = region
            .guest_phys_addr
            .checked_add(region.memory_size)
            .ok_or_else(|| {
                Error::with_message(Errno::EINVAL, "the guest memory range overflows")
            })?;
        let userspace_end = region
            .userspace_addr
            .checked_add(region.memory_size)
            .ok_or_else(|| {
                Error::with_message(Errno::EINVAL, "the userspace memory range overflows")
            })?;

        validate_userspace_range(region.userspace_addr, userspace_end)?;

        Ok(Self {
            id,
            guest_range: region.guest_phys_addr..guest_end,
            userspace_addr: region.userspace_addr,
        })
    }
}

/// A table of guest memory slots.
#[derive(Debug)]
pub(crate) struct MemorySlotTable {
    slots: Vec<Option<MemorySlot>>,
}

impl MemorySlotTable {
    /// Creates a memory slot table.
    pub(crate) fn new() -> Self {
        Self {
            slots: vec![None; KVM_NR_MEMSLOTS],
        }
    }

    /// Sets or deletes a guest memory slot.
    pub(crate) fn set_user_memory_region(
        &mut self,
        region: KvmUserspaceMemoryRegion,
    ) -> Result<()> {
        let slot_id = validate_slot_id(region.slot)?;

        if region.flags != 0 {
            return_errno_with_message!(Errno::EINVAL, "memory slot flags are not supported");
        }

        if region.memory_size == 0 {
            validate_region_alignment(&region)?;
            self.slots[slot_id] = None;
            return Ok(());
        }

        let new_slot = MemorySlot::new(slot_id, &region)?;
        self.validate_no_guest_overlap(&new_slot)?;
        self.slots[slot_id] = Some(new_slot);

        Ok(())
    }

    fn validate_no_guest_overlap(&self, new_slot: &MemorySlot) -> Result<()> {
        for slot in self
            .slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|slot| slot.id != new_slot.id)
        {
            if ranges_overlap(&slot.guest_range, &new_slot.guest_range) {
                return_errno_with_message!(Errno::EINVAL, "guest memory slots overlap");
            }
        }

        Ok(())
    }
}

impl Default for MemorySlotTable {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_slot_id(slot_id: u32) -> Result<usize> {
    if slot_id as usize >= KVM_NR_MEMSLOTS {
        return_errno_with_message!(Errno::EINVAL, "the memory slot id is out of range");
    }

    Ok(slot_id as usize)
}

fn validate_region_alignment(region: &KvmUserspaceMemoryRegion) -> Result<()> {
    let page_size = PAGE_SIZE as u64;

    if !region.guest_phys_addr.is_multiple_of(page_size) {
        return_errno_with_message!(Errno::EINVAL, "the guest physical address is not aligned");
    }

    if !region.userspace_addr.is_multiple_of(page_size) {
        return_errno_with_message!(Errno::EINVAL, "the userspace address is not aligned");
    }

    if !region.memory_size.is_multiple_of(page_size) {
        return_errno_with_message!(Errno::EINVAL, "the memory size is not aligned");
    }

    Ok(())
}

fn validate_userspace_range(userspace_addr: u64, userspace_end: u64) -> Result<()> {
    let start = usize::try_from(userspace_addr)
        .map_err(|_| Error::with_message(Errno::EINVAL, "the userspace address is too large"))?;
    let end = usize::try_from(userspace_end).map_err(|_| {
        Error::with_message(Errno::EINVAL, "the userspace end address is too large")
    })?;

    if start < VMAR_LOWEST_ADDR || end > VMAR_CAP_ADDR {
        return_errno_with_message!(Errno::EINVAL, "the userspace memory range is invalid");
    }

    Ok(())
}

fn ranges_overlap(first: &Range<u64>, second: &Range<u64>) -> bool {
    first.start < second.end && second.start < first.end
}

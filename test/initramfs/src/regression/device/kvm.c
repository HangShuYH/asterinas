// SPDX-License-Identifier: MPL-2.0

/*
 * Regression tests for `/dev/kvm`.
 */

#include <errno.h>
#include <fcntl.h>
#include <linux/kvm.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

#include "../common/test.h"

#define KVM_DEVICE "/dev/kvm"

static void exit_if_kvm_is_unavailable(void);

FN_SETUP(check_kvm_availability)
{
	exit_if_kvm_is_unavailable();
}
END_SETUP()

FN_TEST(kvm_get_api_version)
{
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));

	TEST_RES(ioctl(fd, KVM_GET_API_VERSION), _ret == KVM_API_VERSION);

	TEST_SUCC(close(fd));
}
END_TEST()

FN_TEST(kvm_check_extension)
{
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));

	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_USER_MEMORY),
		 _ret == 1);
	int nr_vcpus = TEST_RES(
		ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_NR_VCPUS), _ret > 0);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_NR_MEMSLOTS), _ret > 0);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_MAX_VCPUS),
		 _ret >= nr_vcpus);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, -1), _ret == 0);

	TEST_SUCC(close(fd));
}
END_TEST()

FN_TEST(kvm_get_vcpu_mmap_size)
{
	long page_size = TEST_RES(sysconf(_SC_PAGESIZE), _ret > 0);
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));

	int mmap_size =
		TEST_RES(ioctl(fd, KVM_GET_VCPU_MMAP_SIZE),
			 _ret >= 0 && (size_t)_ret >= sizeof(struct kvm_run));
	TEST_RES(mmap_size % page_size, _ret == 0);

	TEST_SUCC(close(fd));
}
END_TEST()

FN_TEST(kvm_create_vm)
{
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));

	int vm_fd = TEST_SUCC(ioctl(fd, KVM_CREATE_VM, 0));
	TEST_ERRNO(ioctl(vm_fd, KVM_GET_API_VERSION), ENOTTY);
	TEST_SUCC(close(vm_fd));

	TEST_ERRNO(ioctl(fd, KVM_CREATE_VM, 1), EINVAL);

	TEST_SUCC(close(fd));
}
END_TEST()

FN_TEST(kvm_set_user_memory_region)
{
	long page_size = TEST_RES(sysconf(_SC_PAGESIZE), _ret > 0);
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));
	int vm_fd = TEST_SUCC(ioctl(fd, KVM_CREATE_VM, 0));
	char *mem = TEST_SUCC(mmap(NULL, page_size * 3, PROT_READ | PROT_WRITE,
				   MAP_ANONYMOUS | MAP_PRIVATE, -1, 0));
	struct kvm_userspace_memory_region region = {
		.slot = 0,
		.flags = 0,
		.guest_phys_addr = 0,
		.memory_size = page_size,
		.userspace_addr = (uintptr_t)mem,
	};

	TEST_SUCC(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region));

	region.slot = 1;
	region.userspace_addr = (uintptr_t)(mem + page_size);
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.guest_phys_addr = page_size;
	TEST_SUCC(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region));

	region.slot = 2;
	region.guest_phys_addr = 2 * page_size;
	region.userspace_addr = (uintptr_t)(mem + 1);
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.userspace_addr = (uintptr_t)(mem + 2 * page_size);
	region.memory_size = page_size - 1;
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.guest_phys_addr = 2 * page_size + 1;
	region.memory_size = page_size;
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.guest_phys_addr = 2 * page_size;
	region.flags = KVM_MEM_LOG_DIRTY_PAGES;
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.flags = 0;
	region.slot = 1000000;
	TEST_ERRNO(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region), EINVAL);

	region.slot = 0;
	region.guest_phys_addr = 0;
	region.memory_size = 0;
	region.userspace_addr = (uintptr_t)mem;
	TEST_SUCC(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region));

	region.slot = 1;
	region.guest_phys_addr = page_size;
	region.userspace_addr = (uintptr_t)(mem + page_size);
	TEST_SUCC(ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region));

	TEST_SUCC(munmap(mem, page_size * 3));
	TEST_SUCC(close(vm_fd));
	TEST_SUCC(close(fd));
}
END_TEST()

FN_TEST(kvm_create_vcpu)
{
	int fd = TEST_SUCC(open(KVM_DEVICE, O_RDWR));
	int vm_fd = TEST_SUCC(ioctl(fd, KVM_CREATE_VM, 0));
	int mmap_size = TEST_SUCC(ioctl(fd, KVM_GET_VCPU_MMAP_SIZE));

	int vcpu_fd = TEST_SUCC(ioctl(vm_fd, KVM_CREATE_VCPU, 0));
	TEST_ERRNO(ioctl(vm_fd, KVM_CREATE_VCPU, 0), EINVAL);
	TEST_ERRNO(ioctl(vm_fd, KVM_CREATE_VCPU, 1), EINVAL);
	TEST_ERRNO(ioctl(vm_fd, KVM_RUN, 0), ENOTTY);
	TEST_ERRNO(ioctl(vcpu_fd, KVM_CREATE_VCPU, 0), ENOTTY);
	TEST_ERRNO(ioctl(vcpu_fd, KVM_RUN, 0), ENOTTY);
	TEST_ERRNO(ioctl(vcpu_fd, 0, 0), ENOTTY);
#ifdef KVM_GET_REGS
	struct kvm_regs regs;
	memset(&regs, 0, sizeof(regs));
	TEST_ERRNO(ioctl(vm_fd, KVM_GET_REGS, &regs), ENOTTY);
	TEST_SUCC(ioctl(vcpu_fd, KVM_GET_REGS, &regs));
	TEST_RES(regs.rflags & 0x2, _ret == 0x2);
	TEST_RES(regs.rip, _ret == 0xfff0);

#ifdef KVM_SET_REGS
	regs.rax = 0x12345678;
	regs.rbx = 0x87654321;
	regs.rip = 0x100000;
	regs.rsp = 0x200000;
	regs.rflags = 0x202;
	TEST_SUCC(ioctl(vcpu_fd, KVM_SET_REGS, &regs));

	struct kvm_regs readback;
	memset(&readback, 0, sizeof(readback));
	TEST_SUCC(ioctl(vcpu_fd, KVM_GET_REGS, &readback));
	TEST_RES(readback.rax, _ret == regs.rax);
	TEST_RES(readback.rbx, _ret == regs.rbx);
	TEST_RES(readback.rip, _ret == regs.rip);
	TEST_RES(readback.rsp, _ret == regs.rsp);
	TEST_RES(readback.rflags, _ret == regs.rflags);
#endif
#endif

#ifdef KVM_GET_SREGS
	struct kvm_sregs sregs;
	memset(&sregs, 0, sizeof(sregs));
	TEST_ERRNO(ioctl(vm_fd, KVM_GET_SREGS, &sregs), ENOTTY);
	TEST_SUCC(ioctl(vcpu_fd, KVM_GET_SREGS, &sregs));
	TEST_RES(sregs.cs.selector, _ret == 0xf000);
	TEST_RES(sregs.cs.base, _ret == 0xffff0000);
	TEST_RES(sregs.cr0 & 0x10, _ret == 0x10);

#ifdef KVM_SET_SREGS
	sregs.cs.selector = 0x8;
	sregs.cs.base = 0;
	sregs.ds.selector = 0x10;
	sregs.ds.base = 0;
	sregs.cr3 = 0x3000;
	sregs.cr4 = 0x20;
	sregs.efer = 0x500;
	sregs.gdt.base = 0x1000;
	sregs.gdt.limit = 0x30;
	sregs.idt.base = 0x2000;
	sregs.idt.limit = 0x40;
	TEST_SUCC(ioctl(vcpu_fd, KVM_SET_SREGS, &sregs));

	struct kvm_sregs sregs_readback;
	memset(&sregs_readback, 0, sizeof(sregs_readback));
	TEST_SUCC(ioctl(vcpu_fd, KVM_GET_SREGS, &sregs_readback));
	TEST_RES(sregs_readback.cs.selector, _ret == sregs.cs.selector);
	TEST_RES(sregs_readback.cs.base, _ret == sregs.cs.base);
	TEST_RES(sregs_readback.ds.selector, _ret == sregs.ds.selector);
	TEST_RES(sregs_readback.cr3, _ret == sregs.cr3);
	TEST_RES(sregs_readback.cr4, _ret == sregs.cr4);
	TEST_RES(sregs_readback.efer, _ret == sregs.efer);
	TEST_RES(sregs_readback.gdt.base, _ret == sregs.gdt.base);
	TEST_RES(sregs_readback.idt.base, _ret == sregs.idt.base);

	sregs.cr8 = 16;
	TEST_ERRNO(ioctl(vcpu_fd, KVM_SET_SREGS, &sregs), EINVAL);
#endif
#endif

	struct kvm_run *run =
		TEST_SUCC(mmap(NULL, mmap_size, PROT_READ | PROT_WRITE,
			       MAP_SHARED, vcpu_fd, 0));
	TEST_RES(run->exit_reason, _ret == 0);
	run->immediate_exit = 1;
	TEST_RES(run->immediate_exit, _ret == 1);

	TEST_SUCC(munmap(run, mmap_size));
	TEST_SUCC(close(vcpu_fd));
	TEST_SUCC(close(vm_fd));
	TEST_SUCC(close(fd));
}
END_TEST()

static void exit_if_kvm_is_unavailable(void)
{
	int fd = open(KVM_DEVICE, O_RDWR);

	if (fd >= 0) {
		CHECK(close(fd));
		return;
	}

	if (errno == ENOENT || errno == ENODEV || errno == ENXIO) {
		fprintf(stderr, "kvm tests skipped: %s (%s)\n", KVM_DEVICE,
			strerror(errno));
		exit(EXIT_SUCCESS);
	}

	fprintf(stderr, "fatal error: %s: open('%s') failed: %s\n", __func__,
		KVM_DEVICE, strerror(errno));
	exit(EXIT_FAILURE);
}

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
		 _ret == 0);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_NR_VCPUS), _ret == 0);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_NR_MEMSLOTS),
		 _ret == 0);
	TEST_RES(ioctl(fd, KVM_CHECK_EXTENSION, KVM_CAP_MAX_VCPUS), _ret == 0);
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
	TEST_ERRNO(ioctl(vcpu_fd, KVM_CREATE_VCPU, 0), ENOTTY);

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

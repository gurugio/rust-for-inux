# enable CONFIG_*_9P_*, CONFIG_VIRTIO_PCI and CONFIG_PCI
qemu-system-x86_64 -smp 4 -kernel ./rust-for-linux/vmlinux \
-initrd ./initramfs/initramfs.cpio.gz \
-virtfs local,path=./rust-for-linux/samples/rust,mount_tag=host0,security_model=mapped,id=host0  \
-nographic -append "console=ttyS0 rdinit=/init" -enable-kvm

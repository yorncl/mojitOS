set -e

# Clean up the state


cleanup () {
	DEV=$(losetup | grep disk.img | cut -d ' ' -f 1)
	if [ -z $DEV ]; then
		return
	fi
	sudo losetup -d $DEV
	sudo umount ./mnt
	rm -f disk.img
}

# initial clean up (I'm paranoid)
cleanup

qemu-img create -f raw disk.img 40M

# builidng the disk structure
fdisk disk.img <<EOF
n
p
1
2048
40960
a

n
p
2
43008
81919

w
EOF

# mounting shenanigans
sudo losetup -fP disk.img
DEV=$(losetup | grep disk.img | cut -d ' ' -f 1)
if [ -z $DEV ]; then
	echo "Failed to create loop device !"
	exit 1
fi
P1=$DEV"p1"
P2=$DEV"p2"
echo "Device mounted on $DEV, parts are $P1 and $P2"

# creating filesystems
sudo mkfs.vfat $P1
sudo mkfs.ext2 $P2

# mounting, installing grub and copying config files/data files
# boot partitions
sudo mount $P1 ./mnt
sudo grub-install --target=i386-pc --boot-directory=./mnt/ $DEV
sudo cp ./disk/boot/grub/grub.cfg ./mnt/grub/
sudo cp -r ./mojitos.elf ./mnt
sudo umount ./mnt
# root filesystem partition
sudo mount $P2 ./mnt
sudo cp -r ./disk/{etc,home} ./mnt
sudo umount ./mnt

# cleaning up
cleanup

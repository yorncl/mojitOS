lodev=$(losetup | grep disk.img | head -n 1 | awk '{print $1}')
sudo umount ./mnt
sudo losetup -d $lodev

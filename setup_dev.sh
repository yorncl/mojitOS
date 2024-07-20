file=disk.img

sudo losetup -f -P $file
lodev=$(losetup | grep disk.img | head -n 1 | awk '{print $1}')
sudo mount -t vfat "$lodev"p1 ./mnt -o uid=$UID,gid=$UID

file=disk.img
sudo losetup -f -P $file
lodev=$(losetup | grep disk.img | head -n 1 | awk '{print $1}')
sudo mount "$lodev"p1 ./mnt

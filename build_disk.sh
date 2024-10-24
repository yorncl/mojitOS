
file=disk.img

rm -f $file
dd if=/dev/zero of=disk.img bs=512 count=131072

fdisk $file <<EOF
n
p
1


a
w
EOF


sudo losetup /dev/loop20 $file
sudo losetup /dev/loop21 $file -o 1048576

sudo mke2fs /dev/loop21

sudo mount /dev/loop21 ./mnt

sudo mkdir -p ./mnt/boot/grub
sudo cp ./i386-pc ./mnt
sudo cp ./iso/boot/grub/grub.cfg ./mnt/boot/grub

sudo grub-install --boot-directory=./mnt/boot  /dev/loop20

sudo losetup -d /dev/loop20
sudo losetup -d /dev/loop21
sudo umount ./mnt

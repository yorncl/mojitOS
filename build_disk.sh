set -e

rm -f disk.img

guestfish -N disk.img=disk:30M <<EOF

part-init /dev/sda mbr
part-add /dev/sda primary 2048 40960
part-add /dev/sda primary 40961 -40
part-set-bootable /dev/sda 1 true

mkfs vfat /dev/sda1
mkfs ext2 /dev/sda2

mount /dev/sda2 /
mkdir /boot
mkdir /home
mkdir /home/yrn
mount /dev/sda1 /boot
grub-install / /dev/sda

copy-in mojitos.elf /boot
copy-in iso/boot/grub/grub.cfg /boot/grub
EOF

# if one day I get crazy and want to install grub2 manually
#part-set-name /dev/sda 1 'EFI System'
#part-set-gpt-type /dev/sda 1 c12a7328-f81f-11d2-ba4b-00a0c93ec93b





QEMU=qemu-system-i386
CARGO=cargo

NAME=mojitos.elf
ISO=mojitos.iso
DISK_IMG=disk.img
AS=nasm
# TODO do a check for DEBUG=1 in command line
ASFLAGS=-f elf32 -g

KLIB=libmojitos.a

OBJDIR=target/obj # TODO not very clean
ASM_SOURCES=$(shell find src/ -type f -name '*.S')
ASM_OBJECTS=$(patsubst src/%.S, target/obj/%.o, $(ASM_SOURCES))

LD=ld
LD_FLAGS=-n -nostdlib -m elf_i386
LINK_SCRIPT=linker/x86.ld

all: $(NAME)

$(NAME): $(KLIB) asm link

# iso: $(NAME)
# 	cp mojitos.elf iso/boot/mojitos.elf
# 	grub-mkrescue -o $(ISO) iso

$(DISKIMG): 
	./build_disk.sh

update_mnt: all
	mkdir -p mnt
	cp $(NAME) mnt/$(NAME)
	#I got issues where the file wasn't immediately runnable by grub when copied in the vfat mounted folder
	sync
	

$(KLIB):
	$(CARGO) build --features debug_serial  # TODO debug ?

target/obj/%.o: src/%.S
	mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

objdir:
	mkdir -p $(OBJDIR)
asm : objdir $(ASM_OBJECTS)

link:$(LINK_SCRIPT)
	$(LD) $(LD_FLAGS) -T $(LINK_SCRIPT) -o $(NAME) $(ASM_OBJECTS) target/x86/debug/$(KLIB)

clean:
	$(CARGO) clean
	rm -f $(NAME)
	rm -f $(ASM_OBJECTS)

# run_iso: iso # TODO for some reason using the $(ISO) rule as dependency requires to run this twice for the iso to be rebuilt
# 	$(QEMU) -cdrom $(ISO) -no-reboot

run: update_mnt $(DISKIMG)
	$(QEMU) -drive format=raw,file=$(DISK_IMG),if=none,id=disk1 -device ide-hd,drive=disk1 -serial stdio -no-reboot


klib_test:
	$(CARGO) test --no-run

# test: klib_test asm link
# 	cp mojitos.elf iso/boot/mojitos.elf
# 	grub-mkrescue -o $(ISO) iso
# 	$(QEMU) -cdrom $(ISO) -no-reboot

debug: $(NAME)
	$(QEMU) -drive format=raw,file=$(DISK_IMG),if=none,id=disk1 -device ide-hd,drive=disk1 -s -S -no-reboot -d int,cpu_reset
	# $(QEMU) -kernel $(NAME) -s -S -no-reboot -d int,cpu_reset

.PHONE: all clean run debug link asm

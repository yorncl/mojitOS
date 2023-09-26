

QEMU=qemu-system-i386
CARGO=cargo

NAME=mojitos.elf
AS=nasm
ASFLAGS=-f elf32

KLIB=libmojitos.a

OBJDIR=target/obj # TODO not very clean
ASM_SOURCES=$(shell find src/ -type f -name '*.S')
ASM_OBJECTS=$(patsubst src/%.S, target/obj/%.o, $(ASM_SOURCES))

all: $(NAME)

$(NAME): $(KLIB) asm link

$(KLIB):
	$(CARGO) build  # TODO debug ?

target/obj/%.o: src/%.S
	mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

objdir:
	mkdir -p $(OBJDIR)
asm : objdir $(ASM_OBJECTS)

link:
	ld -n -nostdlib -m elf_i386 -T linker/x86.ld -o $(NAME) $(ASM_OBJECTS) target/x86/debug/$(KLIB)

clean:
	$(CARGO) clean
	rm -f $(NAME)
	rm -f $(ASM_OBJECTS)

run: $(NAME)
	$(QEMU) -kernel $(NAME)

debug: $(NAME)
	$(QEMU) -kernel $(NAME) -s -S -no-reboot -d int,cpu_reset

.PHONE: all clean run debug link asm

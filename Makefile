

QEMU=qemu-system-i386
CARGO=cargo

NAME=mojitos.elf
AS=nasm

KLIB=libmojitos.a


all: $(NAME)

$(NAME): $(KLIB) asm link
	# cp target/my_target/debug/$(KLIB) $(NAME)

$(KLIB):
	$(CARGO) build  # TODO debug ?

asm:
	$(AS) -f elf32 src/arch/x86//bootstrap.S  -o target/bootstrap.o
	$(AS) -f elf32 src/arch//x86/reload_segments.S  -o target/reload_segments.o

link:
	ld -n -nostdlib -m elf_i386 -T linker/x86.ld -o $(NAME) target/bootstrap.o target/reload_segments.o target/my_target/debug/$(KLIB)

clean:
	$(CARGO) clean
	rm -f $(NAME)

run: $(NAME)
	$(QEMU) -kernel $(NAME)

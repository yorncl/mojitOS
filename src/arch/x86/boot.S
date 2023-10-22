
global loader
global stack_ptr
global start

extern kmain

MODULEALIGN equ 1<<0
MEMINFO equ 1<<1
FLAGS equ MODULEALIGN | MEMINFO
MAGIC equ 0x1BADB002
CHECKSUM equ -(MAGIC + FLAGS)

section .multiboot
align 4
MultiBootHeader:
  dd MAGIC
  dd FLAGS
  dd CHECKSUM

section .text

STACKSIZE equ 0x4000

start:
  mov esp, stack+STACKSIZE

  push ebx ; multiboot2 info structure pointer
  push eax ; multiboot2 magic number
  call kmain

  cli

hang:
  hlt
  jmp hang



section .bss
align 4
stack:
  resb STACKSIZE
stack_ptr:

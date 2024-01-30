
global loader
global stack_ptr
global start

extern kstart

MODULEALIGN equ 1<<0
MEMINFO equ 1<<1
FLAGS equ MODULEALIGN | MEMINFO
MAGIC equ 0x1BADB002
CHECKSUM equ -(MAGIC + FLAGS)

STACKSIZE equ 0x4000

section .multiboot
align 4
MultiBootHeader:
  dd MAGIC
  dd FLAGS
  dd CHECKSUM

section .text


start:
  mov esp, stack+STACKSIZE

  push ebx ; multiboot info structure pointer
  push eax ; multiboot magic number
  call kstart

  cli

hang:
  hlt
  jmp hang



section .bss
align 4
stack:
  resb STACKSIZE
stack_ptr:

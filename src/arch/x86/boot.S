
global loader
global stack_ptr
global kernel_bootstrap

extern kstart
extern setup_early_paging

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

HIGH_MAPPING_START equ 0xC0000000

kernel_bootstrap:
  mov esp, (stack+STACKSIZE - HIGH_MAPPING_START) ; TODO are we clear on this ? Am I sure it doesn't break ?
 
  call setup_early_paging

  push eax
  lea eax, [higher_half]
  jmp eax

  higher_half:
  add esp, HIGH_MAPPING_START
  add ebp, HIGH_MAPPING_START
  pop eax


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


global loader
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
  ; Setting up the stack, offset symbol to get physical address
  mov esp, (stack+STACKSIZE - HIGH_MAPPING_START) ; TODO are we clear on this ? Am I sure it doesn't break ?
  mov ebp, esp
 
  ; Jump to rust code to setup paging
  pushad
  call setup_early_paging
  popad

  ; Once the early linear mapping is done, jump to higher half
  ; the symbol "higher_half" is addressed in 0xC.. space
  ; we make a long jump to force a non-relative one
  push eax
  lea eax, [higher_half]
  jmp eax

  higher_half:
  ; Readjust the stack to be in higher half
  add esp, HIGH_MAPPING_START
  add ebp, HIGH_MAPPING_START
  pop eax

  push ebx ; multiboot info structure pointer
  push eax ; multiboot magic number

  ; here we go into rust code
  call kstart

  cli

hang:
  hlt
  jmp hang


section .bss
align 4
stack:
  resb STACKSIZE

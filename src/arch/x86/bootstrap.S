
global loader
global stack_ptr

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
  push eax
  push ebx

  call kmain

  cli

hang:
  hlt
  jmp hang



extern keystroke_handler
global keyboard_interrupt
keyboard_interrupt:
push eax    ;; make sure you don't damage current state
in al,60h   ;; read information from the keyboard
; call to custom function to check

; pass arguments
push eax ; TODO don't really understand why that works but it does
call keystroke_handler
pop eax

xor eax, eax
mov al,20h
out 20h,al  ;; acknowledge the interrupt to the PIC
pop eax     ;; restore state
iret        ;; return to code executed before.

section .bss
align 4
stack:
  resb STACKSIZE
stack_ptr:

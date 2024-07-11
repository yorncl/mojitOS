extern generic_handler

%macro interrupt_handler 1
interrupt_wrapper_%1:
  pushad
  push %1
  call generic_handler
  pop eax
  popad
  iret
%endmacro

%assign i 0
%rep 256
interrupt_handler i
%assign i i+1
%endrep

extern fill_idt_entry
%macro fill_entry 1
  mov ebx, interrupt_wrapper_%1
  push ebx
  mov eax, %1
  push eax
  call fill_idt_entry
  pop eax
  pop ebx
%endmacro

global fill_idt
fill_idt:
  push eax
  push ebx

  mov ebx, 0
  ; This macro will be repeated 256 times
  %assign i 0
  %rep 256
  fill_entry i
  %assign i i+1
  %endrep

  pop ebx
  pop eax
  ret

; interrupt_wrapper:
; pushad
; push eax    ;; make sure you don't damage current state
; in al,60h   ;; read information from the keyboard
; ; call to custom function to check

; ; pass arguments
; push eax ; TODO don't really understand why that works but it does
; call generic_handler
; pop eax

; pop eax     ;; restore state
; popad
; iret        ;; return to code executed before.


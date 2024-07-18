extern generic_handler
extern exception_handler

extern CONTEXT_CHANGE
extern NEED_SCHED

%macro interrupt_handler 1
interrupt_wrapper_%1:

  ; General purpose registers
  pushad

  push %1 ; interrupt number
  call generic_handler
  pop ebx ; remove interrupt number

  popad
  iret
%endmacro

%macro exception_handler 1
exception_wrapper_%1:

  ; General purpose registers
  push %1 ; interrupt number
  call exception_handler
  pop ebx ; remove interrupt number

  iret
%endmacro

%assign i 0
%rep 32
exception_handler i
%assign i i+1
%endrep

%assign i 32
%rep 224
interrupt_handler i
%assign i i+1
%endrep

extern fill_idt_entry

%macro fill_entry_int 1
  mov ebx, interrupt_wrapper_%1
  push ebx
  mov eax, %1
  push eax
  call fill_idt_entry
  pop eax
  pop ebx
%endmacro

%macro fill_entry_exception 1
  mov ebx, exception_wrapper_%1
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
  %rep 32
  fill_entry_exception i
  %assign i i+1
  %endrep

  %assign i 32
  %rep 224
  fill_entry_int i
  %assign i i+1
  %endrep

  pop ebx
  pop eax
  ret


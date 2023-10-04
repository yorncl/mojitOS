
extern generic_handler
global interrupt_wrapper
interrupt_wrapper:
pushad
cld
push eax    ;; make sure you don't damage current state
in al,60h   ;; read information from the keyboard
; call to custom function to check

; pass arguments
push eax ; TODO don't really understand why that works but it does
call generic_handler
pop eax

xor eax, eax
mov al,20h
out 20h,al  ;; acknowledge the interrupt to the PIC
pop eax     ;; restore state
popad
iret        ;; return to code executed before.


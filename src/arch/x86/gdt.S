section .text

global load_gdt
global reload_segments  

gdtr DW 0 ; For limit storage
     DD 0 ; For base storage
 
load_gdt:
   MOV   AX, [esp + 4]
   MOV   [gdtr], AX
   MOV   EAX, [ESP + 8]
   MOV   [gdtr + 2], EAX
   LGDT  [gdtr]
   RET

reload_segments:
        jmp   0x8:reload_CS
reload_CS:
        mov   AX, 0x10
        mov   DS, AX
        mov   ES, AX
        mov   FS, AX
        mov   GS, AX
        mov   SS, AX
        ret
    

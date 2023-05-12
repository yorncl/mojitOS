.globl reload_segments
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
    

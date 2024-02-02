; The goal here is to map the first 4MB of the physical memory in two places in the virtual space
; one identity mapped, and one mapped in the higher half
; this code is one of the first things to run
; the kernel symbols, being linked as if it were loaded physically in the higher half, but it's not
; thus in order for all the adress to be valid, we must first create the double mapping and enable paging

global setup_early_paging

HIGH_MAPPING_START equ 0xC0000000
PDE_INDEX equ (HIGH_MAPPING_START >> 22) ; only the 10 leftmost bits are used to index into the PDE


align 4096
EARLY_PAGE_TABLE:
  times 1024 dd 0

align 4096
EARLY_PAGE_DIRECTORY:
  times 1024 dd 0

section .text

EPT_PHYS equ (EARLY_PAGE_TABLE - HIGH_MAPPING_START)
EPD_PHYS equ (EARLY_PAGE_DIRECTORY - HIGH_MAPPING_START)

setup_early_paging:

  push eax
  push ebx
  push ecx

  mov eax, 0x0
  mov ebx, 0x0 ; base physical address
  fill_table:
      mov ecx, ebx
      or ecx, 3
      mov [EPT_PHYS+eax*4], ecx
      add ebx, 4096 ; increment 4KB everytime
      inc eax
      cmp eax, 1024
      jne fill_table
 

  mov eax, EPT_PHYS
  or eax, 0x83
  mov [EPD_PHYS], eax
  mov [EPD_PHYS + PDE_INDEX * 4], eax

  mov eax, EPD_PHYS
  mov cr3, eax

  mov eax, cr0
  or eax, 0x80000000 ; paging bit
  mov cr0, eax
  

  pop ecx
  pop ebx
  pop eax

  ret



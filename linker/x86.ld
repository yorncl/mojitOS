ENTRY(start)
OUTPUT(elf32-i386)

SECTIONS
{
    /* /1* . = 0x1000; *1/ */
    /* . : { */
		/* KEEP(*(.multiboot)) */
	/* } */
    . = 0x00100000;
  .text : ALIGN(CONSTANT(MAXPAGESIZE)) {
	  . = ALIGN(8);
	  KEEP(*(.multiboot))
	_TEXT_START_ = .;
	    *(.text)
	    *(.text.*)
	_TEXT_END_ = .;
  }

  .data : ALIGN(CONSTANT(MAXPAGESIZE)) {
	_DATA_START_ = .;
	    *(.data)
	    *(.data.*)
	_DATA_END_ = .;
  }

  .rodata : ALIGN(CONSTANT(MAXPAGESIZE)) {
	_DATA_START_ = .;
	    *(.rodata)
	    *(.rodata.*)
	_DATA_END_ = .;
  }

  .bss : ALIGN(CONSTANT(MAXPAGESIZE)) {
	_BSS_START_ = .;
	    *(.bss)
	_BSS_END_ = .;
  }
}

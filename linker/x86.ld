ENTRY(kernel_bootstrap)
OUTPUT(elf32-i386)

SECTIONS
{
    . = 0x00100000;

    .multiboot : {
	  KEEP(*(.multiboot))
    }

    . += 0xC0000000;
    kernel_image_start = .;

  .text ALIGN(4K) : AT (ADDR (.text) - 0xC0000000) {
  /* .text ALIGN(4K) : { */
	  . = ALIGN(8);
	_TEXT_START_ = .;
	    *(.text)
	    *(.text.*)
	_TEXT_END_ = .;
  }

  .data ALIGN(4K) : AT (ADDR (.data) - 0xC0000000)  {
  /* .data ALIGN(4K) : { */
	_DATA_START_ = .;
	    *(.data)
	    *(.data.*)
	_DATA_END_ = .;
  }

  .rodata ALIGN(4K) : AT (ADDR (.rodata) - 0xC0000000)  {
  /* .rodata ALIGN(4K) : { */
	_DATA_START_ = .;
	    *(.rodata)
	    *(.rodata.*)
	_DATA_END_ = .;
  }

  .bss ALIGN(4K) : AT (ADDR (.bss) - 0xC0000000)  {
  /* .bss ALIGN(4K) : { */
	_BSS_START_ = .;
	    *(.bss.*)
	_BSS_END_ = .;
  }
  kernel_image_end = .;
}

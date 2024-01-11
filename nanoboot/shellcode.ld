ENTRY(_start);

SECTIONS
{
    . = 0x08000000;
	. = ALIGN(16);
	.text :
	{
		*(.text.prologue)
		*(.text)
		*(.rodata)
	}
	.data :
	{
		*(.data)
	}

	/DISCARD/ :
	{
		*(.interp)
		*(.comment)
		*(.debug_frame)
	}
}
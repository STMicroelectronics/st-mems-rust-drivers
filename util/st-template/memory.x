/* Linker script for STM32 microcontrollers */
MEMORY
{
  /* Main Flash memory */
  FLASH : ORIGIN = 0x08000000, LENGTH = {{flash_size_kb}}K

  /* Main RAM (includes SRAM, and on F7/H7 includes DTCM in the total) */
  RAM : ORIGIN = 0x20000000, LENGTH = {{ram_kb}}K

  {% if ccm_kb != 0 -%}
  /* Core Coupled Memory (CCM) - F3/F4 series */
  CCRAM : ORIGIN = 0x10000000, LENGTH = {{ccm_kb}}K
  {% endif -%}
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* Define heap start (optional, adjust as needed) */
_heap_start = ORIGIN(RAM);
_heap_end = ORIGIN(RAM) + LENGTH(RAM);

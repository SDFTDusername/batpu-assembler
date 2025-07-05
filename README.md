# batpu-assembler
batpu-assembler is a [Rust](https://www.rust-lang.org/) program and library for assembling assembly for the [BatPU](https://github.com/mattbatwings/BatPU-2).

To assemble something, you can put in the assembly file, and then the output file, like this:
``batpu-assembler program.asm program.mc``

There are other arguments you can use:

```
-d, --disable-default-defines - Disables built-in defines, such as SCR_PIX_X
-p, --no-print-info           - Do not print assembler info
-t, --text-output             - Assemble to text file with binary representation
```

## Built-in defines
- ``SCR_PIX_X         (240) - Screen Pixel X``
- ``SCR_PIX_Y         (241) - Screen Pixel Y``
- ``SCR_DRAW_PIX      (242) - Draw pixel``
- ``SCR_CLR_PIX       (243) - Clear pixel``
- ``SCR_GET_PIX       (244) - Get pixel``
- ``SCR_DRAW          (245) - Push screen buffer``
- ``SCR_CLR           (246) - Clear screen buffer``
- ``CHAR_DISP_WRITE   (247) - Write character``
- ``CHAR_DISP_PUSH    (248) - Push character buffer``
- ``CHAR_DISP_CLR     (249) - Clear character buffer``
- ``NUM_DISP_SHOW     (250) - Show number``
- ``NUM_DISP_CLR      (251) - Clear number``
- ``NUM_DISP_SIGNED   (252) - Set to signed mode``
- ``NUM_DISP_UNSIGNED (253) - Set to unsigned mode``
- ``RNG               (254) - Random number generator``
- ``CONTROLLER        (255) - Controller input``

## Assembly code example
```
#define MEM_ADDR r1
#define MEM_VAL r2

jmp +3 // Jump 3 instructions ahead, so to "jmp main"

main:
  ldi MEM_ADDR 0x00; ldi MEM_VAL 0xFF
  str MEM_ADDR MEM_VAL 0

  jmp main // "jmp -2" is also possible
```

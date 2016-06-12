# chipster
Yet another Chip8 implementation.

![Screenshot - F8z](screenshot_f8z.png)

The project is still in early stage, because:

- Not all instructions currently implemented,
- Several of those implemented probably have not been
 tested thoroughly, some other may suffer from quirks (8xy6, 8xyE, Fx55, Fx65),
- Hardcoded controls,  
- Tested only on macOS.

Some plans (in random order):

- Assembler/Disassembler,
- Debugger, or even better time-travel debbuger, 
- Try to figure out and implement canonical behavior of quirked instructions, or
 at least implement "Quirk-switches",
- Support different backends for window (like ncurses) and audio,
- Configurable controls, limited gamepad mappings,
- Extensive compatibility test suite, that can be used by other chip8 makers,
- Support Windows

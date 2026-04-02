<div align="center">

# CHRONOS

### The Programming Language of Discipline

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-Alpha-red.svg)]()

**Explicit. Ceremonial. Absolute.**

A programming language where nothing is implicit, every operation is a ritual,
and discipline is enforced by design.

[Getting Started](#getting-started) •
[Language Guide](#language-overview) •
[Examples](#examples) •
[Architecture](#architecture) •
[Contributing](#contributing)

---
firstly, it contains pretty much turkish texts inside.

</div>

## Why CHRONOS?

Most modern languages strive for brevity and convenience. CHRONOS takes the opposite path.

| Philosophy | Meaning |
|---|---|
| **Explicit over Implicit** | Every type, every annotation, every operation must be written out |
| **Ceremony is Discipline** | Writing code is a ritual — shortcuts breed errors |
| **Zero Tolerance** | Every error must be handled. No exceptions. No ignoring. |
| **Type Absolutism** | No type inference. You declare it, or it doesn't compile. |


#![module::entry(main)]
@require core::io::{ StreamWriter, BufferMode };
@require core::types::{ String, ExitCode };

contract Main :: EntryPoint {

    @static
    @throws(IOError, RuntimeException)
    fn main(args: Vector<String>) -> ExitCode {

        let writer: StreamWriter = StreamWriter::acquire(
            target: StdOut,
            mode: BufferMode::LineBuffered,
            encoding: Encoding::UTF8
        );

        writer.emit(
            payload: "Hello, World!",
            terminate: LineEnding::LF
        );

        writer.release();
        return ExitCode::Success(0x00);
    }
}

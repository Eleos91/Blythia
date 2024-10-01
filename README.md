# Welcome!

Blythia is a compiler for a python dialect written in Rust.
This interpretation of Python will be statically typed and compiled.
At the moment, there are no expectations for this project.
It's just a way to learn Rust for me and flex to other people...

The only compile target atm is nasm-elf64 aseembly.
To enjoy your first blythia linux x86_64 binary,
please install:
- rust
- cargo
- nasm

then write your first unimpressive python code and run:
`cargo run com -r <path/to/file.py>`
and immediately report the bug that i don't know about yet but you'll surely have.

## Features
At this time (01-10-2024), the language supports the following features:

- Golbal variables
- Variable shadowing
- u64 integers
- add, min, mult ,div, equal, grater and lesser operations
- function definitions with params in the System V x86_64 style (but no return values yet)
- a build in function that can print one u64 value/variable
- while loop
- if and an optional else

It's not much tbh but that shit was hard enough for me to pull off...

# Next Target
- Implement types (u64 and f64 for now) in a proper manner.
- Variables must be typed at declaration
- System V x86_64 module to prperly handle ints and floats

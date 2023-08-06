# kestrel
![rustc 1.71.0](https://img.shields.io/badge/rustc-1.71.0-red)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)
![Test status](https://github.com/EricLBuehler/kestrel/actions/workflows/tests.yml/badge.svg)
![Build status](https://github.com/EricLBuehler/kestrel/actions/workflows/build.yml/badge.svg)
![Docs status](https://github.com/EricLBuehler/kestrel/actions/workflows/docs.yml/badge.svg)

Simple and safe.

## Todo Roadmap

- **Add more primitive `std` types** (by 8/7/23)

    Add: `bool`, `i8`, `i16`, `i64`, `i128`, `u8`, `u16`, `u64`, `u128` types.
    
    Special notes for `{i|u}{8-128}`: Add match clause for the new types in existing `i32` implementation.

    Special notes for `bool`: none

- **Add comparison traits** (by 8/9/23)

    Add: `Eq`, `Ne` traits, implement for `std` types.

- **Add control flow** (by 8/13/23)

    Add: `if` keyword, `elif` keyword, `else` keyword, use phi values.

    MIR effect: Will require permutations to prove invariants.
    
- **Add C-style enums** (by 8/15/23)

    Add: `enum` keyword, enum variant instantiation with `::` operator.

- **Add `match` keyword** (by 8/17/23)

    Add: `match` keyword, use phi values.
    
    MIR effect: Will require permutations to prove invariants.

## Features
- **Ahead of time compilation** - Kestrel is compiled ahead of time (AOT), instead of being interpreted or JIT compiled. AOT compilation allows Kestrel to catch entire classes of runtime errors, vastly improving the developer experience.

- **Statically typed** - Kestrel resolves types at compile time, resulting in immediate warnings and feedback.

- **Performance** - AOT compilation means that Kestrel programs are compiled directly to machine code, allowing programs to be executed on any target platform natively, with blazing fast performance.

- **Helpful compiler** - Descriptive and detailed error messages improve the debugging experience.

- **Borrow checker** - Ensures that invariants which prevent many types of memory errors by converting Kestrel code into Kestrel MIR.
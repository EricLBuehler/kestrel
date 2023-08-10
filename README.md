# kestrel
![rustc 1.71.0](https://img.shields.io/badge/rustc-1.71.0-red)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)
![Test status](https://github.com/EricLBuehler/kestrel/actions/workflows/tests.yml/badge.svg)
![Build status](https://github.com/EricLBuehler/kestrel/actions/workflows/build.yml/badge.svg)
![Docs status](https://github.com/EricLBuehler/kestrel/actions/workflows/docs.yml/badge.svg)

Simple and safe.

## Todo Roadmap

- **Add functions** (by 8/10/23)

    Add: 
    - [x] `fn` keyword
    - [x] Use `main` for entry point
    - [x] `return` keyword
    - [x] function calls

    MIR effect:
    - [ ] Will require permutations to prove invariants (guaranteed return for now).

- **Add comparison traits** (by 8/12/23)

    Add: `Eq`, `Ne` traits, implement for `std` types.

- **Add control flow** (by 8/16/23)

    Add: `if` keyword, `elif` keyword, `else` keyword, use phi values.

    MIR effect: Will require permutations to prove invariants.
    
- **Add C-style enums** (by 8/18/23)

    Add: `enum` keyword, enum variant instantiation with `::` operator.

- **Add `match` keyword** (by 8/21/23)

    Add: `match` keyword, use phi values.
    
    MIR effect: Will require permutations to prove invariants.
    
- **Add Rust-style enums** (by 8/24/23)

    Add: `enum` instantiation really instantiates a struct containing the value and the discriminant, update `match keyword`

- **Add `String` type** (by 8/26/23)

    Add: `String` type.

- **Add `std::io::print` type** (by 8/28/23)

    Add: `std::io::print` builtin function.

- **Allow `fn` to take references** (by 9/2/23)

    Update: `fn` parameter list.
    
    Mir effect: Will require lifetime checks for functions.


## Features
- **Ahead of time compilation** - Kestrel is compiled ahead of time (AOT), instead of being interpreted or JIT compiled. AOT compilation allows Kestrel to catch entire classes of runtime errors, vastly improving the developer experience.

- **Statically typed** - Kestrel resolves types at compile time, resulting in immediate warnings and feedback.

- **Performance** - AOT compilation means that Kestrel programs are compiled directly to machine code, allowing programs to be executed on any target platform natively, with blazing fast performance.

- **Helpful compiler** - Descriptive and detailed error messages improve the debugging experience.

- **Borrow checker** - Ensures that invariants which prevent many types of memory errors by converting Kestrel code into Kestrel MIR.
# kestrel
![rustc 1.71.0](https://img.shields.io/badge/rustc-1.71.0-red)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)
![Test status](https://github.com/EricLBuehler/kestrel/actions/workflows/tests.yml/badge.svg)
![Build status](https://github.com/EricLBuehler/kestrel/actions/workflows/build.yml/badge.svg)
![Docs status](https://github.com/EricLBuehler/kestrel/actions/workflows/docs.yml/badge.svg)

Simple and safe.
## Features
- **Ahead of time compilation** - Kestrel is compiled ahead of time (AOT), instead of being interpreted or JIT compiled. AOT compilation allows Kestrel to catch entire classes of runtime errors, vastly improving the developer experience.

- **Statically typed** - Kestrel resolves types at compile time, resulting in immediate warnings and feedback.

- **Performance** - AOT compilation means that Kestrel programs are compiled directly to machine code, allowing programs to be executed on any target platform natively, with blazing fast performance.

- **Helpful compiler** - Descriptive and detailed error messages improve the debugging experience.

- **Borrow checker** - Ensures that invariants which prevent many types of memory errors by converting Kestrel code into Kestrel MIR.
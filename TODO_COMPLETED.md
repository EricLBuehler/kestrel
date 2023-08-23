# Todo roadmap items that have been completed

- **Add more primitive `std` types** (by 8/7/23)

    Add: `bool`, `i8`, `i16`, `i64`, `i128`, `u8`, `u16`, `u64`, `u128` types.
    
    Special notes for `{i|u}{8-128}`: Add match clause for the new types in existing `i32` implementation.

    Special notes for `bool`: none

- **Add functions** (by 8/10/23)

    Add: 
    - [x] `fn` keyword
    - [x] Use `main` for entry point
    - [x] `return` keyword
    - [x] function calls

    MIR effect:
    - [x] Will require permutations to prove invariants (guaranteed return for now).
    
- **Add comparison traits** (by 8/12/23)

    Add: 
    - [x] `Eq`, `Ne` traits, implement for `std` types.
    - [x] Add `==`, `!=` operators.
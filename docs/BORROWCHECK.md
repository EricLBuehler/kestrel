# Kestrel Borrow Checker
The Kestrel borrow checker ensures that several invariants are maintained:

- Each value has one owner
- There may only be one reference to any value at one time (this is currently a contrived demonstrative example until `&mut ref`)
- References may not be returned from functions (this is currently a contrived demonstrative example until actual lifetime checks)

For example, this code is valid:
```
fn main() {
    let 😀🤠 = 1+2
    let mut 😐 = &😀🤠
    😐 = &100
    let 😐😐 = &😀🤠
}
```

This also works becuase of to the fact that the references to `😀🤠` on line 4 are essentially the same reference - and not separate. If they were separate this would cause a compilation error.
```
fn main() {
    let 😀🤠 = 1+2
    let mut 😐 = &😀🤠
    😐 = &100
    let 😐😐 = &&😀🤠
}
```

The borrow checker runs before code generation. It works by converting the program into MIR (Mid Intermediate Representation), which is a simplified form of Kestrel that is boiled down to a set of simple instructions akin to assembly. These enable the borrow checker to analyze the program from a simpler view. See the .mir output files.

## MIR generation process
1) MIR generation creates MIR with without any lifetime metadata.
2) The lifetime generation pass adds lifetime metadata to the MIR by analying the MIR.
3) The borrow checker itself runs. This ensures the reference invariants.

# Task breakdown for step 2
- Check for ownership invariants.
- Ensure references are not returned (**only for soundness currently**).

# Task breakdown for step 3
- Ensure single-reference invariants (**contrived limitation**).

## Drop order
Bindings are dropped not when they go out of scope, but when they are last used. This is reflected in the outputted .mir file.
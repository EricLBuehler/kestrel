# Kestrel Borrow Checker
The Kestrel borrow checker ensures that several invariants are maintained:

- Each value has one owner
- There may only be one reference to any binding at one time (this is currently a contrived demonstrative example)

For example, this code is valid:
```
let 😀🤠 = 1+2
let mut 😐 = &😀🤠
😐 = &100
let 😐😐 = &😀🤠
```

This also works becuase of to the fact that the references to `😀🤠` on line 4 are essentially the same reference - and not separate. If they were separate this would cause a compilation error.
```
let 😀🤠 = 1+2
let mut 😐 = &😀🤠
😐 = &100
let 😐😐 = &&😀🤠
```

The borrow checker runs before code generation. It works by converting the program into MIR (Mid Intermediate Representation), which is a simplified form of Kestrel that is boiled down to a set of simple instructions akin to assembly. These enable the borrow checker to analyze the program from a simpler view. See the .mir output files.

## MIR generation process
The borrow checker first generates MIR without any lifetime metadata. Next, another pass called the lifetime generation pass adds lifetime metadata to the MIR by analying the MIR. In addition, this pass checks for ownership invariants. Finally, the borrow checker itself runs. This ensures the reference invariants.

## Drop order
Bindings are dropped not when they go out of scope, but when they are last used. This is reflected in the outputted .mir file.
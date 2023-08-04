# Kestrel Borrow Checker

The Kestrel borrow checker ensures that several invariants are maintained:

- Each value has one owner
- There may only be one reference to any binding at one time (this is currently a contrived demonstrative example)

The borrow checker runs before code generation. It works by converting the program into MIR (Mid Intermediate Representation), which is a simplified form of Kestrel that is boiled down to a set of simple instructions akin to assembly. These enable the borrow checker to analyze the program from a simpler view. See the .mir output files.

# MIR generation process

The borrow checker first generates MIR without any lifetime metadata. Next, another pass called the lifetime generation pass adds lifetime metadata to the MIR by analying the MIR. In addition, this pass checks for ownership invariants. Finally, the borrow checker itself runs. This ensures the reference invariants.
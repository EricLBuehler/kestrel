# MIR instructions

Kestrel MIR is a boiled-down version of Kestrel. It expresses program flow by breaking down code into instructions, and is between LLVM IR and Kestrel in terms of abstraction. Its procedural, simplified form allows the Kestrel ownership and borrow checker to run.

## `I8(literal)`
Introduce a literal `i8`.
## `I16(literal)`
Introduce a literal `i16`.
## `I32(literal)`
Introduce a literal `i32`.
## `I64(literal)`
Introduce a literal `i64`.
## `I128(literal)`
Introduce a literal `i128`.
## `U8(literal)`
Introduce a literal `u8`.
## `U16(literal)`
Introduce a literal `u16`.
## `U32(literal)`
Introduce a literal `u32`.
## `U64(literal)`
Introduce a literal `u64`.
## `U128(literal)`
Introduce a literal `u128`.
## `Add(left, right)`
Add the results of `left` and `right` using the `Add` trait.
## `Declare(name, is_mut)`
Declare a binding `name` with mutability specified by `is_mut`.
## `Store(name, right)`
Store the result of `right` into the binding `name`.
## `Own(right)`
Take ownership of the result of `right`.
## `Load(name)`
Load the binding `name`.
## `Reference(right)`
Take a reference of the result of `right`.
## `Copy(right)`
Copy the result of `right`.
## `Bool(literal)`
Introduce a literal `bool`.
## `Return(right)`
Return the result of `right`.
## `CallFunction(name)`
Call the function with name `name`.
## `Eq(left, right)`
Compare the results of `left` and `right` using the `Eq` trait.
## `Ne(left, right)`
Compare the results of `left` and `right` using the `Ne` trait.`
## `Deref(right)`
Dereference the result of `right`.
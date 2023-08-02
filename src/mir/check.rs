use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    types::{Lifetime, Trait, TraitType},
};

use super::{Mir, MirInstruction, RawMirInstruction};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum ReferenceType {
    Immutable,
}

#[derive(Debug)]
pub struct MirTag {
    is_owned: bool,
    is_ref: bool,
    is_mut: bool,
    owner: Option<usize>,
    referenced: Option<Vec<(usize, ReferenceType)>>,
    lifetime: Lifetime,
}

type MirNamespace = HashMap<String, (Option<usize>, Option<usize>, MirTag)>; //(declaration, right, tag)

pub fn calculate_last_use(i: &usize, instructions: &mut Vec<MirInstruction>) -> usize {
    let mut uses = Vec::new();
    for j in (*i)..instructions.len() {
        match &instructions.get(j).as_ref().unwrap().instruction {
            RawMirInstruction::Add { left, right } => {
                if i == left || i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::Declare {
                name: _,
                is_mut: _,
                is_ref: _,
            } => {}
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::Load(_) => {}
            RawMirInstruction::Own(result) => {
                if i == result {
                    uses.push(j);
                }
            }
            RawMirInstruction::Store { name: _, right } => {
                if i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::Reference(right) => {
                if i == right {
                    uses.push(j);
                }
            }
        }
    }

    match uses.len() {
        0 => *i,
        _ => *uses.last().unwrap(),
    }
}

pub fn generate_lifetimes(this: &mut Mir, instructions: &mut Vec<MirInstruction>) -> MirNamespace {
    let mut namespace: MirNamespace = HashMap::new();
    let mut leftime_num = 0;

    for i in 0..instructions.len() {
        let mut instruction = instructions.get(i).unwrap().clone();
        match &instruction.instruction {
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::Add { left, right } => {
                let left_tp = instructions.get(*left).unwrap().tp.as_ref().unwrap();
                let right_tp = instructions.get(*right).unwrap().tp.as_ref().unwrap();
                //_res will be used in the future with custom lifetimes
                let _res = if let Some(Trait::Add { code: _, skeleton }) =
                    left_tp.traits.get(&TraitType::Add)
                {
                    skeleton(
                        this,
                        &instructions.get(*left).unwrap().pos,
                        left_tp.clone(),
                        right_tp.clone(),
                    )
                } else {
                    raise_error(
                        &format!("Type '{}' does not implement Add.", left_tp.qualname()),
                        ErrorType::TypeMismatch,
                        &instructions.get(*left).unwrap().pos,
                        &this.info,
                    );
                };
            }
            RawMirInstruction::Declare {
                ref name,
                is_mut,
                is_ref,
            } => {
                leftime_num += 1;

                let mut uses = Vec::new();
                for j in i..instructions.len() {
                    if let RawMirInstruction::Load(load_name) =
                        &instructions.get(j).as_ref().unwrap().instruction
                    {
                        if name == load_name {
                            uses.push(j);
                        }
                    }

                    if let RawMirInstruction::Store {
                        name: load_name,
                        right: _,
                    } = &instructions.get(j).as_ref().unwrap().instruction
                    {
                        if name == load_name {
                            uses.push(j);
                        }
                    }
                }
                let end_mir = if uses.is_empty() {
                    i
                } else {
                    *uses.last().unwrap()
                };

                namespace.insert(
                    name.clone(),
                    (
                        Some(i),
                        None,
                        MirTag {
                            is_owned: true,
                            is_ref: *is_ref,
                            is_mut: *is_mut,
                            owner: None,
                            referenced: None,
                            lifetime: Lifetime::ImplicitLifetime {
                                name: leftime_num.to_string(),
                                start_mir: i,
                                end_mir,
                            },
                        },
                    ),
                );
            }
            RawMirInstruction::Load(ref name) => {
                if namespace.get(name).is_none() {
                    let fmt: String = format!("Binding '{}' not found in scope.", name);
                    raise_error(
                        &fmt,
                        ErrorType::BindingNotFound,
                        &instruction.pos,
                        &this.info,
                    );
                }

                let old_instruction = &instructions
                    .get(namespace.get(name).unwrap().2.owner.unwrap())
                    .unwrap();

                if !(namespace.get(name).unwrap().2.is_owned
                    || old_instruction.tp.is_some()
                        && old_instruction
                            .tp
                            .as_ref()
                            .unwrap()
                            .traits
                            .contains_key(&TraitType::Copy))
                {
                    raise_error_multi(
                        vec![
                            format!("Use of binding '{name}' after move."),
                            "It was moved here:".into(),
                        ],
                        ErrorType::MovedBinding,
                        vec![&instruction.pos, &old_instruction.pos],
                        &this.info,
                    );
                } else {
                    namespace.get_mut(name).unwrap().2.owner = Some(i);
                }
            }
            RawMirInstruction::Own(ref item) => {
                if let RawMirInstruction::Load(ref name) =
                    instructions.get_mut(*item).unwrap().instruction
                {
                    namespace.get_mut(name).unwrap().2.is_owned = false;
                }
            }
            RawMirInstruction::Store {
                ref name,
                ref right,
            } => {
                namespace.insert(
                    name.clone(),
                    (
                        namespace.get(name).unwrap().0,
                        Some(*right),
                        MirTag {
                            is_owned: true,
                            is_mut: namespace.get(name).unwrap().2.is_mut,
                            is_ref: namespace.get(name).unwrap().2.is_ref,
                            owner: Some(*right),
                            referenced: None,
                            lifetime: namespace.get(name).unwrap().2.lifetime.clone(),
                        },
                    ),
                );
            }
            RawMirInstruction::Reference(right) => {
                if let RawMirInstruction::Load(name) =
                    &instructions.get(*right).as_ref().unwrap().instruction
                {
                    if namespace.get_mut(name).unwrap().2.referenced.is_some() {
                        namespace
                            .get_mut(name)
                            .unwrap()
                            .2
                            .referenced
                            .as_mut()
                            .unwrap()
                            .push((i, ReferenceType::Immutable));
                        namespace
                            .get_mut(name)
                            .unwrap()
                            .2
                            .referenced
                            .as_mut()
                            .unwrap()
                            .sort();
                    } else {
                        namespace.get_mut(name).unwrap().2.referenced =
                            Some(vec![(i, ReferenceType::Immutable)]);
                    }
                };
            }
        }

        if let RawMirInstruction::Declare {
            name: _,
            is_mut: _,
            is_ref: _,
        } = instruction.instruction
        {
        } else if instruction.tp.is_some() {
            leftime_num += 1;
            let end_mir = calculate_last_use(&i, instructions); //Do this before the removal!
            instructions.remove(i);

            let mutable_type = instruction.tp.as_mut().unwrap();

            mutable_type.lifetime = Lifetime::ImplicitLifetime {
                name: leftime_num.to_string(),
                start_mir: i,
                end_mir,
            };

            instructions.insert(i, instruction);
        }
    }

    let mut out = String::new();
    for (i, instruction) in instructions.iter().enumerate() {
        out.push_str(&format!(".{:<5}", format!("{}:", i)));
        out.push_str(&instruction.instruction.to_string());
        if let RawMirInstruction::Declare {
            name,
            is_mut: _,
            is_ref: _,
        } = &instruction.instruction
        {
            out.push_str(&namespace.get(name).unwrap().2.lifetime.to_string());
        }
        if instruction.tp.is_some() {
            out.push_str(&format!(
                " -> {}",
                instruction.tp.as_ref().unwrap().qualname()
            ));
            out.push_str(&format!("{}", instruction.tp.as_ref().unwrap().lifetime));
        }
        out.push('\n');
    }
    let mut f = File::create("a.mir").expect("Unable to create MIR output file.");
    f.write_all(out.as_bytes()).expect("Unable to write MIR.");

    namespace
}

pub fn check(this: &mut Mir, instructions: &mut [MirInstruction], namespace: &mut MirNamespace) {
    for (name, (_declaration, _right, tag)) in namespace.iter() {
        //This is contrived.
        let len = if tag.referenced.is_some() {
            let mut referenced = tag.referenced.as_ref().unwrap().clone();
            referenced.dedup_by(|x, y| x.1 == y.1);
            referenced.len()
        } else {
            0
        };

        if tag.referenced.is_some() && tag.referenced.as_ref().unwrap().len() >= 2 && len == 1 {
            raise_error_multi(
                vec![
                    format!("Binding '{name}' has multiple immutable references."),
                    "First reference here.".into(),
                ],
                ErrorType::MultipleReferences,
                vec![
                    &instructions
                        .get(tag.referenced.as_ref().unwrap().get(1).unwrap().0)
                        .unwrap()
                        .pos,
                    &instructions
                        .get(tag.referenced.as_ref().unwrap().first().unwrap().0)
                        .unwrap()
                        .pos,
                ],
                &this.info,
            );
        }
    }
}

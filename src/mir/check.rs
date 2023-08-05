use std::{collections::HashMap, fs::File, io::Write};

use indexmap::IndexMap;

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
    is_mut: bool,
    owner: Option<usize>,
    lifetime: Lifetime,
}

type MirNamespace = HashMap<String, (Option<usize>, Option<usize>, MirTag)>; //(declaration, right, tag)
type MirReference = (usize, ReferenceType, Lifetime, ReferenceBase); //(right, type, lifetime, referred)

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceBase {
    I32(Lifetime),
    Load {
        borrowed_life: Lifetime,
        value_life: Lifetime,
    },
    Reference(Lifetime),
}

pub fn calculate_last_use(i: &usize, instructions: &mut Vec<MirInstruction>) -> usize {
    let mut uses = Vec::new();
    for j in (*i)..instructions.len() {
        match &instructions.get(j).as_ref().unwrap().instruction {
            RawMirInstruction::Add { left, right } => {
                if i == left || i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::Declare { name: _, is_mut: _ } => {}
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
            RawMirInstruction::Copy(right) => {
                if i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::DropBinding(_, _) => {}
        }
    }

    match uses.len() {
        0 => *i,
        _ => *uses.last().unwrap(),
    }
}

pub fn generate_lifetimes<'a>(
    this: &mut Mir,
    instructions: &mut Vec<MirInstruction<'a>>,
) -> (
    MirNamespace,
    IndexMap<usize, MirReference>,
    Vec<MirInstruction<'a>>,
) {
    let mut namespace: MirNamespace = HashMap::new();
    let mut instructions_drop = instructions.clone();
    let mut lifetime_num = 0;
    let mut references = IndexMap::new();

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
            RawMirInstruction::Declare { ref name, is_mut } => {
                lifetime_num += 1;

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

                instructions_drop.insert(
                    end_mir + 1,
                    MirInstruction {
                        instruction: RawMirInstruction::DropBinding(name.clone(), end_mir + 1),
                        pos: instructions.get(end_mir).as_ref().unwrap().pos.clone(),
                        tp: instructions.get(end_mir).as_ref().unwrap().tp.clone(),
                    },
                );

                namespace.insert(
                    name.clone(),
                    (
                        Some(i),
                        None,
                        MirTag {
                            is_owned: true,
                            is_mut: *is_mut,
                            owner: None,
                            lifetime: Lifetime::ImplicitLifetime {
                                name: lifetime_num.to_string(),
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
                            owner: Some(*right),
                            lifetime: namespace.get(name).unwrap().2.lifetime.clone(),
                        },
                    ),
                );
            }
            RawMirInstruction::Reference(right) => {
                lifetime_num += 1;
                let mut rt = *right;
                let mut referred_type;
                loop {
                    match &instructions.get(rt).as_ref().unwrap().instruction {
                        RawMirInstruction::Reference(_) => {
                            referred_type = ReferenceBase::Reference(
                                instructions
                                    .get(rt)
                                    .as_ref()
                                    .unwrap()
                                    .tp
                                    .as_ref()
                                    .unwrap()
                                    .lifetime
                                    .clone(),
                            );
                            break;
                        }
                        RawMirInstruction::Load(name) => {
                            referred_type = ReferenceBase::Load {
                                borrowed_life: instructions
                                    .get(rt)
                                    .as_ref()
                                    .unwrap()
                                    .tp
                                    .as_ref()
                                    .unwrap()
                                    .lifetime
                                    .clone(),
                                value_life: instructions
                                    .get(rt)
                                    .as_ref()
                                    .unwrap()
                                    .tp
                                    .as_ref()
                                    .unwrap()
                                    .lifetime
                                    .clone(),
                            };
                            for j in rt..instructions_drop.len() {
                                if let RawMirInstruction::DropBinding(ref name_drop, _) =
                                    instructions_drop.get(j).as_ref().unwrap().instruction
                                {
                                    if name_drop == name {
                                        referred_type = ReferenceBase::Load {
                                            borrowed_life: instructions_drop
                                                .get(j)
                                                .as_ref()
                                                .unwrap()
                                                .tp
                                                .as_ref()
                                                .unwrap()
                                                .lifetime
                                                .clone(),
                                            value_life: instructions_drop
                                                .get(rt)
                                                .as_ref()
                                                .unwrap()
                                                .tp
                                                .as_ref()
                                                .unwrap()
                                                .lifetime
                                                .clone(),
                                        };
                                        break;
                                    }
                                }
                            }
                            break;
                        }
                        RawMirInstruction::Copy(new_rt) => {
                            rt = *new_rt;
                        }
                        RawMirInstruction::I32(_) => {
                            referred_type = ReferenceBase::I32(
                                instructions
                                    .get(rt)
                                    .as_ref()
                                    .unwrap()
                                    .tp
                                    .as_ref()
                                    .unwrap()
                                    .lifetime
                                    .clone(),
                            );
                            break;
                        }
                        RawMirInstruction::DropBinding(_, new_rt) => {
                            rt = *new_rt;
                        }
                        _ => {}
                    }
                }

                let res: MirReference = (
                    rt,
                    ReferenceType::Immutable,
                    Lifetime::ImplicitLifetime {
                        name: lifetime_num.to_string(),
                        start_mir: i,
                        end_mir: calculate_last_use(&i, instructions),
                    },
                    referred_type,
                );

                references.insert(i, res);
            }

            RawMirInstruction::Copy(right) => {
                let tp = instructions
                    .get(*right)
                    .as_ref()
                    .unwrap()
                    .tp
                    .clone()
                    .unwrap();
                if !tp.traits.contains_key(&TraitType::Copy) {
                    raise_error(
                        &format!("Type {} does not implement Copy", tp.qualname()),
                        ErrorType::TraitNotImplemented,
                        &instruction.pos,
                        &this.info,
                    );
                }
            }
            RawMirInstruction::DropBinding(_, _) => {}
        }

        if let RawMirInstruction::Declare { name: _, is_mut: _ } = instruction.instruction {
        } else if instruction.tp.is_some() {
            lifetime_num += 1;
            let end_mir = calculate_last_use(&i, instructions); //Do this before the removal!
            instructions.remove(i);

            let mutable_type = instruction.tp.as_mut().unwrap();

            mutable_type.lifetime = Lifetime::ImplicitLifetime {
                name: lifetime_num.to_string(),
                start_mir: i,
                end_mir,
            };

            instructions.insert(i, instruction);
        }
    }

    (namespace, references, instructions_drop)
}

pub fn write_mir(instructions: Vec<MirInstruction>, namespace: &mut MirNamespace) {
    let mut out = String::new();
    for (i, instruction) in instructions.iter().enumerate() {
        out.push_str(&format!(".{:<5}", format!("{}:", i)));
        out.push_str(&instruction.instruction.to_string());
        if let RawMirInstruction::Declare { name, is_mut: _ } = &instruction.instruction {
            out.push_str(&namespace.get(name).unwrap().2.lifetime.to_string());
        }
        if let RawMirInstruction::DropBinding(_, _) = &instruction.instruction {
        } else if instruction.tp.is_some() {
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
}

pub fn check(
    this: &mut Mir,
    instructions: &mut [MirInstruction],
    _namespace: &mut MirNamespace,
    references: &IndexMap<usize, MirReference>,
) {
    for (i, (right, _reftype, life, base)) in references {
        for (j, (_right, _reftype, other_life, other_base)) in references {
            if i >= j {
                continue;
            }

            if let ReferenceBase::Load {
                borrowed_life,
                value_life: _,
            } = base
            {
                if let ReferenceBase::Load {
                    borrowed_life: borrowed_life_other,
                    value_life: _,
                } = other_base
                {
                    let l1_end = if let Lifetime::ImplicitLifetime {
                        name: _,
                        start_mir: _,
                        end_mir,
                    } = borrowed_life
                    {
                        *end_mir
                    } else {
                        usize::MIN
                    };

                    let l2_start = if let Lifetime::ImplicitLifetime {
                        name: _,
                        start_mir,
                        end_mir: _,
                    } = borrowed_life_other
                    {
                        *start_mir
                    } else {
                        usize::MAX
                    };

                    let RawMirInstruction::Load(ref name) = instructions.get(*right).as_ref().unwrap().instruction else {unreachable!()};

                    if l1_end > l2_start {
                        raise_error_multi(
                            vec![
                                format!("Binding '{}' has multiple immutable references.", name),
                                "First reference here.".into(),
                            ],
                            ErrorType::MultipleReferences,
                            vec![
                                &instructions.get(*j).unwrap().pos,
                                &instructions.get(*i).unwrap().pos,
                            ],
                            &this.info,
                        );
                    }
                }
            }

            let l1_end = if let Lifetime::ImplicitLifetime {
                name: _,
                start_mir: _,
                end_mir,
            } = life
            {
                *end_mir
            } else {
                usize::MIN
            };

            let l2_start = if let Lifetime::ImplicitLifetime {
                name: _,
                start_mir,
                end_mir: _,
            } = other_life
            {
                *start_mir
            } else {
                usize::MAX
            };

            if l1_end > l2_start {
                raise_error_multi(
                    vec![
                        format!("Multiple immutable references."),
                        "First reference here.".into(),
                    ],
                    ErrorType::MultipleReferences,
                    vec![
                        &instructions.get(*i).unwrap().pos,
                        &instructions.get(*j).unwrap().pos,
                    ],
                    &this.info,
                );
            }
        }
    }
}

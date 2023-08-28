use indexmap::IndexMap;

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    types::{implements_trait, Lifetime, Trait, TraitType},
};

use super::{
    check, Mir, MirInstruction, MirReference, MirTag, RawMirInstruction, ReferenceBase,
    ReferenceType,
};

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
            RawMirInstruction::I8(_) => {}
            RawMirInstruction::I16(_) => {}
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::I64(_) => {}
            RawMirInstruction::I128(_) => {}
            RawMirInstruction::U8(_) => {}
            RawMirInstruction::U16(_) => {}
            RawMirInstruction::U32(_) => {}
            RawMirInstruction::U64(_) => {}
            RawMirInstruction::U128(_) => {}
            RawMirInstruction::Bool(_) => {}
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
            RawMirInstruction::Return(right) => {
                if i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::CallFunction(_) => {}
            RawMirInstruction::Eq { left, right } => {
                if i == left || i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::Ne { left, right } => {
                if i == left || i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::Deref(right) => {
                if i == right {
                    uses.push(j);
                }
            }
            RawMirInstruction::IfCondition {
                code: _,
                check_n: _,
                right,
                offset: _,
                id: _,
            } => {
                if right.is_some() && right.unwrap() == *i {
                    uses.push(j);
                }
            }
            RawMirInstruction::InstructionWrapper(_) => {}
        }
    }

    match uses.len() {
        0 => *i,
        _ => *uses.last().unwrap(),
    }
}

pub fn generate_lifetimes<'a>(
    this: &mut Mir<'a>,
    instructions: &mut Vec<MirInstruction<'a>>,
    block_res: Option<usize>,
    id: usize,
) -> IndexMap<usize, MirReference> {
    let mut lifetime_num = 0;
    let mut references = IndexMap::new();

    for i in 0..instructions.len() {
        let mut instruction = instructions.get(i).unwrap().clone();
        match &instruction.instruction {
            RawMirInstruction::I8(_) => {}
            RawMirInstruction::I16(_) => {}
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::I64(_) => {}
            RawMirInstruction::I128(_) => {}
            RawMirInstruction::U8(_) => {}
            RawMirInstruction::U16(_) => {}
            RawMirInstruction::U32(_) => {}
            RawMirInstruction::U64(_) => {}
            RawMirInstruction::U128(_) => {}
            RawMirInstruction::Bool(_) => {}
            RawMirInstruction::Add { left, right } => {
                let left_tp = instructions.get(*left).unwrap().tp.as_ref().unwrap();
                let right_tp = instructions.get(*right).unwrap().tp.as_ref().unwrap();
                //TODO: _res will be used in the future with custom lifetimes
                let _res = if let Some(Trait::Add {
                    code: _,
                    skeleton,
                    ref_n: _,
                }) = left_tp.traits.get(&TraitType::Add)
                {
                    skeleton(
                        this,
                        &instructions.get(*left).unwrap().pos,
                        left_tp.clone(),
                        right_tp.clone(),
                    )
                } else {
                    unreachable!()
                };
            }
            RawMirInstruction::Declare { ref name, is_mut } => {
                let block = this.blocks.get_mut(name.blockid).unwrap();

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

                instructions.get_mut(end_mir).unwrap().last_use = Some(name.name.clone());

                block.namespace_check.insert(
                    name.name.clone(),
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
                let block = this.blocks.get(name.blockid).unwrap();

                let old_blockid = block
                    .namespace_check
                    .get(&name.name)
                    .unwrap()
                    .2
                    .owner
                    .unwrap()
                    .1;

                let old_block = this.blocks.get(old_blockid).unwrap();

                let old_instruction = old_block
                    .instructions
                    .as_ref()
                    .unwrap()
                    .get(
                        block
                            .namespace_check
                            .get(&name.name)
                            .unwrap()
                            .2
                            .owner
                            .unwrap()
                            .0,
                    )
                    .unwrap();

                if !(block.namespace_check.get(&name.name).unwrap().2.is_owned
                    || old_instruction.tp.is_some()
                        && implements_trait(old_instruction.tp.as_ref().unwrap(), TraitType::Copy))
                {
                    raise_error_multi(
                        vec![
                            format!("Use of binding '{}' after move.", name.name),
                            "It was moved here:".into(),
                        ],
                        ErrorType::MovedBinding,
                        vec![Some(&instruction.pos), Some(&old_instruction.pos)],
                        &this.info,
                    );
                } else {
                    let block = this.blocks.get_mut(name.blockid).unwrap();
                    block.namespace_check.get_mut(&name.name).unwrap().2.owner =
                        Some((i, name.blockid));
                }
            }
            RawMirInstruction::Own(ref item) => {
                if let RawMirInstruction::Load(ref name) =
                    instructions.get_mut(*item).unwrap().instruction
                {
                    let block = this.blocks.get_mut(name.blockid).unwrap();

                    block
                        .namespace_check
                        .get_mut(&name.name)
                        .unwrap()
                        .2
                        .is_owned = false;
                }
            }
            RawMirInstruction::Store {
                ref name,
                ref right,
            } => {
                let block = this.blocks.get_mut(name.blockid).unwrap();

                block.namespace_check.insert(
                    name.name.clone(),
                    (
                        block.namespace_check.get(&name.name).unwrap().0,
                        Some(*right),
                        MirTag {
                            is_owned: true,
                            is_mut: block.namespace_check.get(&name.name).unwrap().2.is_mut,
                            owner: Some((*right, name.blockid)),
                            lifetime: block
                                .namespace_check
                                .get(&name.name)
                                .unwrap()
                                .2
                                .lifetime
                                .clone(),
                        },
                    ),
                );
            }
            RawMirInstruction::Reference(right) => {
                //Drill down to the load of a binding or literal.
                let mut rt = *right;
                let referred_type;
                loop {
                    match &instructions.get(rt).as_ref().unwrap().instruction {
                        RawMirInstruction::Reference(_) => {
                            let life = instructions
                                .get(rt)
                                .as_ref()
                                .unwrap()
                                .tp
                                .as_ref()
                                .unwrap()
                                .lifetime
                                .clone();

                            let life = match life {
                                Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: _,
                                } => Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: calculate_last_use(&rt, instructions),
                                },
                                Lifetime::Static => life,
                            };
                            referred_type = ReferenceBase::Reference(life);
                            break;
                        }
                        RawMirInstruction::Load(name) => {
                            let life = this
                                .blocks
                                .get(name.blockid)
                                .unwrap()
                                .namespace_check
                                .get(&name.name)
                                .unwrap()
                                .2
                                .lifetime
                                .clone();
                            let name = name.clone();

                            let life = match life {
                                Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: _,
                                } => Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: calculate_last_use(&rt, instructions),
                                },
                                Lifetime::Static => life,
                            };
                            referred_type = ReferenceBase::Load {
                                name,
                                bindinglife: life,
                            };
                            break;
                        }
                        RawMirInstruction::Copy(new_rt) => {
                            rt = *new_rt;
                        }

                        RawMirInstruction::I8(_)
                        | RawMirInstruction::I16(_)
                        | RawMirInstruction::I32(_)
                        | RawMirInstruction::I64(_)
                        | RawMirInstruction::I128(_)
                        | RawMirInstruction::U8(_)
                        | RawMirInstruction::U16(_)
                        | RawMirInstruction::U32(_)
                        | RawMirInstruction::U64(_)
                        | RawMirInstruction::U128(_) => {
                            let life = instructions
                                .get(rt)
                                .as_ref()
                                .unwrap()
                                .tp
                                .as_ref()
                                .unwrap()
                                .lifetime
                                .clone();
                            let life = match life {
                                Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: _,
                                } => Lifetime::ImplicitLifetime {
                                    name,
                                    start_mir,
                                    end_mir: calculate_last_use(&rt, instructions),
                                },
                                Lifetime::Static => life,
                            };

                            referred_type = ReferenceBase::Literal(life);
                            break;
                        }
                        _ => {}
                    }
                }

                let mut last = calculate_last_use(&i, instructions);
                for j in (i..instructions.len()).rev() {
                    //Find a store
                    if let RawMirInstruction::Store { name, right } =
                        &instructions.get(j).as_ref().unwrap().instruction
                    {
                        let block = this.blocks.get_mut(name.blockid).unwrap();

                        //Is it a store of this?
                        if right == &i {
                            let mut last_tmp = None;
                            for k in (j + 1)..instructions.len() {
                                if let RawMirInstruction::Store {
                                    name: other_name,
                                    right: other_right,
                                } = &instructions.get(k).as_ref().unwrap().instruction
                                {
                                    if right == other_right && name == other_name {
                                        last_tmp = Some(k);
                                        break;
                                    }
                                }
                            }
                            last = last_tmp.unwrap_or(
                                match block
                                    .namespace_check
                                    .get(&name.name)
                                    .as_ref()
                                    .unwrap()
                                    .2
                                    .lifetime
                                {
                                    Lifetime::ImplicitLifetime {
                                        name: _,
                                        start_mir: _,
                                        end_mir,
                                    } => end_mir,
                                    Lifetime::Static => {
                                        unreachable!();
                                    }
                                },
                            );
                            break;
                        }
                    }
                }

                if let Some(res) = block_res {
                    last = last.max(calculate_last_use(&(res + 1), instructions));
                }

                lifetime_num += 1;

                let res: MirReference = (
                    rt,
                    ReferenceType::Immutable,
                    Lifetime::ImplicitLifetime {
                        name: lifetime_num.to_string(),
                        start_mir: i,
                        end_mir: last,
                    },
                    referred_type,
                    id,
                );

                references.insert(i, res);
            }

            RawMirInstruction::Copy(_) => {}
            RawMirInstruction::Return(right) => {
                assert_eq!(
                    instructions.get(*right).unwrap().tp.as_ref().unwrap().ref_n,
                    0
                )
            }
            RawMirInstruction::CallFunction(_) => {}
            RawMirInstruction::Eq { left, right } => {
                let left_tp = instructions.get(*left).unwrap().tp.as_ref().unwrap();
                let right_tp = instructions.get(*right).unwrap().tp.as_ref().unwrap();
                //TODO: _res will be used in the future with custom lifetimes
                let _res = if let Some(Trait::Eq {
                    code: _,
                    skeleton,
                    ref_n: _,
                }) = left_tp.traits.get(&TraitType::Eq)
                {
                    skeleton(
                        this,
                        &instructions.get(*left).unwrap().pos,
                        left_tp.clone(),
                        right_tp.clone(),
                    )
                } else {
                    unreachable!()
                };
            }
            RawMirInstruction::Ne { left, right } => {
                let left_tp = instructions.get(*left).unwrap().clone().tp.unwrap().clone();
                let right_tp = instructions
                    .get(*right)
                    .unwrap()
                    .clone()
                    .tp
                    .unwrap()
                    .clone();
                //TODO: _res will be used in the future with custom lifetimes
                let _res = if let Some(Trait::Ne {
                    code: _,
                    skeleton,
                    ref_n: _,
                }) = left_tp.traits.get(&TraitType::Ne)
                {
                    skeleton(
                        this,
                        &instructions.get(*left).unwrap().pos,
                        left_tp.clone(),
                        right_tp.clone(),
                    )
                } else {
                    unreachable!()
                };
            }
            RawMirInstruction::Deref(right) => {
                let rt_instruction = instructions.get(*right).unwrap();
                let mut tp = rt_instruction.tp.as_ref().unwrap().clone();
                tp.ref_n -= 1;
                if !implements_trait(&tp, TraitType::Copy) {
                    if let RawMirInstruction::Load(name) = &rt_instruction.instruction {
                        let fmt: String = format!(
                            "Cannot move non Copy-able type '{}' out of binding '{}'.",
                            tp.qualname(),
                            &name.name
                        );
                        raise_error(
                            &fmt,
                            ErrorType::CannotMoveOutOfBinding,
                            &rt_instruction.pos,
                            &this.info,
                        );
                    } else {
                        let fmt: String =
                            format!("Cannot move out of not Copy-able type '{}'.", tp.qualname());
                        raise_error(
                            &fmt,
                            ErrorType::CannotMoveOutOfNonCopy,
                            &rt_instruction.pos,
                            &this.info,
                        );
                    }
                }
            }
            RawMirInstruction::IfCondition {
                code,
                check_n: _,
                right: _,
                offset: _,
                id,
            } => {
                check(this, &mut code.clone(), Some(i), *id);
            }
            RawMirInstruction::InstructionWrapper(_) => {}
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

    references
}

fn check_value_life(this: &mut Mir, life: &Lifetime, right: &usize, id: usize) {
    let blockend = this.block_positions.get(&id).unwrap().1;
    match life {
        Lifetime::ImplicitLifetime {
            name: _,
            start_mir: _,
            end_mir: ref_end,
        } => {
            if ref_end < &blockend {
                raise_error_multi(
                    vec![
                        "Value does not live long enough.".into(),
                        "It must live until the end of this block:".into(),
                    ],
                    ErrorType::ValueNotLiveEnough,
                    vec![
                        Some(&this.instructions.get(*right).unwrap().pos),
                        Some(&this.instructions.get(blockend).unwrap().pos),
                    ],
                    &this.info,
                );
            }
        }

        Lifetime::Static => {}
    }
}

pub fn check_references(
    this: &mut Mir,
    instructions: &mut [MirInstruction],
    references: &IndexMap<usize, MirReference>,
    id: usize,
) {
    for (_, (right, _reftype, _life, base, refblock)) in references {
        if *refblock == id
            && (this.block_positions.get(&id).unwrap().0..this.block_positions.get(&id).unwrap().1)
                .contains(right)
        {
            check_value_life(
                this,
                match base {
                    ReferenceBase::Literal(lifetime) => lifetime,
                    ReferenceBase::Load {
                        name: _,
                        bindinglife,
                    } => bindinglife,
                    ReferenceBase::Reference(lifetime) => lifetime,
                },
                right,
                id,
            );
        }
    }

    for (i, (right, _reftype, life, base1, _)) in references {
        for (j, (_right, _reftype, other_life, base2, _)) in references {
            if i >= j {
                continue;
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

            if base1 == base2 {
                if let RawMirInstruction::Load(ref name) =
                    instructions.get(*right).as_ref().unwrap().instruction
                {
                    if l1_end > l2_start {
                        raise_error_multi(
                            vec![
                                format!(
                                    "Binding '{}' has multiple immutable references.",
                                    &name.name
                                ),
                                "First reference here.".into(),
                            ],
                            ErrorType::MultipleImmutableReferences,
                            vec![
                                Some(&instructions.get(*j).unwrap().pos),
                                Some(&instructions.get(*i).unwrap().pos),
                            ],
                            &this.info,
                        );
                    }
                } else if l1_end > l2_start {
                    raise_error_multi(
                        vec![
                            "Value has multiple immutable references.".into(),
                            "First reference here.".into(),
                        ],
                        ErrorType::MultipleImmutableReferences,
                        vec![
                            Some(&instructions.get(*j).unwrap().pos),
                            Some(&instructions.get(*i).unwrap().pos),
                        ],
                        &this.info,
                    );
                }
            }
        }
    }
}

pub fn check_return(_this: &mut Mir, _instructions: &mut [MirInstruction]) {
    /*
    for instruction in instructions {
        if let RawMirInstruction::Return(_) = instruction.instruction {
            return;
        }
    }
    raise_error(
        &format!("Function '{}' does not return.", this.fn_name),
        ErrorType::TraitNotImplemented,
        &this.fn_pos,
        &this.info,
    );
    */
}

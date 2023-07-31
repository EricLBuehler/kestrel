use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    types::{Lifetime, TraitType},
    utils::FileInfo,
};

use super::{MirInstruction, RawMirInstruction};

#[derive(Debug)]
struct MirTag {
    isowned: bool,
    owner: Option<usize>,
    lifetime: Lifetime,
}

pub fn check(mut instructions: Vec<MirInstruction>, info: FileInfo<'_>) {
    let mut namespace: HashMap<String, (Option<usize>, MirTag)> = HashMap::new();
    let mut leftime_num = 0;

    for i in 0..instructions.len() {
        let instruction = instructions.get(i).unwrap().clone();
        match instruction.instruction {
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::Add { left: _, right: _ } => {}
            RawMirInstruction::Declare {
                ref name,
                is_mut: _,
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
                        None,
                        MirTag {
                            isowned: true,
                            owner: None,
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
                    raise_error(&fmt, ErrorType::BindingNotFound, &instruction.pos, &info);
                }

                let old_instruction = &instructions
                    .get(namespace.get(name).unwrap().1.owner.unwrap())
                    .unwrap();

                if !(namespace.get(name).unwrap().1.isowned
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
                            format!("Binding '{name}' is not owned"),
                            "It was moved here:".into(),
                        ],
                        ErrorType::MovedBinding,
                        vec![&instruction.pos, &old_instruction.pos],
                        &info,
                    );
                } else {
                    namespace.get_mut(name).unwrap().1.owner = Some(i);
                }
            }
            RawMirInstruction::Own(ref item) => {
                if let RawMirInstruction::Load(ref name) =
                    instructions.get_mut(*item).unwrap().instruction
                {
                    namespace.get_mut(name).unwrap().1.isowned = false;
                }
            }
            RawMirInstruction::Store {
                ref name,
                ref right,
            } => {
                namespace.insert(
                    name.clone(),
                    (
                        Some(*right),
                        MirTag {
                            isowned: true,
                            owner: Some(*right),
                            lifetime: namespace.get(name).unwrap().1.lifetime.clone(),
                        },
                    ),
                );
            }
        }
    }

    let mut out = String::new();
    for (i, instruction) in instructions.iter().enumerate() {
        out.push_str(&format!(".{:<5}", format!("{}:", i)));
        out.push_str(&instruction.instruction.to_string());
        if let RawMirInstruction::Declare { name, is_mut: _ } = &instruction.instruction {
            out.push_str(&namespace.get(name).unwrap().1.lifetime.to_string());
        }
        if instruction.tp.is_some() {
            out.push_str(&format!(
                " -> {}",
                instruction.tp.as_ref().unwrap().qualname
            ));
            out.push_str(&format!("{}", instruction.tp.as_ref().unwrap().lifetime));
        }
        out.push('\n');
    }
    let mut f = File::create("a.mir").expect("Unable to create MIR output file.");
    f.write_all(out.as_bytes()).expect("Unable to write MIR.");
}

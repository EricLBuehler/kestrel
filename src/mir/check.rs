use std::collections::HashMap;

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    utils::FileInfo,
};

use super::MirInstruction;

#[derive(Debug)]
struct MirTag {
    isowned: bool,
    owner: Option<usize>,
}

pub fn check(mut instructions: Vec<MirInstruction>, info: FileInfo<'_>) {
    let mut namespace: HashMap<String, (Option<usize>, MirTag)> = HashMap::new();

    for i in 0..instructions.len() {
        let instruction = instructions.get(i).unwrap().clone();
        match instruction {
            MirInstruction::I32(_, _) => {}
            MirInstruction::Add {
                left: _,
                right: _,
                pos: _,
            } => {}
            MirInstruction::Declare(ref name, _) => {
                namespace.insert(
                    name.clone(),
                    (
                        None,
                        MirTag {
                            isowned: true,
                            owner: None,
                        },
                    ),
                );
            }
            MirInstruction::Load(ref name, ref pos) => {
                if namespace.get(name).is_none() {
                    let fmt: String = format!("Binding '{}' not found in scope.", name);
                    raise_error(&fmt, ErrorType::BindingNotFound, pos, &info);
                }

                let oldpos = match instructions
                    .get(namespace.get(name).unwrap().1.owner.unwrap())
                    .unwrap()
                {
                    MirInstruction::Add {
                        left: _,
                        right: _,
                        pos,
                    } => pos,
                    MirInstruction::Declare(_, pos) => pos,
                    MirInstruction::I32(_, pos) => pos,
                    MirInstruction::Load(_, pos) => pos,
                    MirInstruction::Store {
                        name: _,
                        right: _,
                        pos,
                    } => pos,
                    MirInstruction::Own(_, pos) => pos,
                };

                if !namespace.get(name).unwrap().1.isowned {
                    raise_error_multi(
                        vec![
                            format!("Binding '{name}' is not owned"),
                            "It was moved here:".into(),
                        ],
                        ErrorType::MovedBinding,
                        vec![pos, oldpos],
                        &info,
                    );
                } else {
                    namespace.get_mut(name).unwrap().1.owner = Some(i);
                }
            }
            MirInstruction::Own(ref item, _) => {
                if let MirInstruction::Load(ref name, _) = instructions.get_mut(*item).unwrap() {
                    namespace.get_mut(name).unwrap().1.isowned = false;
                }
            }
            MirInstruction::Store {
                ref name,
                ref right,
                pos: _,
            } => {
                namespace.insert(
                    name.clone(),
                    (
                        Some(*right),
                        MirTag {
                            isowned: true,
                            owner: Some(*right),
                        },
                    ),
                );
            }
        }
    }
}

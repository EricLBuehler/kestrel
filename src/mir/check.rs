use std::collections::HashMap;

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    types::TraitType,
    utils::FileInfo,
};

use super::{MirInstruction, RawMirInstruction};

#[derive(Debug)]
struct MirTag {
    isowned: bool,
    owner: Option<usize>,
}

pub fn check(mut instructions: Vec<MirInstruction>, info: FileInfo<'_>) {
    let mut namespace: HashMap<String, (Option<usize>, MirTag)> = HashMap::new();

    for i in 0..instructions.len() {
        let instruction = instructions.get(i).unwrap().clone();
        match instruction.instruction {
            RawMirInstruction::I32(_) => {}
            RawMirInstruction::Add { left: _, right: _ } => {}
            RawMirInstruction::Declare(ref name) => {
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
                        },
                    ),
                );
            }
        }
    }
}

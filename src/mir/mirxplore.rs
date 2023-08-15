use std::io::{self, Write};

use indexmap::IndexMap;

use crate::{mir::output_mir, types::Lifetime};

use super::{MirInstruction, MirNamespace, MirReference};

#[allow(unused_assignments)]
pub fn explore(
    instructions: &[MirInstruction<'_>],
    namespace: &mut MirNamespace,
    references: &IndexMap<usize, MirReference>,
) {
    let mut buf = String::from("");
    println!("Kestrel MIR Debugger");
    println!("Type `help`, `quit`, `binding [name]`, or `ref [number]`");
    println!("Note: the reference number is the MIR reference number.");
    loop {
        buf = "".into();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        buf.pop();

        let res = &buf.split(' ').collect::<Vec<_>>()[..];
        if res[0] == "help" {
            println!("Type `quit`, `binding [name]`, or `ref [number]`");
            println!("Note: the reference number is the MIR reference number.");
        } else if res[0] == "binding" {
            let data = namespace.get(res[1]);
            if data.is_none() {
                println!(
                    "Binding {} is not found, here are the defined ones: {:?}",
                    res[1],
                    namespace.keys().collect::<Vec<_>>()
                );
                continue;
            }
            println!("Binding '{}'", res[1]);
            println!("Lifetime: {}", data.unwrap().2.lifetime);
            let life = data.unwrap().2.lifetime.clone();
            match &life {
                Lifetime::ImplicitLifetime {
                    name: _,
                    start_mir,
                    end_mir,
                } => {
                    let mut out = String::from("");
                    output_mir(
                        &instructions[*start_mir..=*end_mir],
                        namespace,
                        &mut out,
                        start_mir,
                    );
                    println!("{out}");
                }
                Lifetime::Static => {
                    unreachable!();
                }
            }
        } else if res[0] == "ref" {
            let num = res[1].parse::<usize>().unwrap();
            let data = references.get(&num);
            if data.is_none() {
                println!(
                    "Reference {} is not found, here are the defined ones: {:?}",
                    res[1],
                    references.keys().collect::<Vec<_>>()
                );
                continue;
            }
            println!("Reference .{}", num);
            println!("Lifetime: {}", data.unwrap().2);
            match &data.unwrap().2 {
                Lifetime::ImplicitLifetime {
                    name: _,
                    start_mir,
                    end_mir,
                } => {
                    let mut out = String::from("");
                    output_mir(
                        &instructions[*start_mir..=*end_mir],
                        namespace,
                        &mut out,
                        start_mir,
                    );
                    println!("{out}");
                }
                Lifetime::Static => {
                    unreachable!();
                }
            }
        } else if res[0] == "quit" {
            break;
        } else {
            println!("Command '{}' not recognized.", res[0]);
        }
    }
}

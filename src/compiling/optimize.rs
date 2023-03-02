use ahash::{HashMap, HashMapExt};

use super::bytecode::Function;
use crate::vm::opcodes::Opcode;

pub fn optimize_function(func: &Function<usize>) -> Function<usize> {
    let mut prev_func = func.clone();
    let mut optimized_func = remove_unused(func);

    while optimized_func.opcodes != prev_func.opcodes {
        prev_func = optimized_func.clone();

        optimized_func = remove_unused(&optimized_func);
    }

    return optimized_func;
}

/// removes both unused operations and unused registers
fn remove_unused(func: &Function<usize>) -> Function<usize> {
    // this is so fucking dumb
    let mut write = [false; 1024]; // I guess 1024 are enough atm
    let mut read = [false; 1024]; // we may change these to hashmaps or something, idk

    for op in &func.opcodes {
        match *op {
            Opcode::LoadBuiltins { dest } => write[dest] = true,
            Opcode::LoadConst { dest, id: _ } => write[dest] = true,
            Opcode::LoadEmpty { dest } => write[dest] = true,
            Opcode::LoadNone { dest } => write[dest] = true,
            Opcode::Add { left, right, dest } => {
                read[left] = true;
                read[right] = true;
                write[dest] = true;
            },
            Opcode::Sub { left, right, dest } => {
                read[left] = true;
                read[right] = true;
                write[dest] = true;
            },
            Opcode::Copy { from, to } => {
                read[from] = true;
                write[to] = true;
            },
            Opcode::Print { reg } => {
                read[reg] = true;
            },
            Opcode::JumpIfFalse { src, to: _ } => read[src] = true,
            Opcode::Jump { to: _ } => {},
            Opcode::Lt { left, right, dest } => {
                read[left] = true;
                read[right] = true;
                write[dest] = true;
            },
            _ => {
                println!("OPTIMIZATION: UNIMPLEMENTED OPERATOR [{:?}]", op);
            },
        }
    }

    let mut registers: HashMap<usize, usize> = HashMap::new();
    let mut next_register = 0;
    let mut get_reg = |reg| {
        if let Some(actual_reg) = registers.get(&reg) {
            println!("{}", actual_reg);
            *actual_reg
        } else {
            let actual_reg = next_register;
            assert_eq!(true, registers.insert(reg, next_register).is_none());
            next_register += 1;
            actual_reg
        }
    };
    let is_used = |reg| {
        match (read[reg], write[reg]) {
            (true, true) => true,
            (true, false) => false, // unreachable!(),
            (false, true) => false,
            (false, false) => false,
        }
    };

    let opcodes = func
        .opcodes
        .iter()
        .copied()
        .map(|op| match op {
            Opcode::LoadBuiltins { dest } => is_used(dest).then(|| Opcode::LoadBuiltins {
                dest: get_reg(dest),
            }),
            Opcode::LoadConst { dest, id } => is_used(dest).then(|| Opcode::LoadConst {
                dest: get_reg(dest),
                id,
            }),
            Opcode::LoadEmpty { dest } => is_used(dest).then(|| Opcode::LoadEmpty {
                dest: get_reg(dest),
            }),
            Opcode::LoadNone { dest } => is_used(dest).then(|| Opcode::LoadNone {
                dest: get_reg(dest),
            }),
            Opcode::Add { left, right, dest } => is_used(dest).then(|| Opcode::Add {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Sub { left, right, dest } => is_used(dest).then(|| Opcode::Sub {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Lt { left, right, dest } => is_used(dest).then(|| Opcode::Lt {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Lte { left, right, dest } => is_used(dest).then(|| Opcode::Lte {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Gt { left, right, dest } => is_used(dest).then(|| Opcode::Gt {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Gte { left, right, dest } => is_used(dest).then(|| Opcode::Gte {
                left: get_reg(left),
                right: get_reg(right),
                dest: get_reg(dest),
            }),
            Opcode::Copy { from, to } => is_used(to).then(|| Opcode::Copy {
                from: get_reg(from),
                to: get_reg(to),
            }),
            Opcode::Print { reg } => Some(Opcode::Print { reg: get_reg(reg) }),
            Opcode::JumpIfFalse { src, to } => Some(Opcode::JumpIfFalse {
                src: get_reg(src),
                to,
            }),
            Opcode::Jump { to } => Some(Opcode::Jump { to }),
            _ => {
                println!("OPTIMIZATION: UNIMPLEMENTED OPERATOR [{:?}]", op);
                Some(op)
            },
        })
        .collect::<Vec<_>>();

    let mut output = Function { opcodes: vec![] };

    // fix jumps
    for (_i, op) in opcodes
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
    {
        match op {
            Opcode::Jump { to } | Opcode::JumpIfFalse { src: _, to } => {
                let to = to as usize;

                let output_address = to
                    - &opcodes[0..to]
                        .into_iter()
                        .filter(|v| matches!(v, None))
                        .count();

                output.opcodes.push(match op {
                    Opcode::Jump { to: _ } => Opcode::Jump {
                        to: output_address as u16,
                    },
                    Opcode::JumpIfFalse { src, to: _ } => Opcode::JumpIfFalse {
                        src,
                        to: output_address as u16,
                    },
                    _ => unreachable!(),
                });
            },
            _ => output.opcodes.push(op),
        }
    }

    output
}

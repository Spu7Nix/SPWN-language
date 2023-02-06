// CURRENT PROBLEM:
// - reading variables in a sub-macro isn't possible if optimized

use ahash::{HashMap, HashMapExt};

use crate::vm::opcodes::Opcode;
use super::bytecode::Function;

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

    for (input, output) in &func.capture_regs {
        write[*output] = true;
    }

    macro_rules! opcode {
        ($( $bytecode:ident { $( $var:ident($typ:ident) )* } )*) => {
            {
                for op in &func.opcodes {
                    opcode!(
                        READ/WRITE
                        op
                        $(
                            $bytecode { $( $var($typ) )* }
                        )*
                    );
                }

                let is_used = |reg| {
                    match (read[reg], write[reg]) {
                        (true, true) => true,
                        (true, false) => true, // this could be false
                        (false, true) => false,
                        (false, false) => false,
                    }
                };

                func.opcodes
                    .iter()
                    .copied()
                    .map(|op|
                        opcode!(
                            UNUSED
                            op
                            is_used
                            $(
                                $bytecode { $( $var($typ) )* }
                            )*
                        )
                    )
                    .collect()
            }
        };

        (READ/WRITE $op:ident $( $bytecode:ident { $( $var:ident($typ:ident) )* } )*) => {
            match *$op {
                $(
                    #[allow(unused_variables)]
                    Opcode::$bytecode { $( $var, )* } => {
                        $(
                            opcode!(#READ/WRITE $var $typ);
                        )*
                    },
                )*
                _ => opcode!(UNIMPLEMENTED $op),
            }
        };
        (#READ/WRITE $var:ident read) => { read[$var] = true; };
        (#READ/WRITE $var:ident write) => { write[$var] = true; };
        (#READ/WRITE $var:ident constant) => {};
        (#READ/WRITE $var:ident jump) => {};
        (#READ/WRITE $var:ident ignore) => {};

        (UNUSED $op:ident $is_used:ident $( $bytecode:ident { $( $var:ident($typ:ident) )* } )*) => {
            match $op {
                $(
                    Opcode::$bytecode { $( $var, )* } => [$(opcode!(#IS_USED $is_used $var($typ)),)*].iter().all(|v| *v == true).then(|| Opcode::$bytecode {
                        $(
                            $var: opcode!(#UNUSED $var($typ)),
                        )*
                    }),
                )*
                _ => {
                    opcode!(UNIMPLEMENTED $op);
                    Some($op)
                },
            }
        };
        (#UNUSED $var:ident(read)) => { get_reg($var) };
        (#UNUSED $var:ident(write)) => { get_reg($var) };
        (#UNUSED $var:ident(constant)) => { $var };
        (#UNUSED $var:ident(jump)) => { $var };
        (#UNUSED $var:ident(ignore)) => { $var };

        (#IS_USED $is_used:ident $var:ident(write)) => { $is_used($var) };
        (#IS_USED $is_used:ident $var:ident($_:ident)) => { true };

        (UNIMPLEMENTED $op:ident) => {
            println!("OPTIMIZATION: UNIMPLEMENTED OPERATOR [{:?}]", $op)
        };
    }

    let opcodes: Vec<Option<Opcode<usize>>> = opcode!(
        LoadConst { id(constant) dest(write) }

        Copy { from(read) to(write) }
        Print { reg(read) }

        Call { base(ignore) args(ignore) dest(write) }

        AllocArray { size(ignore) dest(write) }
        AllocDict { size(ignore) dest(write) }

        PushArrayElem { elem(read) dest(write) }
        PushDictElem { elem(read) key(ignore) dest(write) }

        CreateMacro { id(ignore) dest(write)  }
        PushMacroArg { name(read) dest(write) }
        SetMacroArgDefault { src(read) dest(write) }
        SetMacroArgPattern { src(read) dest(write) }

        Add { left(read) right(read) dest(write) }
        Sub { left(read) right(read) dest(write) }
        Mult { left(read) right(read) dest(write) }
        Div { left(read) right(read) dest(write) }
        Mod { left(read) right(read) dest(write) }
        Pow { left(read) right(read) dest(write) }
        ShiftLeft { left(read) right(read) dest(write) }
        ShiftRight { left(read) right(read) dest(write) }
        BinOr { left(read) right(read) dest(write) }
        BinAnd { left(read) right(read) dest(write) }

        // TODO: this requires an operator to be both read and write, which isn't implemented yet
        AddEq { left(write) right(read) }
        SubEq { left(write) right(read) }
        MultEq { left(write) right(read) }
        DivEq { left(write) right(read) }
        ModEq { left(write) right(read) }
        PowEq { left(write) right(read) }
        ShiftLeftEq { left(write) right(read) }
        ShiftRightEq { left(write) right(read) }
        BinOrEq { left(write) right(read) }
        BinAndEq { left(write) right(read) }
        BinNotEq { left(write) right(read) }

        Not { src(read) dest(write) }
        Negate { src(read) dest(write) }
        BinNot { src(read) dest(write) }

        Eq { left(read) right(read) dest(write) }
        Neq { left(read) right(read) dest(write) }

        Gt { left(read) right(read) dest(write) }
        Lt { left(read) right(read) dest(write) }
        Gte { left(read) right(read) dest(write) }
        Lte { left(read) right(read) dest(write) }

        Range { left(read) right(read) dest(write) }
        In { left(read) right(read) dest(write) }
        As { left(read) right(read) dest(write) }
        Is { left(read) right(read) dest(write) }

        And { left(read) right(read) dest(write) }
        Or { left(read) right(read) dest(write) }

        Jump { to(jump) }
        JumpIfFalse { src(read) to(jump) }

        Ret { src(read) }

        WrapMaybe { src(read) dest(write) }
        LoadNone { dest(write) }
        LoadEmpty { dest(write) }

        Index { from(read) dest(write) index(read) }
        Member { from(read) dest(write) member(read) }
        Associated { from(read) dest(write) name(read) }

        // empty enum?
        // YeetContext
        EnterArrowStatement { skip_to(jump) }

        LoadBuiltins { dest(write) }
        
        Export { src(read) }
        Import { src(ignore) dest(write) }
    );

    
    let mut output = func.clone();
    output.opcodes.clear();
    output.regs_used = read.clone().iter().filter(|v| **v).count();

    // fix captures
    output.capture_regs = output.capture_regs.iter().map(|(input, output)| (get_reg(*input), *output)).collect();

    // fix jumps
    for (_i, op) in opcodes.iter().enumerate().filter_map(|(i, v)| v.map(|v| (i, v))) {
        match op {
            Opcode::Jump { to } | Opcode::JumpIfFalse { src: _, to } => {
                let to = to as usize;

                let output_address = to - &opcodes[0..to].into_iter().filter(|v| matches!(v, None)).count();

                output.opcodes.push(match op {
                    Opcode::Jump { to: _ } => Opcode::Jump { to: output_address as u16 },
                    Opcode::JumpIfFalse { src, to: _ } => Opcode::JumpIfFalse { src, to: output_address as u16 },
                    _ => unreachable!(),
                });
            },
            _ => output.opcodes.push(op),
        }
    }

    output
}

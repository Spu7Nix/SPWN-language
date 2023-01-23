use ahash::{HashMap, HashMapExt};
use petgraph::{Graph, graph::NodeIndex};

use crate::vm::opcodes::Opcode;
use super::bytecode::Function;

pub fn optimize_function(func: &Function<usize>) -> Function<usize> {
    let optimized_func = remove_unused(func);

    return func.clone();
}

/// removes both unused operations and unused registers
fn remove_unused(func: &Function<usize>) -> Function<usize> {
    type GraphNode = (usize, Opcode<usize>);

    let size = func.opcodes.len();

    let mut write_graph = Graph::<GraphNode, ()>::new();
    let mut read_graph = Graph::<GraphNode, ()>::new();
    let mut jump_graph = Graph::<GraphNode, ()>::new();
    let mut constant_graph = Graph::<GraphNode, ()>::new();

    // vectors would be enough
    let mut write_nodes = Vec::with_capacity(size);
    let mut read_nodes = Vec::with_capacity(size);
    let mut jump_nodes = Vec::with_capacity(size);
    let mut constant_nodes = Vec::with_capacity(size);

    // CREATE NODES
    for (i, op) in func.opcodes.iter().copied().enumerate() {
        let write_node = write_graph.add_node((i, op));
        write_nodes.push(write_node);

        let read_node = read_graph.add_node((i, op));
        read_nodes.push(read_node);

        let jump_node = jump_graph.add_node((i, op));
        jump_nodes.push(jump_node);

        let constant_node = constant_graph.add_node((i, op));
        constant_nodes.push(constant_node);
    }

    macro_rules! unused {
        ($( $variant:ident $tree:tt )*) => {
            {
                for (i, op) in func.opcodes.iter().copied().enumerate() {
                    match op {
                        $(
                            unused!(@ $variant $tree) => unused!(= $variant $tree),
                        )*
                        _ => todo!("{:?}", op),
                    };
                }
            }
        };
    
        (@ $variant:ident { $($name:ident($typ:ident) $(,)?)* }) => {
            Opcode::$variant { $($name,)* }
        };

        (= $variant:ident { $($name:ident($typ:ident) $(,)?)* }) => {
            {
                $(
                    unused!(# $name($typ));
                )*
            }
        };

        (# $name:ident(read)) => {
            drop($name);
        };
        (# $name:ident(write)) => {
            drop($name);
        };
        (# $name:ident(jump)) => {
            drop($name);
        };
        (# $name:ident(constant)) => {
            drop($name);
        };
    }

    for (i, op) in func.opcodes.iter().copied().enumerate() {
        let write_node = write_graph.add_node((i, op));
        let read_node = read_graph.add_node((i, op));
        let jump_node = jump_graph.add_node((i, op));
        let constant_node = constant_graph.add_node((i, op));

        unused!(
            LoadBuiltins { dest(write) }
            LoadConst { id(constant) dest(write) }
            LoadEmpty { dest(write) }
            LoadNone { dest(write) }

            Add { left(read), right(read), dest(write) }
            Sub { left(read), right(read), dest(write) }

            Lt { left(read), right(read), dest(write) }
            Lte { left(read), right(read), dest(write) }
            Gt { left(read), right(read), dest(write) }
            Gte { left(read), right(read), dest(write) }

            Copy { from(read), to(write) }
            Print { reg(read) }

            JumpIfFalse { src(read), to(jump) }
            Jump { to(jump) }
        );
    }

    func.clone()
    // todo!()

    // v1
    // let mut write = [false; 256];
    // let mut read = [false; 256];
    // for op in &func.opcodes {
    //     match *op {
    //         Opcode::LoadBuiltins { dest } => write[dest] = true,
    //         Opcode::LoadConst { dest, id: _ } => write[dest] = true,
    //         Opcode::LoadEmpty { dest } => write[dest] = true,
    //         Opcode::LoadNone { dest } => write[dest] = true,
    //         Opcode::Add { left, right, dest } => {
    //             read[left] = true;
    //             read[right] = true;
    //             write[dest] = true;
    //         }
    //         Opcode::Sub { left, right, dest } => {
    //             read[left] = true;
    //             read[right] = true;
    //             write[dest] = true;
    //         }
    //         Opcode::Copy { from, to } => {
    //             read[from] = true;
    //             write[to] = true;
    //         }
    //         Opcode::Print { reg } => {
    //             read[reg] = true;
    //         }
    //         Opcode::JumpIfFalse { src, to: _ } => read[src] = true,
    //         Opcode::Jump { to: _ } => {}
    //         Opcode::Lt { left, right, dest } => {
    //             read[left] = true;
    //             read[right] = true;
    //             write[dest] = true; 
    //         }
    //         _ => unimplemented!("{:?}", op),
    //     }
    // }
    // let mut output: Function<usize> = Function { opcodes: vec![] };
    // let mut registers: HashMap<usize, usize> = HashMap::new();
    // let mut next_register = 0;
    // let mut get_reg = |reg| {
    //     if let Some(actual_reg) = registers.get(&reg) {
    //         println!("{}", actual_reg);
    //         *actual_reg
    //     } else {
    //         let actual_reg = next_register;
    //         assert_eq!(true, registers.insert(reg, next_register).is_none());
    //         next_register += 1;
    //         actual_reg
    //     }
    // };
    // let is_used = |reg| {
    //     match (read[reg], write[reg]) {
    //         (true, true) => true,
    //         (true, false) => false, // unreachable!(),
    //         (false, true) => false,
    //         (false, false) => false,
    //     }
    // };
    // for op in &func.opcodes {
    //     let opcode = match *op {
    //         Opcode::LoadBuiltins { dest } =>
    //             is_used(dest).then_some(|| Opcode::LoadBuiltins { dest: get_reg(dest) }),
    //         Opcode::LoadConst { dest, id } =>
    //             is_used(dest).then_some(|| Opcode::LoadConst { dest: get_reg(dest), id }),
    //         Opcode::LoadEmpty { dest } =>
    //             is_used(dest).then_some(|| Opcode::LoadEmpty { dest: get_reg(dest) }),
    //         Opcode::LoadNone { dest } => 
    //             is_used(dest).then_some(|| Opcode::LoadNone { dest: get_reg(dest) }),
    //         Opcode::Add { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Add { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Sub { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Sub { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Lt { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Lt { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Lte { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Lte { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Gt { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Gt { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Gte { left, right, dest } =>
    //             is_used(dest).then_some(|| Opcode::Gte { left: get_reg(left), right: get_reg(right), dest: get_reg(dest) }),
    //         Opcode::Copy { from, to } =>
    //             is_used(to).then_some(|| Opcode::Copy { from: get_reg(from), to: get_reg(to) }),
    //         Opcode::Print { reg } => Some(|| Opcode::Print { reg: get_reg(reg) }),
    //         Opcode::JumpIfFalse { src, to } => Some(|| Opcode::JumpIfFalse { src: get_reg(src), to }),
    //         Opcode::Jump { to } => Some(|| Opcode::Jump { to }),
    //         _ => unimplemented!("{:?}", op),
    //     };
    //     if let Some(opcode) = opcode {
    //         output.opcodes.push(opcode);
    //     }
    // }
    // output
}

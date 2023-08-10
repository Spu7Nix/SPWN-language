use ahash::{AHashMap, AHashSet};
use heuristic_graph_coloring::{color_rlf, ColorableGraph};
use itertools::Itertools;

use crate::compiling::bytecode::{Function, Register, UnoptBytecode, UnoptFunction, UnoptRegister};
use crate::compiling::opcodes::{CallExprID, FuncID, Opcode, OpcodePos, UnoptOpcode};

struct Node<'a> {
    opcode: &'a UnoptOpcode,
    live_in: AHashSet<UnoptRegister>,
}

pub struct InterferenceGraph {
    pub adjacency_map: Vec<Vec<usize>>,
}

impl InterferenceGraph {
    fn new(capacity: usize) -> Self {
        InterferenceGraph {
            adjacency_map: vec![vec![]; capacity],
        }
    }

    fn add_edge(&mut self, var1: usize, var2: usize) {
        if var1 != var2 && !self.adjacency_map[var1].contains(&var2) {
            self.adjacency_map[var1].push(var2);
            self.adjacency_map[var2].push(var1);
        }
    }

    pub fn from_func(func: &UnoptFunction, code: &UnoptBytecode) -> Self {
        let mut nodes = func
            .opcodes
            .iter()
            .map(|r| Node {
                opcode: r,
                live_in: AHashSet::new(),
            })
            .collect_vec();

        loop {
            let mut changed = false;

            let mut visited = AHashSet::new();

            fn visit(
                node_idx: OpcodePos,
                visited: &mut AHashSet<OpcodePos>,
                nodes: &mut Vec<Node>,
                changed: &mut bool,
                code: &UnoptBytecode,
            ) {
                // depth-first post-order traversal
                visited.insert(node_idx);
                let successors = nodes[*node_idx as usize]
                    .opcode
                    .get_successors(node_idx.into(), nodes.len());
                for succ in successors.iter() {
                    if !visited.contains(succ) {
                        visit(*succ, visited, nodes, changed, code);
                    }
                }

                // `LiveOut` set
                let mut out = AHashSet::new();
                for succ in successors {
                    for reg in &nodes[*succ as usize].live_in {
                        out.insert(*reg);
                    }
                }

                let node = &mut nodes[*node_idx as usize];

                let mut new_live_in: AHashSet<_> =
                    node.opcode.get_read(code).iter().copied().collect();

                let write = node.opcode.get_write(code);
                for i in out {
                    // compute difference of sets
                    if !write.contains(&i) {
                        new_live_in.insert(i);
                    }
                }

                if new_live_in != node.live_in {
                    node.live_in = new_live_in;
                    *changed = true;
                }
            }

            visit(0usize.into(), &mut visited, &mut nodes, &mut changed, code);

            if !changed {
                break;
            }
        }
        let mut graph = Self::new(func.regs_used);

        // building the graph
        for (i, node) in nodes.iter().enumerate() {
            for g in node.live_in.iter().combinations(2) {
                graph.add_edge(**g[0], **g[1])
            }

            let mut out = AHashSet::new();
            for succ in nodes[i].opcode.get_successors(i.into(), nodes.len()) {
                for reg in &nodes[*succ as usize].live_in {
                    out.insert(*reg);
                }
            }
            println!(
                "{}: IN{{{}}}, OUT{{{}}}, DEF[{}], USE[{}]",
                i,
                node.live_in.iter().map(|v| format!("{}", v)).join(", "),
                out.iter().map(|v| format!("{}", v)).join(", "),
                node.opcode
                    .get_write(code)
                    .iter()
                    .map(|v| format!("{}", v))
                    .join(", "),
                node.opcode
                    .get_read(code)
                    .iter()
                    .map(|v| format!("{}", v))
                    .join(", ")
            );

            for a in node.opcode.get_write(code).iter() {
                for b in &out {
                    graph.add_edge(**a, **b)
                }
            }
        }

        // for (_, capture_reg) in &func.capture_regs {
        //     key_regs[*capture_reg] = true;
        // }

        // println!("zuzu {:?}", key_regs);
        // for i in 0..func.regs_used {
        //     // for &j in &func.ref_arg_regs {
        //     //     graph.add_edge(i, j)
        //     // }
        //     for j in (i + 1)..func.regs_used {
        //         if key_regs[i] != key_regs[j] {
        //             graph.add_edge(i, j)
        //         }
        //     }
        // }

        for ((_, r1), (_, r2)) in func
            .captured_regs
            .iter()
            .cartesian_product(func.captured_regs.iter())
        {
            graph.add_edge(**r1, **r2)
        }

        // for r1 in func
        //     .capture_regs
        //     .iter()
        //     .map(|(_, r)| r)
        //     .chain(&func.arg_regs)
        //     .chain(&func.ref_arg_regs)
        //     .chain(
        //         func.inner_funcs
        //             .iter()
        //             .flat_map(|f| &b.functions[*f as usize].capture_regs)
        //             .map(|(r, _)| r),
        //     )
        // {
        //     for r2 in 0..func.regs_used {
        //         graph.add_edge(*r1, r2)
        //     }
        // }

        graph
    }
}

pub fn optimize(code: &mut UnoptBytecode, func: FuncID) -> bool {
    let graph = InterferenceGraph::from_func(&code.functions[*func as usize], &*code);
    let mut coloring = color_rlf(graph);

    println!("{func}");
    println!("{:?}", coloring);
    {
        let arg_amount = code.functions[*func as usize].args.len();
        if !coloring.iter().take(arg_amount).all_unique() {
            panic!("gunkle fish bitch dick")
        }
        let mut arg_swaps = AHashMap::new();
        for (arg_idx, color) in coloring.iter_mut().enumerate() {
            let r = *color;
            if arg_idx < arg_amount && r != arg_idx {
                arg_swaps.insert(r, arg_idx);
                arg_swaps.insert(arg_idx, r);
            }
            *color = arg_swaps.get(&r).copied().unwrap_or(r);
        }
    }

    println!("{:?}\n==================\n\n", coloring);

    let mut changed = false;

    let mut call_expr_changes: AHashMap<CallExprID, CallExprID> = AHashMap::new();

    for opcode in code.functions[*func as usize].opcodes.iter() {
        #[allow(clippy::single_match)]
        match opcode.value {
            Opcode::Call { call, .. } => {
                let mut call_expr = code.call_exprs[*call as usize].clone();
                let mut expr_changed = false;
                for reg in call_expr
                    .dest
                    .iter_mut()
                    .chain(call_expr.positional.iter_mut().map(|(r, _)| r))
                    .chain(call_expr.named.iter_mut().map(|(_, r, _)| r))
                {
                    if *reg != Register(coloring[**reg]) {
                        *reg = Register(coloring[**reg]);
                        expr_changed = true;
                    }
                }

                changed |= expr_changed;

                if expr_changed {
                    code.call_exprs.push(call_expr);
                    call_expr_changes.insert(call, (code.call_exprs.len() - 1).into());
                }
            },
            _ => {},
        }
    }

    for opcode in code.functions[*func as usize].opcodes.iter_mut() {
        for reg in opcode.value.get_used_regs() {
            if *reg != Register(coloring[**reg]) {
                *reg = Register(coloring[**reg]);
                changed = true;
            }
        }
        #[allow(clippy::single_match)]
        match &mut opcode.value {
            Opcode::Call { call, .. } => *call = *call_expr_changes.get(call).unwrap_or(call),
            _ => {},
        }
    }
    for (_, reg) in code.functions[*func as usize].captured_regs.iter_mut() {
        *reg = Register(coloring[**reg]);
    }

    // TODO: FUNC ARGS AND CALL EXPR

    // for reg in &mut code.functions[*func as usize].ref_arg_regs {
    //     *reg = coloring[*reg];
    // }
    // for reg in &mut code.functions[*func as usize].arg_regs {
    //     // dbg!(&reg);
    //     *reg = coloring[*reg];
    // }

    for &child in code.functions[*func as usize].child_funcs.clone().iter() {
        println!("{func} -> {child}");
        let f = &mut code.functions[*child as usize];
        println!("ehooggy {:?}", f.captured_regs);
        for (reg, _) in f.captured_regs.iter_mut() {
            println!("uhuy {} {}", **reg, coloring[**reg]);
            *reg = Register(coloring[**reg]);
            changed = true;
        }
    }
    code.functions[*func as usize].regs_used = coloring.iter().unique().count();

    changed
}

impl ColorableGraph for InterferenceGraph {
    fn num_vertices(&self) -> usize {
        self.adjacency_map.len()
    }

    fn neighbors(&self, vi: usize) -> &[usize] {
        &self.adjacency_map[vi]
    }
}

// use ahash::{AHashMap, AHashSet};
// use heuristic_graph_coloring::{color_rlf, ColorableGraph};
// use itertools::Itertools;

// use crate::compiling::bytecode::{Function, Register, UnoptBytecode, UnoptFunction, UnoptRegister};
// use crate::compiling::opcodes::{CallExprID, FuncID, Opcode, OpcodePos, UnoptOpcode};

// enum NodeType<'a> {
//     Opcode(&'a UnoptOpcode),
//     Ending,
// }

// struct Node<'a> {
//     typ: NodeType<'a>,
//     live_in: AHashSet<UnoptRegister>,
// }

// impl<'a> Node<'a> {
//     pub fn get_successors(&self, idx: usize, len: usize) -> Vec<OpcodePos> {
//         match self.typ {
//             NodeType::Opcode(o) => o.get_successors(idx, len),
//             NodeType::Ending => vec![],
//         }
//     }

//     pub fn get_read(&self, code: &UnoptBytecode, func: FuncID) -> Vec<UnoptRegister> {
//         match self.typ {
//             NodeType::Opcode(o) => o.get_read(code),
//             NodeType::Ending => code.functions[*func as usize].child_funcs.iter().flat_map(|c| code.functions[**c as usize].captured_regs.iter().map(|(a, b)|)),
//         }
//     }
// }

// pub struct InterferenceGraph {
//     pub adjacency_map: Vec<Vec<usize>>,
// }

// impl InterferenceGraph {
//     fn new(capacity: usize) -> Self {
//         InterferenceGraph {
//             adjacency_map: vec![vec![]; capacity],
//         }
//     }

//     fn add_edge(&mut self, var1: usize, var2: usize) {
//         if var1 != var2 && !self.adjacency_map[var1].contains(&var2) {
//             self.adjacency_map[var1].push(var2);
//             self.adjacency_map[var2].push(var1);
//         }
//     }

//     pub fn from_func(func_id: FuncID, code: &UnoptBytecode) -> Self {
//         let func = &code.functions[*func as usize];
//         let mut nodes = func
//             .opcodes
//             .iter()
//             .map(|r| Node {
//                 typ: NodeType::Opcode(r),
//                 live_in: AHashSet::new(),
//             })
//             .collect_vec();

//         // nodes.push(Nod)

//         loop {
//             let mut changed = false;

//             let mut visited = AHashSet::new();

//             fn visit(
//                 node_idx: OpcodePos,
//                 visited: &mut AHashSet<OpcodePos>,
//                 nodes: &mut Vec<Node>,
//                 changed: &mut bool,
//                 code: &UnoptBytecode,
//             ) {
//                 // depth-first post-order traversal
//                 visited.insert(node_idx);
//                 let successors =
//                     nodes[*node_idx as usize].get_successors(node_idx.into(), nodes.len());
//                 for succ in successors.iter() {
//                     if !visited.contains(succ) {
//                         visit(*succ, visited, nodes, changed, code);
//                     }
//                 }

//                 // `LiveOut` set
//                 let mut out = AHashSet::new();
//                 for succ in successors {
//                     for reg in &nodes[*succ as usize].live_in {
//                         out.insert(*reg);
//                     }
//                 }

//                 let node = &mut nodes[*node_idx as usize];

//                 let mut new_live_in: AHashSet<_> =
//                     node.opcode.get_read(code).iter().copied().collect();

//                 let write = node.opcode.get_write(code);
//                 for i in out {
//                     // compute difference of sets
//                     if !write.contains(&i) {
//                         new_live_in.insert(i);
//                     }
//                 }

//                 if new_live_in != node.live_in {
//                     node.live_in = new_live_in;
//                     *changed = true;
//                 }
//             }

//             visit(0usize.into(), &mut visited, &mut nodes, &mut changed, code);

//             if !changed {
//                 break;
//             }
//         }
//         let mut graph = Self::new(func.regs_used);

//         // building the graph
//         for (i, node) in nodes.iter().enumerate() {
//             for g in node.live_in.iter().combinations(2) {
//                 graph.add_edge(**g[0], **g[1])
//             }

//             let mut out = AHashSet::new();
//             for succ in nodes[i].opcode.get_successors(i.into(), nodes.len()) {
//                 for reg in &nodes[*succ as usize].live_in {
//                     out.insert(*reg);
//                 }
//             }
//             println!(
//                 "{}: IN{{{}}}, OUT{{{}}}, DEF[{}], USE[{}]",
//                 i,
//                 node.live_in.iter().map(|v| format!("{}", v)).join(", "),
//                 out.iter().map(|v| format!("{}", v)).join(", "),
//                 node.opcode
//                     .get_write(code)
//                     .iter()
//                     .map(|v| format!("{}", v))
//                     .join(", "),
//                 node.opcode
//                     .get_read(code)
//                     .iter()
//                     .map(|v| format!("{}", v))
//                     .join(", ")
//             );

//             for a in node.opcode.get_write(code).iter() {
//                 for b in &out {
//                     graph.add_edge(**a, **b)
//                 }
//             }
//         }

//         // for (_, capture_reg) in &func.capture_regs {
//         //     key_regs[*capture_reg] = true;
//         // }

//         // println!("zuzu {:?}", key_regs);
//         // for i in 0..func.regs_used {
//         //     // for &j in &func.ref_arg_regs {
//         //     //     graph.add_edge(i, j)
//         //     // }
//         //     for j in (i + 1)..func.regs_used {
//         //         if key_regs[i] != key_regs[j] {
//         //             graph.add_edge(i, j)
//         //         }
//         //     }
//         // }

//         for ((_, r1), (_, r2)) in func
//             .captured_regs
//             .iter()
//             .cartesian_product(func.captured_regs.iter())
//         {
//             graph.add_edge(**r1, **r2)
//         }

//         // for r1 in func
//         //     .capture_regs
//         //     .iter()
//         //     .map(|(_, r)| r)
//         //     .chain(&func.arg_regs)
//         //     .chain(&func.ref_arg_regs)
//         //     .chain(
//         //         func.inner_funcs
//         //             .iter()
//         //             .flat_map(|f| &b.functions[*f as usize].capture_regs)
//         //             .map(|(r, _)| r),
//         //     )
//         // {
//         //     for r2 in 0..func.regs_used {
//         //         graph.add_edge(*r1, r2)
//         //     }
//         // }

//         graph
//     }
// }

// pub fn optimize(code: &mut UnoptBytecode, func: FuncID) -> bool {
//     let graph = InterferenceGraph::from_func(func, &*code);
//     let mut coloring = color_rlf(graph);

//     println!("{func}");
//     println!("{:?}", coloring);
//     {
//         let arg_amount = code.functions[*func as usize].args.len();
//         if !coloring.iter().take(arg_amount).all_unique() {
//             panic!("gunkle fish bitch dick")
//         }
//         let mut arg_swaps = AHashMap::new();
//         for (arg_idx, color) in coloring.iter_mut().enumerate() {
//             let r = *color;
//             if arg_idx < arg_amount && r != arg_idx {
//                 arg_swaps.insert(r, arg_idx);
//                 arg_swaps.insert(arg_idx, r);
//             }
//             *color = arg_swaps.get(&r).copied().unwrap_or(r);
//         }
//     }

//     println!("{:?}\n==================\n\n", coloring);

//     let mut changed = false;

//     let mut call_expr_changes: AHashMap<CallExprID, CallExprID> = AHashMap::new();

//     for opcode in code.functions[*func as usize].opcodes.iter() {
//         #[allow(clippy::single_match)]
//         match opcode.value {
//             Opcode::Call { call, .. } => {
//                 let mut call_expr = code.call_exprs[*call as usize].clone();
//                 let mut expr_changed = false;
//                 for reg in call_expr
//                     .dest
//                     .iter_mut()
//                     .chain(call_expr.positional.iter_mut().map(|(r, _)| r))
//                     .chain(call_expr.named.iter_mut().map(|(_, r, _)| r))
//                 {
//                     if *reg != Register(coloring[**reg]) {
//                         *reg = Register(coloring[**reg]);
//                         expr_changed = true;
//                     }
//                 }

//                 changed |= expr_changed;

//                 if expr_changed {
//                     code.call_exprs.push(call_expr);
//                     call_expr_changes.insert(call, (code.call_exprs.len() - 1).into());
//                 }
//             },
//             _ => {},
//         }
//     }

//     for opcode in code.functions[*func as usize].opcodes.iter_mut() {
//         for reg in opcode.value.get_used_regs() {
//             if *reg != Register(coloring[**reg]) {
//                 *reg = Register(coloring[**reg]);
//                 changed = true;
//             }
//         }
//         #[allow(clippy::single_match)]
//         match &mut opcode.value {
//             Opcode::Call { call, .. } => *call = *call_expr_changes.get(call).unwrap_or(call),
//             _ => {},
//         }
//     }
//     for (_, reg) in code.functions[*func as usize].captured_regs.iter_mut() {
//         *reg = Register(coloring[**reg]);
//     }

//     // TODO: FUNC ARGS AND CALL EXPR

//     // for reg in &mut code.functions[*func as usize].ref_arg_regs {
//     //     *reg = coloring[*reg];
//     // }
//     // for reg in &mut code.functions[*func as usize].arg_regs {
//     //     // dbg!(&reg);
//     //     *reg = coloring[*reg];
//     // }

//     for &child in code.functions[*func as usize].child_funcs.clone().iter() {
//         println!("{func} -> {child}");
//         let f = &mut code.functions[*child as usize];
//         println!("ehooggy {:?}", f.captured_regs);
//         for (reg, _) in f.captured_regs.iter_mut() {
//             println!("uhuy {} {}", **reg, coloring[**reg]);
//             *reg = Register(coloring[**reg]);
//             changed = true;
//         }
//     }
//     code.functions[*func as usize].regs_used = coloring.iter().unique().count();

//     changed
// }

// impl ColorableGraph for InterferenceGraph {
//     fn num_vertices(&self) -> usize {
//         self.adjacency_map.len()
//     }

//     fn neighbors(&self, vi: usize) -> &[usize] {
//         &self.adjacency_map[vi]
//     }
// }

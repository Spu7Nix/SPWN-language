use std::fmt::Display;

use ahash::AHashSet;
use heuristic_graph_coloring::{color_rlf, ColorableGraph};
use itertools::Itertools;

use crate::compiling::bytecode::{Bytecode, Function};
use crate::interpreting::opcodes::{Opcode, OpcodePos, UnoptOpcode, UnoptRegister};

struct Node<'a> {
    opcode: &'a Opcode<UnoptRegister>,
    live_in: AHashSet<usize>,
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

    pub fn from_func(func: &Function<UnoptRegister>, b: &Bytecode<UnoptRegister>) -> Self {
        let mut nodes: Vec<Node> = func
            .opcodes
            .iter()
            .map(|r| Node {
                opcode: r,
                live_in: AHashSet::new(),
            })
            .collect();

        loop {
            let mut changed = false;

            let mut visited = AHashSet::new();

            fn visit(
                node_idx: OpcodePos,
                visited: &mut AHashSet<OpcodePos>,
                nodes: &mut Vec<Node>,
                changed: &mut bool,
            ) {
                // depth-first post-order traversal
                visited.insert(node_idx);
                let successors = nodes[node_idx as usize]
                    .opcode
                    .get_successors(node_idx, nodes.len());
                for succ in successors.iter() {
                    if !visited.contains(succ) {
                        visit(*succ, visited, nodes, changed);
                    }
                }

                // `LiveOut` set
                let mut out = AHashSet::new();
                for succ in successors {
                    for reg in &nodes[succ as usize].live_in {
                        out.insert(*reg);
                    }
                }

                let node = &mut nodes[node_idx as usize];

                let mut new_live_in: AHashSet<_> = node.opcode.get_read().iter().copied().collect();

                let write = node.opcode.get_write();
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

            visit(0, &mut visited, &mut nodes, &mut changed);

            if !changed {
                break;
            }
        }

        let mut graph = Self::new(func.regs_used);

        // key change registers
        let mut key_regs = vec![false; func.regs_used];

        // building the graph
        for (i, node) in nodes.iter().enumerate() {
            for key_reg in node.opcode.get_key_change_regs() {
                key_regs[key_reg] = true;
            }

            for g in node.live_in.iter().combinations(2) {
                graph.add_edge(*g[0], *g[1])
            }

            let mut out = AHashSet::new();
            for succ in nodes[i].opcode.get_successors(i as OpcodePos, nodes.len()) {
                for reg in &nodes[succ as usize].live_in {
                    out.insert(*reg);
                }
            }
            // println!(
            //     "{}: IN{:?}, OUT{:?}, DEF{:?}, USE{:?}",
            //     i,
            //     node.live_in,
            //     out,
            //     node.opcode.get_write(),
            //     node.opcode.get_read()
            // );

            for a in node.opcode.get_write().iter() {
                for b in &out {
                    graph.add_edge(*a, *b)
                }
            }
        }

        // for (_, capture_reg) in &func.capture_regs {
        //     key_regs[*capture_reg] = true;
        // }

        // println!("zuzu {:?}", key_regs);
        for i in 0..func.regs_used {
            // for &j in &func.ref_arg_regs {
            //     graph.add_edge(i, j)
            // }
            for j in (i + 1)..func.regs_used {
                if key_regs[i] != key_regs[j] {
                    graph.add_edge(i, j)
                }
            }
        }
        // for &inner in &func.inner_funcs {
        //     for &(reg, _) in &b.functions[inner as usize].capture_regs {
        //         for j in 0..func.regs_used {
        //             graph.add_edge(reg, j)
        //         }
        //     }
        // }
        graph
    }
}

pub fn optimize(code: &mut Bytecode<UnoptRegister>, func: u16) -> bool {
    let graph = InterferenceGraph::from_func(&code.functions[func as usize], &*code);
    let coloring = color_rlf(graph);
    // println!("{:?}", coloring);

    let mut changed = false;

    for opcode in &mut code.functions[func as usize].opcodes {
        for reg in opcode.get_used_regs() {
            if *reg != coloring[*reg] {
                *reg = coloring[*reg];
                changed = true;
            }
        }
    }
    for (_, reg) in &mut code.functions[func as usize].capture_regs {
        *reg = coloring[*reg];
    }
    for reg in &mut code.functions[func as usize].ref_arg_regs {
        *reg = coloring[*reg];
    }
    for reg in &mut code.functions[func as usize].arg_regs {
        dbg!(&reg);
        *reg = coloring[*reg];
    }

    for inner in code.functions[func as usize].inner_funcs.clone() {
        let f = &mut code.functions[inner as usize];
        for (reg, _) in &mut f.capture_regs {
            *reg = coloring[*reg];
        }
    }
    code.functions[func as usize].regs_used = coloring.iter().unique().count();

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

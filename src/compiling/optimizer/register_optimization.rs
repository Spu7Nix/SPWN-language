use std::ops::Range;

use crate::compiling::bytecode::Function;
use crate::interpreting::opcodes::UnoptRegister;

fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && a.end > b.start
}

/*
An interference graph is a data structure used for determining the interference relationship between variables in a program.
It can identify which variables cannot be assigned to the same register simultaneously, thereby guiding register allocation.

Given the code below:
```
a = 2
b = 40
c = a
d = 42
c += b
```

It uses 3 registers, which within the interference graph looks like:
```
a ---- b
   __/ |
 /     |
c ---- d
```

- `a` interferes with `b` as it is required for `c` (assigned to `c`, where `c` is used in the plus equals operation).
- `c` interferes with `b` due to the plus equals operation.
- `b` interferes with `d` as ....
- `c` interferes with `d` as ....
Once a graph colouring algorithm is applied to the interference graph, the code would simplify to:
```
a = 2
b = 40
a += b
```

ALSO.,, our thing here doesnt work witgh VARIABLE names... / Instead just registeres.
*/
pub struct InterferenceGraph {
    adjacency_map: Vec<Vec<UnoptRegister>>,
}

impl InterferenceGraph {
    fn new(capacity: UnoptRegister) -> Self {
        InterferenceGraph {
            adjacency_map: vec![vec![]; capacity],
        }
    }

    fn add_edge(&mut self, var1: UnoptRegister, var2: UnoptRegister) {
        self.adjacency_map[var1].push(var2);
        self.adjacency_map[var2].push(var1);
    }

    fn get_neighbors(&self, var: UnoptRegister) -> &[UnoptRegister] {
        &self.adjacency_map[var]
    }
}

pub fn construct_graph(function: &mut Function<UnoptRegister>) -> InterferenceGraph {
    let mut graph = InterferenceGraph::new(function.regs_used);

    let mut lifetimes: Vec<Option<Range<usize>>> = vec![None; function.regs_used];

    for (i, opcode) in function.opcodes.iter().enumerate() {
        for reg in opcode.get_used_regs() {
            if reg == UnoptRegister::MAX {
                break;
            }
            let range = &mut lifetimes[reg];
            match range {
                Some(r) => {
                    if i > r.end {
                        r.end = i
                    }
                },
                None => *range = Some(i..i),
            }
        }
    }
    for (reg, lifetime) in lifetimes.iter().enumerate() {
        println!("R{}: {:?}", reg, lifetime)
    }

    for i in 0..(lifetimes.len() - 1) {
        for j in (i + 1)..lifetimes.len() {
            if let (Some(a), Some(b)) = (&lifetimes[i], &lifetimes[j]) {
                if ranges_overlap(a, b) {
                    graph.add_edge(i, j);
                }
            }
        }
    }

    println!("{:?}", graph.adjacency_map);

    graph
}

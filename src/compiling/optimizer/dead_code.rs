use ahash::AHashSet;

use crate::compiling::bytecode::Function;
use crate::interpreting::opcodes::{Opcode, OpcodePos, UnoptRegister};

pub fn optimize(func: &mut Function<UnoptRegister>) -> bool {
    let mut unreached =
        ((0 as OpcodePos)..(func.opcodes.len() as OpcodePos)).collect::<AHashSet<_>>();

    fn visit(
        node_idx: OpcodePos,
        unreached: &mut AHashSet<OpcodePos>,
        nodes: &mut Vec<Opcode<UnoptRegister>>,
    ) {
        // depth-first post-order traversal
        unreached.remove(&node_idx);
        let successors = nodes[node_idx as usize].get_successors(node_idx, nodes.len());
        for succ in successors.iter().rev() {
            if unreached.contains(succ) {
                visit(*succ, unreached, nodes);
            }
        }
    }

    visit(0, &mut unreached, &mut func.opcodes);

    func.remove_opcodes(&unreached);

    !unreached.is_empty()
}

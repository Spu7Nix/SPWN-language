use crate::builtin::{Group, Id, Item};
use crate::levelstring::ObjParam;
use crate::optimize::*;
use cached::proc_macro::cached;
use std::cmp::{max, min, Ordering};
use std::collections::{HashMap, HashSet};
// instant count algebra :pog:
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IcExpr {
    Or(Box<IcExpr>, Box<IcExpr>),
    And(Box<IcExpr>, Box<IcExpr>),
    True,
    False,
    Equals(Item, i32),
    MoreThan(Item, i32),
    LessThan(Item, i32),
}

impl IcExpr {
    fn get_variables(&self) -> HashSet<Item> {
        let mut variables = HashSet::new();
        match self {
            Self::Equals(item, _) | Self::MoreThan(item, _) | Self::LessThan(item, _) => {
                variables.insert(*item);
            }
            Self::And(expr1, expr2) | Self::Or(expr1, expr2) => {
                variables.extend(expr1.get_variables());
                variables.extend(expr2.get_variables());
            }
            _ => (),
        };
        variables
    }
    fn flatten_and(&self) -> Vec<Self> {
        match &self {
            &Self::And(a, b) => a.flatten_and().into_iter().chain(b.flatten_and()).collect(),
            a => {
                vec![(*a).clone()]
            }
        }
    }

    fn flatten_or(&self) -> Vec<Self> {
        match &self {
            &Self::Or(a, b) => a.flatten_or().into_iter().chain(b.flatten_or()).collect(),
            a => {
                vec![(*a).clone()]
            }
        }
    }
    fn stack_and(mut iter: impl Iterator<Item = Self>) -> Self {
        let mut out = iter.next().unwrap();
        for expr in iter {
            out = Self::And(out.into(), expr.into());
        }
        out
    }
    fn stack_or(mut iter: impl Iterator<Item = Self>) -> Self {
        let mut out = iter.next().unwrap();
        for expr in iter {
            out = Self::Or(out.into(), expr.into());
        }
        out
    }

    // fn remove_duplicates(&self) -> Self {
    //     let list = match self {
    //         Self::And(_, _) => self.flatten_and(),
    //         Self::Or(_, _) => self.flatten_or(),
    //         a => return a.clone(),
    //     };
    //     let mut critical_value_sets = HashMap::new();
    //     for el in &list {
    //         get_critical_value_sets(el, &mut critical_value_sets);
    //     }
    //     let inputs = enumerate_truth_table_inputs(&critical_value_sets);

    //     let mut truthtables = HashMap::new();
    //     for el in list {
    //         let tt = get_truth_table(self, &inputs);
    //         if let Some(expr) = truthtables.get_mut(&tt) {
    //             if get_complexity(expr).1 > get_complexity(&el).1 {
    //                 *expr = el;
    //             }
    //         } else {
    //             truthtables.insert(tt, el);
    //         }
    //     }

    //     Self::stack_and(truthtables.values().cloned())
    // }
    fn remove_duplicates(&self) -> Self {
        match self {
            Self::And(_, _) => {
                let list = self.flatten_and();
                let set: HashSet<_> = list.iter().collect();
                let mut set_iter = set.iter().cloned();

                let mut out = (*set_iter.next().unwrap()).remove_duplicates();
                for expr in set_iter {
                    out = Self::And(out.into(), expr.remove_duplicates().into());
                }
                out
            }
            Self::Or(_, _) => {
                let list = self.flatten_or();
                let set: HashSet<_> = list.iter().collect();
                let mut set_iter = set.iter().cloned();

                let mut out = (*set_iter.next().unwrap()).remove_duplicates();
                for expr in set_iter {
                    out = Self::Or(out.into(), expr.remove_duplicates().into());
                }
                out
            }
            a => a.clone(),
        }
    }
    fn decrease_and(&self) -> Self {
        // remove duplicates before using this
        match self {
            Self::Or(a, b) => {
                if let Self::And(a1, b1) = *a.clone() {
                    if let Self::And(a2, b2) = *b.clone() {
                        let a1 = a1.decrease_and();
                        let b1 = b1.decrease_and();
                        let a2 = a2.decrease_and();
                        let b2 = b2.decrease_and();
                        if a1 == a2 {
                            Self::And(a1.into(), Self::Or(b1.into(), b2.into()).into())
                        } else if a1 == b2 {
                            Self::And(a1.into(), Self::Or(b1.into(), a2.into()).into())
                        } else if b1 == a2 {
                            Self::And(b1.into(), Self::Or(a1.into(), b2.into()).into())
                        } else if b1 == b2 {
                            Self::And(b1.into(), Self::Or(a1.into(), a2.into()).into())
                        } else {
                            Self::Or(a.decrease_and().into(), b.decrease_and().into())
                        }
                    } else {
                        Self::Or(a.decrease_and().into(), b.decrease_and().into())
                    }
                } else {
                    Self::Or(a.decrease_and().into(), b.decrease_and().into())
                }
            }
            Self::And(a, b) => Self::And(a.decrease_and().into(), b.decrease_and().into()),
            a => a.clone(),
        }
    }
}
#[cached]
fn equal_behaviour(first: IcExpr, other: IcExpr) -> bool {
    if first == other {
        return true;
    };
    let mut critical_value_sets = HashMap::new();
    get_critical_value_sets(&first, &mut critical_value_sets);
    get_critical_value_sets(&other, &mut critical_value_sets);
    let inputs = enumerate_truth_table_inputs(&critical_value_sets);

    let truth_table1 = get_truth_table(&first, &inputs);
    let truth_table2 = get_truth_table(&other, &inputs);
    truth_table1 == truth_table2
}

#[derive(Debug, Clone, Eq)]
struct HeapItem {
    complexity: u16,
    formula: IcExpr,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.complexity == other.complexity
    }
}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.complexity.cmp(&other.complexity))
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        //not the same as the original implementation, might have to change
        self.complexity.cmp(&other.complexity)
    }
}

#[derive(Debug, Clone, Eq)]
struct FullHeapItem {
    item: HeapItem,
    count: u32,
    iter: Combinations,
}

impl PartialEq for FullHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl PartialOrd for FullHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for FullHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.item.cmp(&other.item) {
            Ordering::Equal => self.count.cmp(&other.count),
            a => a,
        }
    }
}

impl HeapItem {
    fn new(formula: IcExpr) -> Self {
        Self {
            complexity: get_complexity(&formula),
            formula,
        }
    }
}

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::iter::FromIterator;

type CriticalValueSets = HashMap<Item, HashSet<i32>>;

// Returns a map from each variable to the set of values such that the formula
// might evaluate differently for variable = value-1 versus variable = value.
fn get_critical_value_sets(formula: &IcExpr, result: &mut CriticalValueSets) {
    let mut insert_to_result = |item, num: &i32| {
        if let Some(set) = result.get_mut(item) {
            set.insert(*num);
        } else {
            let mut new_set = HashSet::new();
            new_set.insert(*num);
            result.insert(*item, new_set);
        }
    };
    match formula {
        IcExpr::True | IcExpr::False => (),
        IcExpr::LessThan(item, num) => insert_to_result(item, num),
        IcExpr::Equals(item, num) => {
            insert_to_result(item, num);
            insert_to_result(item, &(*num + 1));
        }
        IcExpr::MoreThan(item, num) => {
            insert_to_result(item, &(*num + 1));
        }
        IcExpr::And(lhs, rhs) | IcExpr::Or(lhs, rhs) => {
            get_critical_value_sets(&**lhs, result); //ladies and gentlemen, the penis operator
            get_critical_value_sets(&**rhs, result);
        }
    };
}

fn get_solve_complexity(formula: &IcExpr) -> u16 {
    match formula {
        IcExpr::True | IcExpr::False => 0,
        IcExpr::LessThan(_, _) => 1,
        IcExpr::Equals(_, _) => 2,
        IcExpr::MoreThan(_, _) => 1,
        IcExpr::And(lhs, rhs) | IcExpr::Or(lhs, rhs) => {
            get_solve_complexity(&**lhs) + get_solve_complexity(&**rhs)
        }
    }
}

// Returns a list of inputs sufficient to compare Boolean combinations of the
// primitives returned by enumerate_useful_primitives.
fn enumerate_truth_table_inputs(
    critical_value_sets: &CriticalValueSets,
) -> Vec<HashMap<Item, i32>> {
    use itertools::Itertools;

    let value_sets = critical_value_sets.values();

    let product = value_sets
        .map(|value_set| {
            let mut new_set = value_set.clone();
            new_set.insert(i32::MIN);
            new_set.iter().copied().collect::<Vec<i32>>()
        })
        .multi_cartesian_product();

    product
        .map(|values| {
            let mut dict = HashMap::new();
            let mut values_iter = values.iter();
            for variable in critical_value_sets.keys() {
                dict.insert(*variable, *values_iter.next().unwrap());
            }
            dict
        })
        .collect()

    // def enumerate_truth_table_inputs(critical_value_sets):
    //     variables, value_sets = zip(*critical_value_sets.items())
    //     return [
    //         dict(zip(variables, values))
    //         for values in product(*({-inf} | value_set for value_set in value_sets))
    //     ]
}

// Returns both constants and all single comparisons whose critical value set is
// a subset of the given ones.
fn enumerate_useful_primitives(critical_value_sets: &CriticalValueSets) -> Vec<IcExpr> {
    let mut out = vec![IcExpr::True, IcExpr::False];

    for (variable, value_set) in critical_value_sets.iter() {
        for value in value_set {
            out.push(IcExpr::LessThan(*variable, *value));
            if value_set.get(&(value + 1)).is_some() {
                out.push(IcExpr::Equals(*variable, *value));
            }
            out.push(IcExpr::MoreThan(*variable, *value - 1));
        }
    }
    out
}

// Evaluates the formula recursively on the given input.
fn evaluate(formula: &IcExpr, input: &HashMap<Item, i32>) -> bool {
    match formula {
        IcExpr::True => true,
        IcExpr::False => false,
        IcExpr::LessThan(item, num) => input[item] < *num,
        IcExpr::Equals(item, num) => input[item] == *num,
        IcExpr::MoreThan(item, num) => input[item] > *num,
        IcExpr::And(e1, e2) => evaluate(&**e1, input) && evaluate(&**e2, input),
        IcExpr::Or(e1, e2) => evaluate(&**e1, input) || evaluate(&**e2, input),
    }
}
//Evaluates the formula on the many inputs, packing the values into an integer.
fn get_truth_table(formula: &IcExpr, inputs: &[HashMap<Item, i32>]) -> u64 {
    let mut truth_table = 0;
    //println!("{}", inputs.len());
    for input in inputs {
        truth_table = (truth_table << 1) + evaluate(formula, input) as u64;
    }
    truth_table
}

// Returns the number of Ands.
pub fn get_complexity(formula: &IcExpr) -> u16 {
    match formula {
        IcExpr::True | IcExpr::False => 0,
        IcExpr::LessThan(_, _) | IcExpr::MoreThan(_, _) | IcExpr::Equals(_, _) => 0,
        IcExpr::And(lhs, rhs) => {
            let ands_lhs = get_complexity(&**lhs);
            let ands_rhs = get_complexity(&**rhs);
            ands_lhs + 1 + ands_rhs
        }
        IcExpr::Or(lhs, rhs) => {
            let ands_lhs = get_complexity(&**lhs);
            let ands_rhs = get_complexity(&**rhs);
            ands_lhs + ands_rhs
        }
    }
}

//#[derive(Debug)]
struct Merge {
    heap: BinaryHeap<Reverse<FullHeapItem>>,
    iter_count: u32,
}

impl Merge {
    fn update(&mut self, mut iter: Combinations) {
        if let Some(value) = iter.next() {
            self.heap.push(Reverse(FullHeapItem {
                item: value,
                count: self.iter_count,
                iter,
            }));
            self.iter_count += 1;
        }
    }
    fn push(&mut self, item: HeapItem) {
        self.heap.push(Reverse(FullHeapItem {
            item,
            count: self.iter_count,
            iter: Combinations {
                or: false,
                formula: IcExpr::False,
                best_formulas: Vec::new(),
                i: 0,
            },
        }));
        self.iter_count += 1;
    }

    fn next(&mut self) -> Option<HeapItem> {
        if self.heap.is_empty() {
            return None;
        }
        let mut item = self.heap.pop().unwrap().0;
        if let Some(next_value) = item.iter.next() {
            self.heap.push(Reverse(FullHeapItem {
                item: next_value,
                ..item.clone()
            }))
        }
        Some(item.item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Combinations {
    or: bool, // true: or, false: and
    formula: IcExpr,
    best_formulas: Vec<IcExpr>,
    i: usize,
}

impl Combinations {
    fn next(&mut self) -> Option<HeapItem> {
        if self.i >= self.best_formulas.len() {
            return None;
        }
        let formula = if self.or {
            IcExpr::Or(
                self.formula.clone().into(),
                self.best_formulas[self.i].clone().into(),
            )
        } else {
            IcExpr::And(
                self.formula.clone().into(),
                self.best_formulas[self.i].clone().into(),
            )
        };
        self.i += 1;
        Some(HeapItem::new(formula))
    }
}

pub fn simplify_ic_expr_full(target_formula: IcExpr) -> IcExpr {
    let mut critical_value_sets = HashMap::new();
    get_critical_value_sets(&target_formula, &mut critical_value_sets);
    let inputs = enumerate_truth_table_inputs(&critical_value_sets);
    let target_truth_table = get_truth_table(&target_formula, &inputs);
    let mut best = HashMap::<u64, IcExpr>::new();
    let mut merge = Merge {
        heap: BinaryHeap::new(),
        iter_count: 0,
    };
    for formula in enumerate_useful_primitives(&critical_value_sets) {
        merge.push(HeapItem::new(formula));
    }
    let mut best_formulas = Vec::new();

    loop {
        if let Some(out) = best.get(&target_truth_table) {
            return out.clone();
        }
        let item = match merge.next() {
            Some(item) => item,
            None => unreachable!(),
        };
        let formula = item.formula;
        let truth_table = get_truth_table(&formula, &inputs);
        if best.get(&truth_table).is_some() {
            continue;
        }
        merge.update(Combinations {
            or: false,
            formula: formula.clone(),
            best_formulas: best_formulas.clone(),
            i: 0,
        });
        merge.update(Combinations {
            or: true,
            formula: formula.clone(),
            best_formulas: best_formulas.clone(),
            i: 0,
        });
        best.insert(truth_table, formula.clone());
        best_formulas.push(formula);
    }

    // while best.get(&target_truth_table).is_none() {
    //     println!("{}", heap.len());
    //     let formula = heap.pop().unwrap().0.formula;
    //     let truth_table = get_truth_table(&formula, &inputs);
    //     if best.get(&truth_table).is_some() {
    //         continue;
    //     }

    //     for other_formula in best.values() {
    //         heap.push(Reverse(HeapItem::new(IcExpr::And(
    //             formula.clone().into(),
    //             other_formula.clone().into(),
    //         ))));
    //         heap.push(Reverse(HeapItem::new(IcExpr::Or(
    //             formula.clone().into(),
    //             other_formula.clone().into(),
    //         ))));
    //     }
    //     best.insert(truth_table, formula);
    // }
}

pub fn build_ic_connections(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    closed_group: &mut u16,
    mut connections: Vec<(Group, Group, IcExpr, Trigger)>,
) {
    if connections.is_empty() {
        return;
    }
    connections = {
        let mut new_connections = Vec::new();
        for (start, end, expr, trigger) in connections {
            for new_expr in expr.flatten_or() {
                new_connections.push((start, end, new_expr, trigger))
            }
        }
        new_connections
    };

    // println!(
    //     "connections: \n{}",
    //     connections
    //         .iter()
    //         .map(|(a, b, c, _)| format!("{:?}", (a, b, c)))
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // );

    let mut new_connections = Vec::new();
    for (start, end, expr, trigger) in &connections {
        match &expr {
            IcExpr::And(_, _) => new_connections.push((*start, *end, expr.clone(), *trigger)),
            _ => {
                build_instant_count_network(
                    network,
                    objects,
                    *start,
                    *end,
                    expr.clone(),
                    *trigger,
                    closed_group,
                );
            }
        }
    }

    if new_connections.is_empty() {
        return;
    }

    let connections = new_connections
        .iter()
        .map(|(s, e, expr, t)| (*s, *e, expr.flatten_and(), *t))
        .collect::<Vec<_>>();

    let mut nodes = Vec::<(IcExpr, u16)>::new();
    let mut costs = HashMap::new();

    use connection_combiner::{erase_blank, reduce, IoNode, Sets};

    let mut sets = Sets::new();
    let mut ref_trigger = connections[0].3;

    for (start, end, list, trigger) in connections {
        let mut new_list = HashSet::new();
        for node in list {
            let compl = get_complexity(&node) + 1;
            let mut found = false;
            for (i, (other, other_compl)) in nodes.iter_mut().enumerate() {
                if equal_behaviour(node.clone(), other.clone()) {
                    if compl < *other_compl {
                        *other = node.clone();
                        costs.insert(i, compl);
                    }
                    new_list.insert(i);

                    found = true;
                    break;
                }
            }
            if !found {
                new_list.insert(nodes.len());
                costs.insert(nodes.len(), compl);
                nodes.push((node, compl));
            }
        }
        sets.push((
            [IoNode::Input(start)].iter().copied().collect(),
            new_list,
            [IoNode::Output(end)].iter().copied().collect(),
            trigger,
        ))
    }

    // println!(
    //     "nodes:\n{}",
    //     nodes
    //         .iter()
    //         .enumerate()
    //         .map(|(i, (e, _))| format!("{}: {:?}", i, e))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    // );
    // println!(
    //     "costs:\n{}",
    //     costs
    //         .iter()
    //         .map(|a| format!("{:?}", a))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    // );

    let mut edges = Vec::new();
    let mut index = 0;
    // erase_blank(&mut sets, |a, b| {
    //     println!("connect {:?} to {:?}", a, b);
    //     edges.push((a, b));
    // });

    while !sets.is_empty() {
        reduce(&mut sets, &mut index, &costs, |a, b| {
            //println!("connect {:?} to {:?}", a, b);
            edges.push((a, b));
        });
    }

    let mut graph = HashMap::<IoNode, HashSet<IoNode>>::new();
    for (start, end) in edges {
        // if let IoNode::Color(_, _) = start {
        //     // select new group
        //     (*closed_group) += 1;
        //     let new_group = Group {
        //         id: Id::Arbitrary(*closed_group),
        //     };
        //     color_node_targets.insert(start, new_group);
        // };
        if let Some(list) = graph.get_mut(&start) {
            list.insert(end);
        } else {
            graph.insert(start, [end].iter().copied().collect());
        }
    }

    let mut compressed: Vec<(HashSet<IoNode>, HashSet<IoNode>)> = Vec::new();
    for (node, set) in graph {
        let mut added = false;
        for el in &mut compressed {
            if el.1 == set {
                (*el).0.insert(node);
                added = true;
                break;
            }
        }
        if !added {
            compressed.push(([node].iter().cloned().collect(), set))
        }
    }

    let mut color_node_targets: HashMap<IoNode, Group> = HashMap::new();

    for (starts, _) in &compressed {
        (*closed_group) += 1;
        let new_group = Group {
            id: Id::Arbitrary(*closed_group),
        };
        for start in starts {
            assert_eq!(color_node_targets.insert(*start, new_group), None);
        }
    }

    //println!("{:?}", color_node_targets);

    for (starts, list) in compressed {
        for start in starts {
            let g = match start {
                IoNode::Input(g) => g,
                IoNode::Output(_) => unreachable!(),
                IoNode::Color(col, i, t) => {
                    ref_trigger = t;
                    color_node_targets[&IoNode::Color(col, i, t)]
                }
            };
            for connection in &list {
                match *connection {
                    IoNode::Input(_) => unreachable!(),
                    IoNode::Output(g2) => {
                        //println!("{:?} -> {:?}", g, g2);
                        build_instant_count_network(
                            network,
                            objects,
                            g,
                            g2,
                            IcExpr::True,
                            ref_trigger,
                            closed_group,
                        );
                    }
                    IoNode::Color(col, i, ref_trigger) => {
                        let target = color_node_targets[&IoNode::Color(col, i, ref_trigger)];

                        build_ic_connections(
                            network,
                            objects,
                            closed_group,
                            vec![(g, target, nodes[col].0.clone(), ref_trigger)],
                        )
                    }
                }
            }
        }
    }

    // for (start, end, expr, ref_trigger) in connections {
    //     build_instant_count_network(
    //         network,
    //         objects,
    //         Some(start),
    //         end,
    //         expr,
    //         ref_trigger,
    //         closed_group,
    //     );
    // }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum MatchingIoGroup {
//     End(Group),
//     Start(Group),
// }
// use MatchingIoGroup::*;

// #[derive(Debug, Clone)]
// struct ExprFrequency {
//     expr: IcExpr,
//     count: u16,
//     ref_trigger: Trigger,
//     complexity: u16,
//     appears_in: HashSet<usize>, // connections it appears in
// }
// //(IcExpr, u16, Trigger, u16)
// let mut equal_io = HashMap::<MatchingIoGroup, Vec<ExprFrequency>>::new();

// for (i, (start, end, expr, trigger)) in connections.iter().enumerate() {
//     let sub_exprs = expr.flatten_and();
//     // starts
//     match equal_io.get_mut(&Start(*start)) {
//         Some(list) => {
//             for e in sub_exprs.clone() {
//                 let mut added = false;

//                 for el in list.iter_mut() {
//                     if equal_behaviour(el.expr.clone(), e.clone()) {
//                         let complexity = get_complexity(&e).1;
//                         if complexity < el.complexity {
//                             (*el).expr = e.clone();
//                             (*el).complexity = complexity;
//                             (*el).ref_trigger = *trigger;
//                         }
//                         (*el).appears_in.insert(i);
//                         (*el).count += 1;
//                         added = true;
//                         break;
//                     }
//                 }
//                 if !added {
//                     list.push(ExprFrequency {
//                         expr: e.clone(),
//                         count: 1,
//                         ref_trigger: *trigger,
//                         complexity: get_complexity(&e).1,
//                         appears_in: [i].iter().copied().collect(),
//                     });
//                 }
//             }
//         }

//         None => {
//             equal_io.insert(
//                 Start(*start),
//                 sub_exprs
//                     .clone()
//                     .iter()
//                     .map(|e| ExprFrequency {
//                         expr: e.clone(),
//                         count: 1,
//                         ref_trigger: *trigger,
//                         complexity: get_complexity(&e).1,
//                         appears_in: [i].iter().copied().collect(),
//                     })
//                     .collect(), //vec![(e.clone(), 1, *end, *trigger, get_complexity(&e).1)],
//             );
//         }
//     }
//     //ends
//     match equal_io.get_mut(&End(*end)) {
//         Some(list) => {
//             for e in sub_exprs {
//                 let mut added = false;

//                 for el in list.iter_mut() {
//                     if equal_behaviour(el.expr.clone(), e.clone()) {
//                         let complexity = get_complexity(&e).1;
//                         if complexity < el.complexity {
//                             (*el).expr = e.clone();
//                             (*el).complexity = complexity;
//                             (*el).ref_trigger = *trigger;
//                         }
//                         (*el).count += 1;
//                         (*el).appears_in.insert(i);
//                         added = true;
//                         break;
//                     }
//                 }
//                 if !added {
//                     list.push(ExprFrequency {
//                         expr: e.clone(),
//                         count: 1,
//                         ref_trigger: *trigger,
//                         complexity: get_complexity(&e).1,
//                         appears_in: [i].iter().copied().collect(),
//                     });
//                 }
//             }
//         }

//         None => {
//             equal_io.insert(
//                 End(*end),
//                 sub_exprs
//                     .iter()
//                     .map(|e| ExprFrequency {
//                         expr: e.clone(),
//                         count: 1,
//                         ref_trigger: *trigger,
//                         complexity: get_complexity(&e).1,
//                         appears_in: [i].iter().copied().collect(),
//                     })
//                     .collect(), //vec![(e.clone(), 1, *end, *trigger, get_complexity(&e).1)],
//             );
//         }
//     }
// }

// let (io, first_or_last_in_chain) = equal_io
//     .iter()
//     .map(|(g, list)| list.iter().map(move |el| (*g, el)))
//     .flatten()
//     .max_by(|a, b| (a.1.count * (a.1.complexity + 1)).cmp(&(b.1.count * (b.1.complexity + 1))))
//     .unwrap();

// println!("io: {:?}", io);
// println!("first/last: {:?}", first_or_last_in_chain);

// (*closed_group) += 1;
// let middle_group = Group {
//     id: Id::Arbitrary(*closed_group),
// };
// let mut new_connections = Vec::new();

// match io {
//     Start(start) => new_connections.push((
//         start,
//         middle_group,
//         first_or_last_in_chain.expr.clone(),
//         first_or_last_in_chain.ref_trigger,
//     )),
//     End(end) => new_connections.push((
//         middle_group,
//         end,
//         first_or_last_in_chain.expr.clone(),
//         first_or_last_in_chain.ref_trigger,
//     )),
// };

// for connection in &connections {
//     let correct_group = match io {
//         Start(start) => connection.0 == start,
//         End(end) => connection.1 == end,
//     };
//     if correct_group {
//         let mut sub_exprs = connection.2.flatten_and();
//         if sub_exprs.len() == 1 {
//             new_connections.push(connection.clone());
//             continue;
//         }
//         match sub_exprs.iter().position(|sub_expr| {
//             equal_behaviour(sub_expr.clone(), first_or_last_in_chain.expr.clone())
//         }) {
//             Some(pos) => {
//                 sub_exprs.swap_remove(pos);

//                 let mut new_connection = connection.clone();
//                 match io {
//                     Start(_) => new_connection.0 = middle_group,
//                     End(_) => new_connection.1 = middle_group,
//                 };

//                 new_connection.2 = IcExpr::stack_and(sub_exprs.iter().cloned());
//                 new_connections.push(new_connection);
//             }
//             None => {
//                 new_connections.push(connection.clone());
//             }
//         }
//     } else {
//         new_connections.push(connection.clone());
//     }
// }

// build_ic_connections(network, objects, closed_group, new_connections);

pub fn build_instant_count_network<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start_group: Group,
    target: Group,
    expr: IcExpr,
    reference_trigger: Trigger,
    closed_group: &mut u16,
) -> bool {
    match expr {
        IcExpr::Equals(item, num) | IcExpr::MoreThan(item, num) | IcExpr::LessThan(item, num) => {
            create_instant_count_trigger(
                reference_trigger,
                target,
                start_group,
                match expr {
                    IcExpr::Equals(_, _) => 0,
                    IcExpr::MoreThan(_, _) => 1,
                    IcExpr::LessThan(_, _) => 2,
                    _ => unreachable!(),
                },
                num,
                item,
                objects,
                network,
                (false, false),
            );
            true
        }

        IcExpr::True => {
            // This can be optimized
            // if let Some(gang) = network.get(&target) {
            //     if gang.connections_in > 1 {

            create_spawn_trigger(
                reference_trigger,
                target,
                start_group,
                0.0,
                objects,
                network,
                (false, false),
            );
            //     } else {
            //         replace_group(target, start_group, network, objects);
            //     }
            // } else {
            //     unreachable!()
            // }

            true
        }

        IcExpr::And(expr1, expr2) => {
            (*closed_group) += 1;
            let middle_group = Group {
                id: Id::Arbitrary(*closed_group),
            };
            if build_instant_count_network(
                network,
                objects,
                start_group,
                middle_group,
                *expr1,
                reference_trigger,
                closed_group,
            ) {
                build_instant_count_network(
                    network,
                    objects,
                    middle_group,
                    target,
                    *expr2,
                    reference_trigger,
                    closed_group,
                )
            } else {
                false
            }
        }

        IcExpr::Or(expr1, expr2) => {
            //unreachable!()
            let result1 = build_instant_count_network(
                network,
                objects,
                start_group,
                target,
                *expr1,
                reference_trigger,
                closed_group,
            );
            let result2 = build_instant_count_network(
                network,
                objects,
                start_group,
                target,
                *expr2,
                reference_trigger,
                closed_group,
            );
            result1 || result2
        }
        IcExpr::False => {
            // delete branch
            // let mut targets = vec![target];
            // while !targets.is_empty() {
            //     let mut new_targets = Vec::new();
            //     for target in targets {
            //         let gang = network.get_mut(&target).unwrap();
            //         if gang.connections_in > 1 {
            //             continue;
            //         }
            //         for trigger in &mut gang.triggers {
            //             (*trigger).deleted = true;
            //             if trigger.role != TriggerRole::Output {
            //                 if let Some(ObjParam::Bool(true)) =
            //                     &objects[trigger.obj].0.params.get(&56)
            //                 {
            //                     if let Some(ObjParam::Group(g)) =
            //                         &objects[trigger.obj].0.params.get(&51)
            //                     {
            //                         new_targets.push(*g);
            //                     }
            //                 }
            //             }
            //         }
            //     }
            //     targets = new_targets;
            // }
            false
        }
    }
}

// fn get_instant_count_network<'a>(
//     network: &'a mut TriggerNetwork,
//     objects: &'a mut Triggerlist,
//     start: (Group, usize),
//     origin_group: Group,
//     ignore_optimized: bool,
//     closed_group: &mut u16,
//     mut visited: HashSet<(Group, usize)>,
//     backwards: bool,
// ) -> Option<Vec<(Group, Group, IcExpr)>> {
//     //u32: delay in millis
//     let trigger = network.get(&start.0).unwrap().triggers[start.1];

//     if visited.contains(&start) {
//         if network[&start.0].triggers[start.1].deleted {
//             return Some(Vec::new());
//         } else {
//             return None;
//         }
//     }

//     visited.insert(start);
//     let start_obj = &objects[trigger.obj].0.params;

//     //println!("{}", network[&start.0].connections_in.len());
//     assert_eq!(start_obj.get(&1), Some(&ObjParam::Number(1811.0)));
//     // group in list is the end group
//     let list: Vec<(usize, Group)>;
//     if backwards {
//         list = network[&start.0]
//             .connections_in
//             .iter()
//             .map(|(a, b)| (*b, *a))
//             .collect();
//     } else if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
//         if let ID::Specific(_) = g.id {
//             (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
//             return None;
//         }

//         if let Some(gang) = network.get(g) {
//             list = vec![*g; gang.triggers.len()]
//                 .iter()
//                 .copied()
//                 .enumerate()
//                 .collect();
//         } else {
//             //dangeling

//             return Some(Vec::new());
//         }
//     } else {
//         //dangling

//         return Some(Vec::new());
//     }

//     if list.is_empty() {
//         return Some(Vec::new());
//     }
//     let start_item = if let ObjParam::Item(i) =
//         start_obj.get(&80).unwrap_or(&ObjParam::Item(Item {
//             id: ID::Specific(0),
//         })) {
//         *i
//     } else {
//         Item {
//             id: ID::Specific(0),
//         }
//     };
//     let start_num =
//         if let ObjParam::Number(a) = start_obj.get(&77).unwrap_or(&ObjParam::Number(0.0)) {
//             *a as i32
//         } else {
//             0
//         };
//     let start_expr = match start_obj.get(&88) {
//         Some(ObjParam::Number(1.0)) => IcExpr::MoreThan(start_item, start_num),
//         Some(ObjParam::Number(2.0)) => IcExpr::LessThan(start_item, start_num),
//         _ => IcExpr::Equals(start_item, start_num),
//     };

//     let mut out = HashSet::new();

//     for (i, g) in list {
//         let trigger_ptr = (g, i);
//         let trigger = network[&trigger_ptr.0].triggers[trigger_ptr.1];

//         //let full_trigger_ptr = (trigger_ptr.0, trigger_ptr.1, full_delay);
//         let target_out = (origin_group, trigger_ptr.0, start_expr.clone());

//         if trigger.optimized && !ignore_optimized {
//             if !trigger.deleted {
//                 out.insert(target_out);
//             }
//         } else if let TriggerRole::Operator = trigger.role {
//             if backwards {
//                 if let Some(children) = get_instant_count_network(
//                     network,
//                     objects,
//                     trigger_ptr,
//                     origin_group,
//                     ignore_optimized,
//                     closed_group,
//                     visited.clone(),
//                     true,
//                 ) {
//                     for el in children.iter().map(|(start_g, end_g, expr)| {
//                         (
//                             *start_g,
//                             *end_g,
//                             IcExpr::And(Box::from(start_expr.clone()), Box::from(expr.clone())),
//                         )
//                     }) {
//                         out.insert(el);
//                     }
//                 } else {
//                     (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted =
//                         false;
//                 }
//             } else {
//                 let forward = get_instant_count_network(
//                     network,
//                     objects,
//                     trigger_ptr,
//                     origin_group,
//                     ignore_optimized,
//                     closed_group,
//                     visited.clone(),
//                     false,
//                 );
//                 if network[&trigger_ptr.0].connections_in.len() > 1 {
//                     let mut incoming_exprs = HashSet::new();
//                     if let Some(children) = get_instant_count_network(
//                         network,
//                         objects,
//                         trigger_ptr,
//                         origin_group,
//                         ignore_optimized,
//                         closed_group,
//                         visited.clone(),
//                         true,
//                     ) {
//                         for el in children.iter().map(|(_, start_g, expr)| {
//                             (
//                                 *start_g,
//                                 IcExpr::And(Box::from(start_expr.clone()), Box::from(expr.clone())),
//                             )
//                         }) {
//                             incoming_exprs.insert(el);
//                         }
//                     } else {
//                         (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
//                             .deleted = false;
//                     }
//                 }
//             }

//             // (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = false;
//             // if optimize_from(network, objects, trigger_ptr, closed_group) {
//             //     out.insert(target_out);
//             // } else {
//             //     (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = true;
//             // }

//             if cross_point {
//             } else {
//                 out.extend(incoming_exprs)
//             }
//         } else if !backwards && optimize_from(network, objects, trigger_ptr, closed_group) {
//             (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = false;
//             out.insert(target_out);
//         }
//     }

//     (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = true;

//     Some(out.iter().cloned().collect())
// }

mod connection_combiner {
    use std::collections::{BTreeSet, HashMap, HashSet};

    use crate::{
        builtin::{Group, Id},
        optimize::Trigger,
    };
    type Color = usize;

    #[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
    pub enum IoNode {
        Input(Group),
        Output(Group),
        Color(Color, u16, Trigger),
    }

    impl std::fmt::Debug for IoNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Input(n) => f.write_str(&format!("input {:?}", n)),
                Self::Output(n) => f.write_str(&format!("output {:?}", n)),
                Self::Color(n, index, _) => f.write_str(&format!("{}_{}", n, index)),
            }
        }
    }

    pub type Sets = Vec<(BTreeSet<IoNode>, HashSet<Color>, BTreeSet<IoNode>, Trigger)>;

    fn combine(sets: &mut Sets) {
        let mut i = 0;
        while i < sets.len() {
            let mut j = i + 1;
            while j < sets.len() {
                if sets[j].1 == sets[i].1 && sets[j].2 == sets[i].2 {
                    for k in sets[j].0.clone() {
                        if !sets[i].0.contains(&k) {
                            sets[i].0.insert(k);
                        }
                    }
                    sets.remove(j);
                    j -= 1;
                }
                j += 1;
            }
            i += 1;
        }
        i = 0;
        while i < sets.len() {
            let mut j = i + 1;
            while j < sets.len() {
                if sets[j].1 == sets[i].1 && sets[j].0 == sets[i].0 {
                    for k in sets[j].2.clone() {
                        if !sets[i].2.contains(&k) {
                            sets[i].2.insert(k);
                        }
                    }
                    sets.remove(j);
                    j -= 1;
                }
                j += 1;
            }
            i += 1;
        }
    }
    fn push_input<F>(
        input: BTreeSet<IoNode>,
        point: Color,
        ref_trigger: Trigger,
        sets: &mut Sets,
        index: &mut u16,
        mut connect: F,
    ) where
        F: FnMut(IoNode, IoNode),
    {
        (*index) += 1;
        let current_index = *index;

        for i in &input {
            connect(*i, IoNode::Color(point, current_index, ref_trigger));
        }

        for set in sets.iter_mut() {
            if set.0 == input && set.1.contains(&point) {
                (*set).1.remove(&point);
                (*set).0 = [IoNode::Color(point, current_index, ref_trigger)]
                    .iter()
                    .copied()
                    .collect()
            }
        }
        erase_blank(sets, connect)
    }
    fn push_output<F>(
        output: BTreeSet<IoNode>,
        point: Color,
        ref_trigger: Trigger,
        sets: &mut Sets,
        index: &mut u16,
        mut connect: F,
    ) where
        F: FnMut(IoNode, IoNode),
    {
        (*index) += 1;
        let current_index = *index;

        for o in &output {
            connect(IoNode::Color(point, current_index, ref_trigger), *o);
        }
        for set in sets.iter_mut() {
            if set.2 == output && set.1.contains(&point) {
                (*set).1.remove(&point);
                (*set).2 = [IoNode::Color(point, current_index, ref_trigger)]
                    .iter()
                    .copied()
                    .collect();
            }
        }
        erase_blank(sets, connect)
    }
    pub fn erase_blank<F>(sets: &mut Sets, mut connect: F)
    where
        F: FnMut(IoNode, IoNode),
    {
        for set in sets.iter() {
            if set.1.is_empty() {
                for a in &set.0 {
                    for b in &set.2 {
                        connect(*a, *b);
                    }
                }
            }
        }

        sets.retain(|a| !a.1.is_empty())
    }

    pub fn reduce<F>(sets: &mut Sets, index: &mut u16, costs: &HashMap<Color, u16>, connect: F)
    where
        F: FnMut(IoNode, IoNode),
    {
        combine(sets);
        #[derive(Debug)]
        struct Score {
            inputs: HashMap<BTreeSet<IoNode>, u16>,
            outputs: HashMap<BTreeSet<IoNode>, u16>,
            ref_trigger: Trigger,
        }
        let mut scores = HashMap::<Color, Score>::new();

        for (input, nodes, output, ref_trigger) in sets.iter_mut() {
            for node in nodes.iter() {
                let score = if let Some(score) = scores.get_mut(node) {
                    score
                } else {
                    scores.insert(
                        *node,
                        Score {
                            inputs: HashMap::new(),
                            outputs: HashMap::new(),
                            ref_trigger: *ref_trigger,
                        },
                    );
                    scores.get_mut(node).unwrap()
                };

                if let Some(num) = score.inputs.get_mut(input) {
                    *num += 1;
                } else {
                    score.inputs.insert(input.clone(), 1);
                }
                if let Some(num) = score.outputs.get_mut(output) {
                    *num += 1;
                } else {
                    score.outputs.insert(output.clone(), 1);
                }
            }
        }

        let mut i_max = 0;
        let mut o_max = 0;
        let mut i_max_point = 0;
        let mut o_max_point = 0;
        let mut max_input = BTreeSet::new();
        let mut max_output = BTreeSet::new();
        let mut max_trigger = sets[0].3;

        for (node, score) in scores {
            let cost = costs[&node];
            for (input, mut num) in score.inputs {
                num *= cost;
                if num > i_max {
                    i_max = num;
                    i_max_point = node;
                    max_input = input;
                    max_trigger = score.ref_trigger;
                }
            }
            for (output, mut num) in score.outputs {
                num *= cost;
                if num > o_max {
                    o_max = num;
                    o_max_point = node;
                    max_output = output;
                    max_trigger = score.ref_trigger;
                }
            }
        }

        if o_max < i_max {
            push_input(max_input, i_max_point, max_trigger, sets, index, connect);
        } else {
            push_output(max_output, o_max_point, max_trigger, sets, index, connect);
        }
    }

    // #[test]
    // fn combining_algo() {
    //     let sets: &[(u16, &[char], u16)] = &[(1, &['A', 'B'], 1), (1, &['A'], 1), (1, &['B'], 1)];

    //     let costs = [('A', 1), ('B', 3), ('C', 2)];

    //     let costs = costs
    //         .iter()
    //         .map(|(k, v)| (*k as Color, *v))
    //         .collect::<HashMap<_, _>>();

    //     println!("initial sets:");

    //     let mut indices = HashMap::new();
    //     let mut real_sets: Sets = Vec::new();

    //     for set in sets {
    //         println!("{:?}", set);
    //         real_sets.push((
    //             [IoNode {
    //                 index: 0,
    //                 node: Node::Input(set.0),
    //             }]
    //             .iter()
    //             .copied()
    //             .collect(),
    //             set.1.iter().map(|c| *c as Color).collect(),
    //             [IoNode {
    //                 index: 0,
    //                 node: Node::Output(set.2),
    //             }]
    //             .iter()
    //             .copied()
    //             .collect(),
    //         ));
    //     }
    //     println!("-------------");
    //     let mut sets = real_sets;
    //     while !sets.is_empty() {
    //         reduce(&mut sets, &mut indices, &costs, |a, b| {
    //             println!("connect {:?} to {:?}", a, b)
    //         });
    //     }
    //     println!("-------------");
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ic_expr_simplify() {
        use crate::builtin::Id::*;
        let a = Item {
            id: Id::Specific(1),
        };
        let b = Item {
            id: Id::Specific(2),
        };
        // let c = Item {
        //     id: Id::Specific(3),
        // };
        use IcExpr::{Equals, LessThan, MoreThan};
        fn Or(e1: IcExpr, e2: IcExpr) -> IcExpr {
            IcExpr::Or(e1.into(), e2.into())
        }
        fn And(e1: IcExpr, e2: IcExpr) -> IcExpr {
            IcExpr::And(e1.into(), e2.into())
        }

        let expr = Or(
            And(
                And(
                    And(
                        And(
                            Equals(Item { id: Arbitrary(29) }, 0),
                            LessThan(Item { id: Specific(56) }, 3),
                        ),
                        MoreThan(Item { id: Arbitrary(27) }, 0),
                    ),
                    Equals(Item { id: Arbitrary(20) }, 1),
                ),
                Equals(Item { id: Specific(54) }, 0),
            ),
            And(
                And(
                    And(
                        And(
                            Equals(Item { id: Arbitrary(29) }, 0),
                            LessThan(Item { id: Specific(56) }, 3),
                        ),
                        LessThan(Item { id: Arbitrary(27) }, 0),
                    ),
                    Equals(Item { id: Arbitrary(20) }, 1),
                ),
                Equals(Item { id: Specific(54) }, 0),
            ),
        );

        println!("len: {:?}\n", get_solve_complexity(&expr));

        /*
        duplicates removed:
        Or(And(MoreThan(B, 2), Equals(C, 2)), And(LessThan(C, 2), MoreThan(B, 2)))

        ands decreased: And(MoreThan(B, 2), Or(Equals(C, 2), LessThan(C, 2)))

        simplified: Some(And(MoreThan(B, 2), LessThan(C, 3)))

        ((B > 2) && (C == 2)) || ((B > 2) && (C < 2))

        (B > 2) && ((C == 2) || (C < 2))

        (B > 2) && (C < 3)

        thats pretty epic

        */

        //println!("simplified: {:?}\n", simplify_ic_expr_full(expr));
    }
}

pub fn get_all_ic_connections(
    triggers: &mut TriggerNetwork,
    objects: &Triggerlist,
) -> Vec<(Group, Group, IcExpr, Trigger)> {
    let mut ictriggers = HashMap::<Group, Vec<(Group, IcExpr, Trigger)>>::new();
    let mut inputs = HashSet::<Group>::new();
    let mut outputs = HashSet::<Group>::new();

    for (group, gang) in triggers {
        let output_condition = gang
            .triggers
            .iter()
            .any(|t| objects[t.obj].0.params.get(&1) != Some(&ObjParam::Number(1811.0)));
        if output_condition {
            outputs.insert(*group);
        }
        for trigger in &mut gang.triggers {
            let obj = &objects[trigger.obj].0.params;

            if let Some(ObjParam::Number(n)) = obj.get(&1) {
                if *n as u16 == 1811 {
                    // dont include ones that dont activate a group
                    if obj.get(&56) == Some(&ObjParam::Bool(false)) {
                        continue;
                    }
                    let target = match obj.get(&51) {
                        Some(ObjParam::Group(g)) => *g,

                        _ => continue,
                    };

                    if gang.non_ic_triggers_in
                        || *group
                            == (Group {
                                id: Id::Specific(0),
                            })
                    {
                        inputs.insert(*group);
                    }

                    let item = if let ObjParam::Item(i) =
                        obj.get(&80).unwrap_or(&ObjParam::Item(Item {
                            id: Id::Specific(0),
                        })) {
                        *i
                    } else {
                        Item {
                            id: Id::Specific(0),
                        }
                    };
                    let num = if let ObjParam::Number(a) =
                        obj.get(&77).unwrap_or(&ObjParam::Number(0.0))
                    {
                        *a as i32
                    } else {
                        0
                    };
                    let expr = match obj.get(&88) {
                        Some(ObjParam::Number(n)) => match *n as u8 {
                            1 => IcExpr::MoreThan(item, num),
                            2 => IcExpr::LessThan(item, num),
                            _ => IcExpr::Equals(item, num),
                        },

                        _ => IcExpr::Equals(item, num),
                    };

                    let group = match obj.get(&57) {
                        Some(ObjParam::Group(g)) => *g,
                        Some(ObjParam::GroupList(_)) => unimplemented!(),
                        _ => Group {
                            id: Id::Specific(0),
                        },
                    };
                    // delete trigger that will be rebuilt
                    (*trigger).deleted = true;
                    (*trigger).optimized = true;

                    if let Some(l) = ictriggers.get_mut(&group) {
                        l.push((target, expr, *trigger))
                    } else {
                        ictriggers.insert(group, vec![(target, expr, *trigger)]);
                    }
                }
            }
        }
    }

    // println!(
    //     "ictriggers: {:?}\n\n inputs: {:?}\n\n outputs: {:?}\n",
    //     ictriggers, inputs, outputs
    // );

    let mut all = Vec::new();
    // set triggers that make cycles to inputs and outputs
    fn look_for_cycle(
        current: Group,
        ictriggers: &HashMap<Group, Vec<(Group, IcExpr, Trigger)>>,
        visited: HashSet<Group>,
        inputs: &mut HashSet<Group>,
        outputs: &mut HashSet<Group>,
        all: &mut Vec<(Group, Group, IcExpr, Trigger)>,
    ) {
        if let Some(connections) = ictriggers.get(&current) {
            for (g, expr, trigger) in connections {
                if visited.contains(&g) {
                    outputs.insert(current);
                    inputs.insert(*g);
                    all.push((current, *g, expr.clone(), *trigger));

                    return;
                }
                let mut new_visited = visited.clone();
                new_visited.insert(current);
                look_for_cycle(*g, ictriggers, new_visited, inputs, outputs, all);
            }
        }
    }
    for start in inputs.clone() {
        look_for_cycle(
            start,
            &ictriggers,
            HashSet::new(),
            &mut inputs,
            &mut outputs,
            &mut all,
        )
    }

    // go from every trigger in an input group and get every possible path to an
    // output group (stopping if it reaches a group already visited)
    fn traverse(
        current: Group,
        origin: Group,
        expr: IcExpr,
        trigger: Option<Trigger>,
        outputs: &HashSet<Group>,
        ictriggers: &HashMap<Group, Vec<(Group, IcExpr, Trigger)>>,
        visited: HashSet<Group>,
        d: u16,
    ) -> Vec<(Group, Group, IcExpr, Trigger)> {
        if visited.contains(&current) {
            unreachable!()
        }

        let mut out = Vec::new();
        if let Some(connections) = ictriggers.get(&current) {
            for (g, e, trigger) in connections {
                //println!("{:?} -> {:?}", current, g);
                let new_expr = if expr == IcExpr::True {
                    e.clone()
                } else {
                    IcExpr::And(expr.clone().into(), e.clone().into())
                };
                if outputs.contains(g) {
                    out.push((origin, *g, new_expr.clone(), *trigger));

                    /*
                    in cases like this:

                    1i.if_is(SMALLER_THAN, 1, !{

                        2i.if_is(EQUAL_TO, 0, !{
                            2i.add(1)
                            1i.if_is(SMALLER_THAN, 0, !{
                                -> BG.pulse(0, 0, 255, fade_out = 0.5)
                            })
                        })

                    })

                    we can't simplify the three expressions together, because we need the result of the 2nd one to happen before it's result
                    therefore, the chain is split before the third expression

                    it cannot add the new inputs to the set because it's used in the current loop, but it doens't matter since the set is not used after this.
                    */

                    out.extend(traverse(
                        *g,
                        *g,
                        IcExpr::True,
                        None,
                        &outputs,
                        &ictriggers,
                        HashSet::new(),
                        d + 1,
                    ));
                } else {
                    let mut new_visited = visited.clone(); // ,>,<[->-[>]<<]>. 1 1
                    new_visited.insert(current);
                    out.extend(traverse(
                        *g,
                        origin,
                        new_expr,
                        Some(*trigger),
                        outputs,
                        ictriggers,
                        new_visited,
                        d + 1,
                    ))
                }
            }
        } else if let Some(t) = trigger {
            out.push((origin, current, expr, t)) //?
        } else {
            assert!(outputs.contains(&current));
        }
        if !out.is_empty() {
            //println!("d: {}, out: {:?}", d, out.len());
        }
        out
    }

    for start in inputs {
        //println!("<{:?}>", start);
        all.extend(traverse(
            start,
            start,
            IcExpr::True, // Should be the same as no expression when 'and'ed together
            None,
            &outputs,
            &ictriggers,
            HashSet::new(),
            0,
        ));
        //println!("</{:?}>", start);
    }

    let mut finished_expressions = Vec::<((Group, Group), (IcExpr, Trigger))>::new();

    for (start, end, expr, trigger) in all {
        // if let Some(e) = finished_expressions.get_mut(&(start, end)) {
        //     *e = (IcExpr::Or(e.0.clone().into(), expr.clone().into()), e.1)
        // } else {
        finished_expressions.push(((start, end), (expr.clone(), trigger)));
        //}
    }

    for ((_, _), (expr, _)) in &mut finished_expressions {
        *expr = simplify_ic_expr_fast(expr.clone());
        // match expr {
        //     IcExpr::True
        //     | IcExpr::False
        //     | IcExpr::LessThan(_, _)
        //     | IcExpr::MoreThan(_, _)
        //     | IcExpr::Equals(_, _) => continue,
        //     _ => (),
        // };

        // if expr.get_variables().len() > 2 || get_solve_complexity(&expr) > 24 {
        //     continue;
        // }
        // *expr = simplify_ic_expr_full(expr.clone());
    }

    //println!("finished simplifying");

    let out = finished_expressions
        .iter()
        .map(|((start, end), (expr, trigger))| (*start, *end, expr.clone(), *trigger))
        .collect();

    //println!("\nout : {:?}", out);
    out
}

fn overlap(mut expr1: IcExpr, mut expr2: IcExpr) -> IcExpr {
    use IcExpr::*;
    expr1 = simplify_ic_expr_fast(expr1);
    expr2 = simplify_ic_expr_fast(expr2);

    let base_expr = And(Box::from(expr1.clone()), Box::from(expr2.clone()));
    match (expr1, expr2) {
        (True, True) => True,
        (False, _) | (_, False) => False,
        (Equals(item1, num1), Equals(item2, num2)) => {
            if item1 == item2 {
                if num1 != num2 {
                    False
                } else {
                    base_expr
                }
            } else {
                base_expr
            }
        }
        (MoreThan(item1, num1), MoreThan(item2, num2)) => {
            if item1 == item2 {
                MoreThan(item1, max(num1, num2))
            } else {
                base_expr
            }
        }
        (LessThan(item1, num1), LessThan(item2, num2)) => {
            if item1 == item2 {
                LessThan(item1, min(num1, num2))
            } else {
                base_expr
            }
        }
        (LessThan(item1, num1), MoreThan(item2, num2))
        | (MoreThan(item2, num2), LessThan(item1, num1)) => {
            if item1 == item2 && num1 <= num2 + 1 {
                False
            } else if item1 == item2 && num1 == num2 + 2 {
                Equals(item1, num2 + 1)
            } else {
                base_expr
            }
        }
        (Equals(item1, num1), MoreThan(item2, num2))
        | (MoreThan(item2, num2), Equals(item1, num1)) => {
            if item1 == item2 {
                if num1 > num2 {
                    Equals(item1, num1)
                } else {
                    False
                }
            } else {
                base_expr
            }
        }
        (Equals(item1, num1), LessThan(item2, num2))
        | (LessThan(item2, num2), Equals(item1, num1)) => {
            if item1 == item2 {
                if num1 < num2 {
                    Equals(item1, num1)
                } else {
                    False
                }
            } else {
                base_expr
            }
        }
        (Or(or1, or2), expr2) | (expr2, Or(or1, or2)) => {
            let attempt = simplify_ic_expr_fast(Or(
                And(or1, expr2.clone().into()).into(),
                And(or2, expr2.into()).into(),
            ));
            if get_complexity(&attempt) < get_complexity(&base_expr) {
                attempt
            } else {
                base_expr
            }
        }
        (And(and1, and2), expr2) | (expr2, And(and1, and2)) => {
            let combinations = [
                And(
                    simplify_ic_expr_fast(And(and1.clone(), expr2.clone().into())).into(),
                    and2.clone(),
                ),
                And(simplify_ic_expr_fast(And(and2, expr2.into())).into(), and1),
                base_expr,
            ];
            combinations
                .iter()
                .min_by(|a, b| get_complexity(&a).cmp(&get_complexity(&b)))
                .unwrap()
                .clone()
        }

        (_, _) => base_expr,
    }
}

fn union(mut expr1: IcExpr, mut expr2: IcExpr) -> IcExpr {
    use IcExpr::*;
    expr1 = simplify_ic_expr_fast(expr1);
    expr2 = simplify_ic_expr_fast(expr2);

    And(Box::from(expr1), Box::from(expr2))
    // match (expr1, expr2) {
    //     (False, False) => False,
    //     (True, _) | (_, True) => True,
    //     (Equals(item1, num1), Equals(item2, num2)) => {
    //         if item1 == item2 {
    //             if num1 != num2 {
    //                 base_expr
    //             } else {
    //                 Equals(item1, num1)
    //             }
    //         } else {
    //             base_expr
    //         }
    //     }
    //     (MoreThan(item1, num1), MoreThan(item2, num2)) => {
    //         if item1 == item2 {
    //             MoreThan(item1, min(num1, num2))
    //         } else {
    //             base_expr
    //         }
    //     }
    //     (LessThan(item1, num1), LessThan(item2, num2)) => {
    //         if item1 == item2 {
    //             LessThan(item1, max(num1, num2))
    //         } else {
    //             base_expr
    //         }
    //     }
    //     (LessThan(item1, num1), MoreThan(item2, num2))
    //     | (MoreThan(item2, num2), LessThan(item1, num1)) => {
    //         if item1 == item2 && num1 > num2 {
    //             True
    //         } else {
    //             base_expr
    //         }
    //     }
    //     (Equals(item1, num1), MoreThan(item2, num2))
    //     | (MoreThan(item2, num2), Equals(item1, num1)) => {
    //         if item1 == item2 {
    //             match num1.cmp(&num2) {
    //                 Ordering::Greater => MoreThan(item2, num2),
    //                 Ordering::Equal => MoreThan(item2, num1 - 1),
    //                 _ => base_expr,
    //             }
    //         } else {
    //             base_expr
    //         }
    //     }
    //     (Equals(item1, num1), LessThan(item2, num2))
    //     | (LessThan(item2, num2), Equals(item1, num1)) => {
    //         if item1 == item2 {
    //             match num1.cmp(&num2) {
    //                 Ordering::Less => LessThan(item2, num2),
    //                 Ordering::Equal => LessThan(item2, num1 + 1),
    //                 _ => base_expr,
    //             }
    //         } else {
    //             base_expr
    //         }
    //     }

    //     (_, _) => base_expr,
    // }
}

fn simplify_ic_expr_fast(mut expr: IcExpr) -> IcExpr {
    //println!("\n\nstart fast: {:?}", expr);
    // expr = expr.remove_duplicates();
    // expr = expr.decrease_and();
    expr = match expr {
        IcExpr::And(e1, e2) => overlap(*e1, *e2),
        IcExpr::Or(e1, e2) => union(*e1, *e2),
        a => a,
    };
    //println!("\nend fast: {:?}", expr);
    expr
}

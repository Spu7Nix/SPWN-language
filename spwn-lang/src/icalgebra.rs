// instant count algebra :pog:
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IcExpr {
    Or(Box<IcExpr>, Box<IcExpr>),
    And(Box<IcExpr>, Box<IcExpr>),
    True,
    False,
    Equals(Item, i32),
    MoreThan(Item, i32),
    LessThan(Item, i32),
}

#[derive(Debug, Clone, Eq)]
struct HeapItem {
    complexity: (u16, u16),
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
        (self.complexity.0 + self.complexity.1).cmp(&(other.complexity.0 + other.complexity.1))
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
    let mut out = Vec::new();
    out.push(IcExpr::True);
    out.push(IcExpr::False);
    for (variable, value_set) in critical_value_sets.iter() {
        for value in value_set {
            out.push(IcExpr::LessThan(*variable, *value));
            if let Some(_) = value_set.get(&(value + 1)) {
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
fn get_truth_table(formula: &IcExpr, inputs: &Vec<HashMap<Item, i32>>) -> u64 {
    let mut truth_table = 0;
    for input in inputs {
        truth_table = (truth_table << 1) + evaluate(formula, input) as u64;
    }
    truth_table
}

// Returns (the number of operations in the formula, the number of Ands).
fn get_complexity(formula: &IcExpr) -> (u16, u16) {
    match formula {
        IcExpr::True | IcExpr::False => (0, 0),
        IcExpr::LessThan(_, _) | IcExpr::MoreThan(_, _) | IcExpr::Equals(_, _) => (1, 0),
        IcExpr::And(lhs, rhs) => {
            let (ops_lhs, ands_lhs) = get_complexity(&**lhs);
            let (ops_rhs, ands_rhs) = get_complexity(&**rhs);
            (ops_lhs + 1 + ops_rhs, ands_lhs + 1 + ands_rhs)
        }
        IcExpr::Or(lhs, rhs) => {
            let (ops_lhs, ands_lhs) = get_complexity(&**lhs);
            let (ops_rhs, ands_rhs) = get_complexity(&**rhs);
            (ops_lhs + 1 + ops_rhs, ands_lhs + ands_rhs)
        }
    }
}
fn simplify_ic_expr_full(target_formula: IcExpr) -> IcExpr {
    println!("\nstart: {:?}\n", target_formula);
    let mut critical_value_sets = HashMap::new();
    get_critical_value_sets(&target_formula, &mut critical_value_sets);
    let inputs = enumerate_truth_table_inputs(&critical_value_sets);
    let target_truth_table = get_truth_table(&target_formula, &inputs);
    let mut best = HashMap::<u64, IcExpr>::new();
    let mut heap: BinaryHeap<Reverse<HeapItem>> = enumerate_useful_primitives(&critical_value_sets)
        .iter()
        .map(|a| Reverse(HeapItem::new(a.clone())))
        .collect();
    while let None = best.get(&target_truth_table) {
        let formula = heap.pop().unwrap().0.formula;
        let truth_table = get_truth_table(&formula, &inputs);
        if let Some(_) = best.get(&truth_table) {
            continue;
        }

        for other_formula in best.values() {
            heap.push(Reverse(HeapItem::new(IcExpr::And(
                formula.clone().into(),
                other_formula.clone().into(),
            ))));
            heap.push(Reverse(HeapItem::new(IcExpr::Or(
                formula.clone().into(),
                other_formula.clone().into(),
            ))));
        }
        best.insert(truth_table, formula);
    }
    let out = best[&target_truth_table].clone();
    println!("end: {:?}\n", out);
    out
}

fn build_instant_count_network<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start_group: Option<Group>,
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
                (true, false),
            );
            true
        }

        IcExpr::True => {
            // This can be optimized
            create_spawn_trigger(
                reference_trigger,
                target,
                start_group,
                0.0,
                objects,
                network,
                (true, false),
            );
            true
        }

        IcExpr::And(expr1, expr2) => {
            (*closed_group) += 1;
            let middle_group = Group {
                id: ID::Arbitrary(*closed_group),
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
                    Some(middle_group),
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
        _ => unreachable!(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let A = Item {
            id: ID::Specific(1),
        };
        let B = Item {
            id: ID::Specific(2),
        };
        let C = Item {
            id: ID::Specific(3),
        };
        use IcExpr::*;

        let expr = Or(
            And(LessThan(A, 1).into(), Equals(A, 5).into()).into(),
            And(MoreThan(A, 1).into(), Equals(A, 5).into()).into(),
        );

        println!("start: {:?}\n", expr);

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

        println!("simplified: {:?}\n", simplify_ic_expr_full(expr));
    }
}

fn create_instant_count_trigger(
    reference_trigger: Trigger,
    target_group: Group,
    group: Option<Group>,
    operation: u8,
    num: i32,
    item: Item,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    //         opt   del
    settings: (bool, bool),
) {
    let mut new_obj_map = HashMap::new();
    new_obj_map.insert(1, ObjParam::Number(1811.0));
    new_obj_map.insert(51, ObjParam::Group(target_group));
    new_obj_map.insert(80, ObjParam::Item(item));
    new_obj_map.insert(77, ObjParam::Number(num.into()));
    new_obj_map.insert(88, ObjParam::Number(operation.into()));

    if let Some(g) = group {
        new_obj_map.insert(57, ObjParam::Group(g));
    }

    let new_obj = GDObj {
        params: new_obj_map,
        func_id: reference_trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[reference_trigger.obj].0.unique_id,
        sync_group: 0,
        sync_part: 0,
    };

    (*objects.list)[reference_trigger.obj.0]
        .obj_list
        .push((new_obj.clone(), reference_trigger.order));

    let obj_index = (
        reference_trigger.obj.0,
        objects.list[reference_trigger.obj.0].obj_list.len() - 1,
    );
    let new_trigger = Trigger {
        obj: obj_index,
        optimized: settings.0,
        deleted: settings.1,
        role: TriggerRole::Operator,
        ..reference_trigger
    };

    if let Some(ObjParam::Group(group)) = new_obj.params.get(&57) {
        match network.get_mut(group) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network.insert(*group, TriggerGang::new(vec![new_trigger]));
            }
        }
    } else {
        match network.get_mut(&NO_GROUP) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network.insert(NO_GROUP, TriggerGang::new(vec![new_trigger]));
            }
        }
    }
}

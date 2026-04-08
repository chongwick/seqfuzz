use crate::lang::LangDescription;
use crate::nodes::*;
use crate::sequence::*;
use rand::Rng;

/// Splice dataflow from splicing_env into target_env by replacing random entries
/// in target's relational database with random nodes from either environment.
pub fn stochastic_splice_dataflow(target: &mut Seq, splice: &Seq, reps: usize) {
    let mut rng = rand::thread_rng();

    for _ in 0..reps {
        let tar_db_len = target.relational_database.len();
        if tar_db_len == 0 {
            continue;
        }

        let index = if tar_db_len > 1 {
            rng.gen_range(0..tar_db_len)
        } else {
            0
        };

        // Collect candidate nodes from both environments
        let mut candidate_indices: Vec<(bool, usize)> = Vec::new(); // (is_splice, idx)
        for &idx in &splice.all_nodes {
            candidate_indices.push((true, idx));
        }
        for &idx in &target.all_nodes {
            candidate_indices.push((false, idx));
        }

        if candidate_indices.is_empty() {
            continue;
        }

        let chosen = &candidate_indices[rng.gen_range(0..candidate_indices.len())];
        let chosen_node = if chosen.0 {
            splice.relational_database[chosen.1].clone()
        } else {
            target.relational_database[chosen.1].clone()
        };

        // If the chosen node is a Func with a dependency, register it
        if let SeqNode::Func {
            ref func_name,
            ref dependency,
            ..
        } = chosen_node
        {
            if dependency.is_some() {
                // If the source has this function env, copy it
                if let Some(func_env) = splice.function_envs.get(func_name) {
                    target
                        .function_envs
                        .insert(func_name.clone(), func_env.clone());
                }
            }
        }

        if index < target.relational_database.len() {
            target.relational_database[index] = chosen_node;
        }
    }
}

/// Splice a control flow branch from splicing_env into target_env.
/// Returns false if no flow branches available to splice.
pub fn stochastic_splice_controlflow(target: &mut Seq, splice: &Seq) -> bool {
    let mut rng = rand::thread_rng();

    if splice.flow_branches.is_empty() {
        return false;
    }

    // Choose a random flow from splice
    let flow = splice.flow_branches[rng.gen_range(0..splice.flow_branches.len())].clone();

    // Choose a random target: either target itself or one of its flow branches
    let target_count = target.flow_branches.len() + 1;
    let target_choice = rng.gen_range(0..target_count);

    if target_choice == target.flow_branches.len() {
        // Insert into root
        target.flow_branches.push(flow);
        let flow_idx = target.flow_branches.len() - 1;
        target.set_node(None, EnvValue::Flow(flow_idx));
    } else {
        // Insert into a flow branch
        let target_flow = &mut target.flow_branches[target_choice];
        target_flow.flow_branches.push(flow);
        let flow_idx = target_flow.flow_branches.len() - 1;
        target_flow.set_node(None, EnvValue::Flow(flow_idx));
    }

    // Also do dataflow splicing
    stochastic_splice_dataflow(target, splice, 1);
    true
}

/// Combine multiple trees into one by randomly selecting nodes from all trees.
pub fn rando_max(trees: &[Seq]) -> Seq {
    let mut rng = rand::thread_rng();
    let mut yggdrasil = Seq::new();

    // Collect all nodes from all trees
    let mut all_candidate_nodes: Vec<SeqNode> = Vec::new();
    for t in trees {
        for &idx in &t.all_nodes {
            if idx < t.relational_database.len() {
                all_candidate_nodes.push(t.relational_database[idx].clone());
            }
        }
    }

    // Collect all relationships
    let mut all_relationships: Vec<SeqNode> = Vec::new();
    for t in trees {
        all_relationships.extend(t.relational_database.iter().cloned());
    }

    if all_candidate_nodes.is_empty() {
        return yggdrasil;
    }

    // Randomly select nodes
    for _ in 0..all_candidate_nodes.len() {
        let node = all_candidate_nodes[rng.gen_range(0..all_candidate_nodes.len())].clone();
        let idx = yggdrasil.relational_database.len();
        yggdrasil.relational_database.push(node);
        yggdrasil.set_node(None, EnvValue::Node(idx));
        yggdrasil.all_nodes.push(idx);
    }

    // Set relationships
    yggdrasil.relational_database = all_relationships;

    // Try control flow splicing, fall back to dataflow
    let ygg_clone = yggdrasil.clone();
    if !stochastic_splice_controlflow(&mut yggdrasil, &ygg_clone) {
        let ygg_clone2 = yggdrasil.clone();
        stochastic_splice_dataflow(&mut yggdrasil, &ygg_clone2, 1);
    }

    yggdrasil
}

/// Mutate operator nodes in the target environment by randomly changing their operators.
pub fn modify_operation(target: &mut Seq, lang: &LangDescription) {
    let mut rng = rand::thread_rng();

    // Collect indices of op nodes
    let op_indices: Vec<usize> = target
        .all_nodes
        .iter()
        .filter(|&&idx| {
            idx < target.relational_database.len()
                && target.relational_database[idx].is_op_node()
        })
        .copied()
        .collect();

    if op_indices.is_empty() {
        return;
    }

    let middle_ops: Vec<&String> = lang.middle_ops.values().collect();
    let front_ops: Vec<&String> = lang.front_ops.values().collect();
    let inner_ops: Vec<&String> = lang.inner_ops.values().collect();
    let rear_ops: Vec<&String> = lang.rear_ops.values().collect();

    let num_mutations = rng.gen_range(0..=op_indices.len());
    for _ in 0..num_mutations {
        let idx = op_indices[rng.gen_range(0..op_indices.len())];
        let node = &mut target.relational_database[idx];
        match node {
            SeqNode::MiddleOp { value, .. } => {
                if !middle_ops.is_empty() {
                    *value = middle_ops[rng.gen_range(0..middle_ops.len())].clone();
                }
            }
            SeqNode::FrontOp { value, .. } => {
                if !front_ops.is_empty() {
                    *value = front_ops[rng.gen_range(0..front_ops.len())].clone();
                }
            }
            SeqNode::InnerOp { value, .. } => {
                if !inner_ops.is_empty() {
                    *value = inner_ops[rng.gen_range(0..inner_ops.len())].clone();
                }
            }
            SeqNode::RearOp { value, .. } => {
                if !rear_ops.is_empty() {
                    *value = rear_ops[rng.gen_range(0..rear_ops.len())].clone();
                }
            }
            _ => {}
        }
    }
}

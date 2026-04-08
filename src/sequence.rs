use crate::nodes::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An entry in node_env: either a SeqNode index or a FlowBranch index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvValue {
    Node(usize),    // index into relational_database
    Flow(usize),    // index into flow_branches vec
    Direct(SeqNode), // a node not in rel_db (for NonAssignment, etc.)
}

/// The main sequence environment, representing a scope of PHP code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seq {
    /// Ordered map of named/anonymous values in this environment
    pub node_env: IndexMap<EnvKey, EnvValue>,
    /// All nodes registered via check_in (only relevant for top-level)
    pub all_nodes: Vec<usize>, // indices into relational_database
    /// The relational database: shared arena of all SeqNodes
    pub relational_database: Vec<SeqNode>,
    /// Child environments
    pub function_envs: IndexMap<String, FunctionBranch>,
    pub flow_branches: Vec<FlowBranch>,
    /// Class definitions (class_name -> source string)
    pub defined_classes: HashMap<String, String>,
    /// Object declarations (obj_name -> rel_db index)
    pub defined_objects: HashMap<String, usize>,
    /// Whether this is a child env (has parent)
    pub is_child: bool,
}

/// Key for node_env entries
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvKey {
    Named(String),
    Anonymous(i64),
}

#[allow(dead_code)]
impl Seq {
    pub fn new() -> Self {
        Seq {
            node_env: IndexMap::new(),
            all_nodes: Vec::new(),
            relational_database: Vec::new(),
            function_envs: IndexMap::new(),
            flow_branches: Vec::new(),
            defined_classes: HashMap::new(),
            defined_objects: HashMap::new(),
            is_child: false,
        }
    }

    /// Register a node into all_nodes (only meaningful for root env).
    /// In child envs, this should propagate to root, but since we pass root's
    /// all_nodes around, we handle this at the walker level.
    pub fn check_in(&mut self, rel_db_idx: usize) {
        self.all_nodes.push(rel_db_idx);
    }

    /// Add a node to the relational database, return its index.
    pub fn set_relation(&mut self, node: SeqNode) -> usize {
        self.relational_database.push(node);
        self.relational_database.len() - 1
    }

    pub fn get_relation(&self, key: usize) -> &SeqNode {
        &self.relational_database[key]
    }

    pub fn get_relation_mut(&mut self, key: usize) -> &mut SeqNode {
        &mut self.relational_database[key]
    }

    /// Set a node in this environment.
    pub fn set_node(&mut self, name: Option<&str>, value: EnvValue) {
        match name {
            Some(n) => {
                let key = EnvKey::Named(n.to_string());
                if self.node_env.contains_key(&key) {
                    // Rename the old entry with a negative key (like Python)
                    let neg_key = EnvKey::Anonymous(-(self.node_env.len() as i64));
                    // Get the old name from the old node to preserve it on the new node
                    let old_value = self.node_env.shift_remove(&key).unwrap();
                    self.node_env.insert(neg_key, old_value);

                    // If both old and new are rel_db indices, copy the name
                    if let EnvValue::Node(new_idx) = &value {
                        // We need to set the name on the new node to match the old one
                        // But we need access to rel_db which is &mut self
                        // This is handled by the caller
                        let _ = new_idx;
                    }
                }
                self.node_env.insert(key, value);
            }
            None => {
                let idx = self.node_env.len() as i64;
                self.node_env.insert(EnvKey::Anonymous(idx), value);
            }
        }
    }

    /// Set a node by name, copying the old node's name to the new node.
    pub fn set_node_with_rename(&mut self, name: &str, rel_db_idx: usize) {
        let key = EnvKey::Named(name.to_string());
        if self.node_env.contains_key(&key) {
            let old_value = self.node_env.shift_remove(&key).unwrap();
            let neg_key = EnvKey::Anonymous(-(self.node_env.len() as i64));
            // Get old node's name
            if let EnvValue::Node(old_idx) = &old_value {
                if let Some(old_name) = self.relational_database[*old_idx].get_name() {
                    let old_name = old_name.to_string();
                    self.relational_database[rel_db_idx].set_name(old_name);
                }
            }
            self.node_env.insert(neg_key, old_value);
        }
        self.node_env.insert(key, EnvValue::Node(rel_db_idx));
    }

    pub fn set_node_anonymous(&mut self, value: EnvValue) {
        let idx = self.node_env.len() as i64;
        self.node_env.insert(EnvKey::Anonymous(idx), value);
    }

    pub fn add_class_def(&mut self, class_name: String, definition: String) {
        self.defined_classes.insert(class_name, definition);
    }

    pub fn get_class_def(&self, class_name: &str) -> Option<&String> {
        self.defined_classes.get(class_name)
    }

    pub fn add_object(&mut self, obj_name: String, rel_db_idx: usize) {
        self.defined_objects.insert(obj_name, rel_db_idx);
    }

    pub fn get_object(&self, obj_name: &str) -> Option<usize> {
        self.defined_objects.get(obj_name).copied()
    }

    pub fn get_node(&self, name: &str) -> Option<&EnvValue> {
        self.node_env.get(&EnvKey::Named(name.to_string()))
    }

    pub fn get_node_rel_idx(&self, name: &str) -> Option<usize> {
        match self.node_env.get(&EnvKey::Named(name.to_string())) {
            Some(EnvValue::Node(idx)) => Some(*idx),
            _ => None,
        }
    }

    pub fn add_fn_dependency(&mut self, name: String, dep: FunctionBranch) {
        self.function_envs.insert(name, dep);
    }

    pub fn output_env(&mut self, output_str: &mut Vec<String>) {
        // Ensure FuncSeqNode dependencies are registered
        for node_idx in &self.all_nodes.clone() {
            let node = &self.relational_database[*node_idx];
            if let SeqNode::Func {
                func_name,
                dependency,
                ..
            } = node
            {
                if !self.function_envs.contains_key(func_name) {
                    if let Some(dep_name) = dependency {
                        // The dependency is a function env name - look it up
                        // This mirrors the Python: if fn.get_func_name() not in children_envs["Function"]
                        // and fn.get_dependency() != None, add it
                        let _ = dep_name; // Already in function_envs if it exists
                    }
                }
            }
        }

        // Output function definitions first
        let func_keys: Vec<String> = self.function_envs.keys().cloned().collect();
        for func_name in func_keys {
            let mut func_env = self.function_envs.shift_remove(&func_name).unwrap();
            func_env.output_env(&mut self.relational_database, output_str);
            self.function_envs.insert(func_name, func_env);
        }

        // Output all values in node_env
        let keys: Vec<EnvKey> = self.node_env.keys().cloned().collect();
        for key in keys {
            let value = self.node_env.get(&key).unwrap().clone();
            match value {
                EnvValue::Node(idx) => {
                    get_expression(&mut self.relational_database, idx, output_str);
                }
                EnvValue::Flow(flow_idx) => {
                    let mut flow = self.flow_branches[flow_idx].clone();
                    flow.get_expression(
                        &mut self.relational_database,
                        &self.node_env,
                        output_str,
                    );
                    self.flow_branches[flow_idx] = flow;
                }
                EnvValue::Direct(mut node) => {
                    // For NonAssignment nodes stored directly
                    let mut tmp = Vec::new();
                    node_get_expression_direct(&mut node, &mut self.relational_database, &mut tmp);
                    output_str.extend(tmp);
                }
            }
        }
    }

    /// Spawn a new FlowBranch child environment, copying current node_env.
    pub fn spawn_flow(&mut self, flow_type: &str) -> usize {
        let flow = FlowBranch::new(self.node_env.clone(), flow_type.to_string());
        self.flow_branches.push(flow);
        let flow_idx = self.flow_branches.len() - 1;
        // Add to node_env for printing
        let anon_key = EnvKey::Anonymous(self.node_env.len() as i64);
        self.node_env.insert(anon_key, EnvValue::Flow(flow_idx));
        flow_idx
    }

    /// Spawn a new FunctionBranch child environment.
    pub fn spawn_function(&mut self, func_name: &str) -> &mut FunctionBranch {
        let func = FunctionBranch::new(func_name.to_string());
        self.function_envs.insert(func_name.to_string(), func);
        self.function_envs.get_mut(func_name).unwrap()
    }
}

/// Helper to get expression from a directly-stored node
fn node_get_expression_direct(
    node: &mut SeqNode,
    _rel_db: &mut Vec<SeqNode>,
    pp_env: &mut Vec<String>,
) {
    if let SeqNode::NonAssignment { value } = node {
        pp_env.push(value.clone());
    }
}

/// Function definition environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBranch {
    pub func_name: String,
    pub node_env: IndexMap<EnvKey, EnvValue>,
    pub params: IndexMap<String, usize>, // param_name -> rel_db index
    pub function_envs: IndexMap<String, FunctionBranch>,
    pub flow_branches: Vec<FlowBranch>,
}

#[allow(dead_code)]
impl FunctionBranch {
    pub fn new(func_name: String) -> Self {
        FunctionBranch {
            func_name,
            node_env: IndexMap::new(),
            params: IndexMap::new(),
            function_envs: IndexMap::new(),
            flow_branches: Vec::new(),
        }
    }

    pub fn set_params(&mut self, params: IndexMap<String, usize>) {
        self.params = params;
    }

    pub fn set_node(&mut self, name: Option<&str>, value: EnvValue) {
        match name {
            Some(n) => {
                let key = EnvKey::Named(n.to_string());
                self.node_env.insert(key, value);
            }
            None => {
                let idx = self.node_env.len() as i64;
                self.node_env.insert(EnvKey::Anonymous(idx), value);
            }
        }
    }

    pub fn set_node_with_rename(&mut self, name: &str, rel_db: &mut Vec<SeqNode>, rel_db_idx: usize) {
        let key = EnvKey::Named(name.to_string());
        if self.node_env.contains_key(&key) {
            let old_value = self.node_env.shift_remove(&key).unwrap();
            let neg_key = EnvKey::Anonymous(-(self.node_env.len() as i64));
            if let EnvValue::Node(old_idx) = &old_value {
                if let Some(old_name) = rel_db[*old_idx].get_name() {
                    let old_name = old_name.to_string();
                    rel_db[rel_db_idx].set_name(old_name);
                }
            }
            self.node_env.insert(neg_key, old_value);
        }
        self.node_env.insert(key, EnvValue::Node(rel_db_idx));
    }

    pub fn get_node(&self, name: &str) -> Option<&EnvValue> {
        self.node_env.get(&EnvKey::Named(name.to_string()))
    }

    pub fn get_node_rel_idx(&self, name: &str) -> Option<usize> {
        match self.node_env.get(&EnvKey::Named(name.to_string())) {
            Some(EnvValue::Node(idx)) => Some(*idx),
            _ => None,
        }
    }

    pub fn spawn_flow(&mut self, flow_type: &str, parent_node_env: &IndexMap<EnvKey, EnvValue>) -> usize {
        let flow = FlowBranch::new(parent_node_env.clone(), flow_type.to_string());
        self.flow_branches.push(flow);
        let flow_idx = self.flow_branches.len() - 1;
        let anon_key = EnvKey::Anonymous(self.node_env.len() as i64);
        self.node_env.insert(anon_key, EnvValue::Flow(flow_idx));
        flow_idx
    }

    pub fn output_env(&mut self, rel_db: &mut Vec<SeqNode>, output_str: &mut Vec<String>) {
        let mut func_prototype = format!("function {}(", self.func_name);
        let param_keys: Vec<String> = self.params.keys().cloned().collect();
        for param_name in &param_keys {
            let param_idx = self.params[param_name];
            let param_node = &rel_db[param_idx];
            let name = param_node.get_name().unwrap_or("").to_string();
            func_prototype.push_str(&name);
            if let Some(nv) = param_node.get_node_value() {
                if !nv.is_none() {
                    func_prototype.push_str(&format!("={},", nv.to_php_string()));
                    // Set value to name (like Python: param.set_value(param.get_name()))
                    rel_db[param_idx].set_value_str(name.clone());
                } else {
                    func_prototype.push(',');
                }
            } else {
                func_prototype.push(',');
            }
        }
        func_prototype.push_str("){\n");
        output_str.push(func_prototype);

        // Output nested function definitions
        let func_keys: Vec<String> = self.function_envs.keys().cloned().collect();
        for fname in func_keys {
            let mut fenv = self.function_envs.shift_remove(&fname).unwrap();
            fenv.output_env(rel_db, output_str);
            self.function_envs.insert(fname, fenv);
        }

        // Output body
        let keys: Vec<EnvKey> = self.node_env.keys().cloned().collect();
        for key in keys {
            let value = self.node_env.get(&key).unwrap().clone();
            match value {
                EnvValue::Node(idx) => {
                    get_expression(rel_db, idx, output_str);
                }
                EnvValue::Flow(flow_idx) => {
                    let mut flow = self.flow_branches[flow_idx].clone();
                    flow.get_expression(rel_db, &self.node_env, output_str);
                    self.flow_branches[flow_idx] = flow;
                }
                EnvValue::Direct(mut node) => {
                    node_get_expression_direct(&mut node, rel_db, output_str);
                }
            }
        }
        output_str.push("}".to_string());
    }
}

/// Control flow branch (if, else, while, for, foreach, try, catch, switch, case, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowBranch {
    pub flow_type: String,
    pub node_env: IndexMap<EnvKey, EnvValue>,
    pub init_env: IndexMap<EnvKey, EnvValue>, // snapshot of parent env at creation
    pub flow_branches: Vec<FlowBranch>,       // nested flows (e.g. cases in switch)
    pub defined_objects: HashMap<String, usize>,

    // Condition info
    pub conditional_statements: Option<ConditionExpr>,
    pub conditional_variable: Option<String>,

    // Try-catch specific
    pub catch_types: Option<Vec<String>>,
    pub catch_var: Option<String>,

    // Foreach specific
    pub foreach_var: Option<String>,
    pub by_ref_var: bool,

    // For specific
    pub for_var: Option<String>,

    // Switch case
    pub cases: Option<Vec<String>>,
}

/// Represents conditional statements which can be a single node ref or multiple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionExpr {
    Single(usize),           // rel_db index
    Multiple(Vec<CondItem>), // for switch/for statements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CondItem {
    NodeRef(usize),  // rel_db index
    Literal(String), // raw string (for parsed conditions)
}

#[allow(dead_code)]
impl FlowBranch {
    pub fn new(parent_node_env: IndexMap<EnvKey, EnvValue>, flow_type: String) -> Self {
        FlowBranch {
            flow_type,
            node_env: parent_node_env.clone(),
            init_env: parent_node_env,
            flow_branches: Vec::new(),
            defined_objects: HashMap::new(),
            conditional_statements: None,
            conditional_variable: None,
            catch_types: None,
            catch_var: None,
            foreach_var: None,
            by_ref_var: false,
            for_var: None,
            cases: None,
        }
    }

    pub fn set_condition(&mut self, cond: Option<ConditionExpr>, cond_var: String) {
        self.conditional_statements = cond;
        self.conditional_variable = Some(cond_var);
    }

    pub fn set_catch_types_var(&mut self, types: Vec<String>, var: String) {
        self.catch_types = Some(types);
        self.catch_var = Some(var);
    }

    pub fn set_foreach_var(&mut self, var: String) {
        self.foreach_var = Some(var);
    }

    pub fn set_by_ref_foreach_var(&mut self, by_ref: bool) {
        self.by_ref_var = by_ref;
    }

    pub fn set_for_var(&mut self, var: String) {
        self.for_var = Some(var);
    }

    pub fn set_node(&mut self, name: Option<&str>, value: EnvValue) {
        match name {
            Some(n) => {
                let key = EnvKey::Named(n.to_string());
                if self.node_env.contains_key(&key) {
                    let old_value = self.node_env.shift_remove(&key).unwrap();
                    let neg_key = EnvKey::Anonymous(-(self.node_env.len() as i64));
                    self.node_env.insert(neg_key, old_value);
                }
                self.node_env.insert(key, value);
            }
            None => {
                let idx = self.node_env.len() as i64;
                self.node_env.insert(EnvKey::Anonymous(idx), value);
            }
        }
    }

    pub fn set_node_with_rename(&mut self, name: &str, rel_db: &mut Vec<SeqNode>, rel_db_idx: usize) {
        let key = EnvKey::Named(name.to_string());
        if self.node_env.contains_key(&key) {
            let old_value = self.node_env.shift_remove(&key).unwrap();
            let neg_key = EnvKey::Anonymous(-(self.node_env.len() as i64));
            if let EnvValue::Node(old_idx) = &old_value {
                if let Some(old_name) = rel_db[*old_idx].get_name() {
                    let old_name = old_name.to_string();
                    rel_db[rel_db_idx].set_name(old_name);
                }
            }
            self.node_env.insert(neg_key, old_value);
        }
        self.node_env.insert(key, EnvValue::Node(rel_db_idx));
    }

    pub fn get_node(&self, name: &str) -> Option<&EnvValue> {
        self.node_env.get(&EnvKey::Named(name.to_string()))
    }

    pub fn get_node_rel_idx(&self, name: &str) -> Option<usize> {
        match self.node_env.get(&EnvKey::Named(name.to_string())) {
            Some(EnvValue::Node(idx)) => Some(*idx),
            _ => None,
        }
    }

    pub fn spawn_flow(&mut self, flow_type: &str) -> usize {
        let flow = FlowBranch::new(self.node_env.clone(), flow_type.to_string());
        self.flow_branches.push(flow);
        let flow_idx = self.flow_branches.len() - 1;
        let anon_key = EnvKey::Anonymous(self.node_env.len() as i64);
        self.node_env.insert(anon_key, EnvValue::Flow(flow_idx));
        flow_idx
    }

    pub fn add_object(&mut self, obj_name: String, rel_db_idx: usize) {
        self.defined_objects.insert(obj_name, rel_db_idx);
    }

    pub fn get_object(&self, obj_name: &str) -> Option<usize> {
        self.defined_objects.get(obj_name).copied()
    }

    /// Generate the PHP expression for this flow branch.
    pub fn get_expression(
        &mut self,
        rel_db: &mut Vec<SeqNode>,
        parent_node_env: &IndexMap<EnvKey, EnvValue>,
        output_str: &mut Vec<String>,
    ) {
        let mut ret_expr = String::new();

        match self.flow_type.as_str() {
            "if" => {
                if let Some(ConditionExpr::Single(cond_idx)) = self.conditional_statements {
                    let mut tmp = Vec::new();
                    get_expression(rel_db, cond_idx, &mut tmp);
                    let cond_expr = tmp.join("\n");
                    let cv = self.conditional_variable.as_deref().unwrap_or("");
                    ret_expr = format!("{} \nif({})", cond_expr, cv);
                }
            }
            "elseif" => {
                if let Some(ConditionExpr::Single(cond_idx)) = self.conditional_statements {
                    let mut tmp = Vec::new();
                    get_expression(rel_db, cond_idx, &mut tmp);
                    let cond_expr = tmp.join("\n");
                    let cv = self.conditional_variable.as_deref().unwrap_or("");
                    ret_expr = format!("{} \nelseif({})", cond_expr, cv);
                }
            }
            "else" => {
                ret_expr = "else".to_string();
            }
            "try" => {
                ret_expr = "try".to_string();
            }
            "catch" => {
                ret_expr = "catch (".to_string();
                if let Some(types) = &self.catch_types {
                    for t in types {
                        ret_expr.push_str(t);
                        ret_expr.push_str(" | ");
                    }
                }
                // Remove trailing " | "
                if ret_expr.ends_with(" | ") {
                    ret_expr.truncate(ret_expr.len() - 2);
                }
                let cv = self.catch_var.as_deref().unwrap_or("");
                ret_expr.push('$');
                ret_expr.push_str(cv);
                ret_expr.push(')');
            }
            "switch" => {
                if let Some(ConditionExpr::Multiple(items)) = &self.conditional_statements {
                    let mut tmp = Vec::new();
                    for item in items {
                        match item {
                            CondItem::NodeRef(idx) => {
                                get_expression(rel_db, *idx, &mut tmp);
                            }
                            CondItem::Literal(s) => tmp.push(s.clone()),
                        }
                    }
                    let cond_expr = tmp.join("\n");
                    let cv = self.conditional_variable.as_deref().unwrap_or("");
                    ret_expr = format!("{}\nswitch({})", cond_expr, cv);
                }
            }
            "case" => {
                let cv = self.conditional_variable.as_deref().unwrap_or("");
                ret_expr = format!("case {}:\n", cv);
            }
            "case_default" => {
                ret_expr = "default:\n".to_string();
            }
            "foreach" => {
                if let Some(ConditionExpr::Single(cond_idx)) = self.conditional_statements {
                    let mut tmp = Vec::new();
                    get_expression(rel_db, cond_idx, &mut tmp);
                    let cond_expr = tmp.join("\n");
                    let cv = self.conditional_variable.as_deref().unwrap_or("");
                    let fv = self.foreach_var.as_deref().unwrap_or("");
                    let tmp_foreach_var = if self.by_ref_var {
                        format!("&${}", fv)
                    } else {
                        format!("${}", fv)
                    };
                    ret_expr = format!(
                        "{}\nforeach ({} as {})",
                        cond_expr, cv, tmp_foreach_var
                    );
                }
            }
            "while" => {
                if let Some(ConditionExpr::Single(cond_idx)) = self.conditional_statements {
                    let mut tmp = Vec::new();
                    get_expression(rel_db, cond_idx, &mut tmp);
                    let cond_expr = tmp.join("\n");
                    let cv = self.conditional_variable.as_deref().unwrap_or("");
                    ret_expr = format!("{} \nwhile({})", cond_expr, cv);
                }
            }
            "for" => {
                if let Some(ConditionExpr::Multiple(items)) = &self.conditional_statements {
                    let mut tmp = Vec::new();
                    // items[0] is init (NodeRef), items[1] is cond (Literal), items[2] is loop (Literal)
                    if items.len() >= 3 {
                        if let CondItem::NodeRef(init_idx) = &items[0] {
                            get_expression(rel_db, *init_idx, &mut tmp);
                        }
                        let init_str = tmp.join("");
                        let cond = match &items[1] {
                            CondItem::Literal(s) => s.clone(),
                            CondItem::NodeRef(idx) => {
                                let mut t = Vec::new();
                                get_expression(rel_db, *idx, &mut t);
                                t.join("")
                            }
                        };
                        let loop_str = match &items[2] {
                            CondItem::Literal(s) => s.clone(),
                            CondItem::NodeRef(idx) => {
                                let mut t = Vec::new();
                                get_expression(rel_db, *idx, &mut t);
                                t.join("")
                            }
                        };
                        ret_expr = format!("for({}{};{})", init_str, cond, loop_str);
                    }
                }
            }
            "finally" => {
                ret_expr = "finally".to_string();
            }
            _ => {}
        }

        // Add body
        if self.flow_type != "case" && self.flow_type != "case_default" {
            ret_expr.push_str("{\n");
            let mut tmp_v = Vec::new();
            let keys: Vec<EnvKey> = self.node_env.keys().cloned().collect();
            for key in keys {
                let value = self.node_env.get(&key).unwrap().clone();
                // Only output values not in parent env
                if let Some(parent_val) = parent_node_env.get(&key) {
                    if *parent_val == value {
                        continue;
                    }
                }

                match value {
                    EnvValue::Node(idx) => {
                        get_expression(rel_db, idx, &mut tmp_v);
                    }
                    EnvValue::Flow(flow_idx) => {
                        let mut flow = self.flow_branches[flow_idx].clone();
                        flow.get_expression(rel_db, &self.node_env, &mut tmp_v);
                        self.flow_branches[flow_idx] = flow;
                    }
                    EnvValue::Direct(mut node) => {
                        node_get_expression_direct(&mut node, rel_db, &mut tmp_v);
                    }
                }
            }
            let body = tmp_v.join("\n");
            ret_expr.push_str(&body);
            ret_expr.push_str("\n}");
        } else {
            let mut tmp_v = Vec::new();
            let keys: Vec<EnvKey> = self.node_env.keys().cloned().collect();
            for key in keys {
                let value = self.node_env.get(&key).unwrap().clone();
                if let Some(parent_val) = parent_node_env.get(&key) {
                    if *parent_val == value {
                        continue;
                    }
                }
                match value {
                    EnvValue::Node(idx) => {
                        get_expression(rel_db, idx, &mut tmp_v);
                    }
                    EnvValue::Flow(flow_idx) => {
                        let mut flow = self.flow_branches[flow_idx].clone();
                        flow.get_expression(rel_db, &self.node_env, &mut tmp_v);
                        self.flow_branches[flow_idx] = flow;
                    }
                    EnvValue::Direct(mut node) => {
                        node_get_expression_direct(&mut node, rel_db, &mut tmp_v);
                    }
                }
            }
            let body = tmp_v.join("\n");
            ret_expr.push_str(&body);
        }

        output_str.push(ret_expr);
    }
}

/// Compare two EnvValues for identity (same variant and same index)
#[allow(dead_code)]
fn env_value_eq(a: &EnvValue, b: &EnvValue) -> bool {
    match (a, b) {
        (EnvValue::Node(a_idx), EnvValue::Node(b_idx)) => a_idx == b_idx,
        (EnvValue::Flow(a_idx), EnvValue::Flow(b_idx)) => a_idx == b_idx,
        _ => false,
    }
}

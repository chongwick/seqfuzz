use crate::lang::LangDescription;
use crate::nodes::*;
use crate::sequence::*;
use serde_json::Value;
use std::process::Command;

#[allow(dead_code)]
pub struct Walker {
    pub lang: LangDescription,
    pub debug: bool,
    pub file_string: Option<String>,
    pub master: Seq,
    dangling_call: bool,
    curr_obj_name: Option<String>,
}

#[allow(dead_code)]
impl Walker {
    pub fn new(debug: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let lang = LangDescription::load()?;
        Ok(Walker {
            lang,
            debug,
            file_string: None,
            master: Seq::new(),
            dangling_call: true,
            curr_obj_name: None,
        })
    }

    pub fn new_with_lang(lang: LangDescription, debug: bool) -> Self {
        Walker {
            lang,
            debug,
            file_string: None,
            master: Seq::new(),
            dangling_call: true,
            curr_obj_name: None,
        }
    }

    fn get_node_type(node: &Value) -> &str {
        node["nodeType"].as_str().unwrap_or("")
    }

    fn parse_file_string(&self, start: usize, stop: usize) -> String {
        if let Some(ref fs) = self.file_string {
            fs[start..=stop].to_string()
        } else {
            String::new()
        }
    }

    pub fn analyze(&mut self, target_file: &str) -> (bool, &mut Seq) {
        // Read the file
        self.file_string = std::fs::read_to_string(target_file).ok();

        // Convert to AST via php_to_ast.sh
        let stmts = match Command::new("bash")
            .args(["./php_to_ast.sh", target_file])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Value>(&stdout) {
                    Ok(Value::Array(arr)) => arr,
                    _ => return (false, &mut self.master),
                }
            }
            Err(_) => return (false, &mut self.master),
        };

        if self.debug {
            for node in &stmts {
                self.eval_node(node);
            }
            return (false, &mut self.master);
        }

        for node in &stmts {
            if self.eval_node(node).is_none() {
                // Some nodes return None normally (statements)
            }
        }
        (true, &mut self.master)
    }

    /// Main eval dispatcher. Returns Some(rel_db_idx) for expression nodes,
    /// None for statement nodes.
    fn eval_node(&mut self, node: &Value) -> Option<usize> {
        self.eval_node_in_env(node, EnvTarget::Root)
    }

    fn eval_node_in_env(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        if self.debug {
            eprintln!("eval_node: {}", node_type);
        }

        // Check op node maps first
        if self.lang.middle_ops.contains_key(node_type) {
            return self.eval_middle_op(node, env);
        }
        if self.lang.front_ops.contains_key(node_type) {
            return self.eval_front_op(node, env);
        }
        if self.lang.rear_ops.contains_key(node_type) {
            return self.eval_rear_op(node, env);
        }
        if self.lang.inner_ops.contains_key(node_type) {
            return self.eval_inner_op(node, env);
        }

        match node_type {
            "Scalar_Int" | "Scalar_LNumber" | "Scalar_Float" => {
                self.eval_constant(node)
            }
            "Stmt_Declare" | "Expr_Eval" | "Stmt_Use" | "Stmt_Interface"
            | "Stmt_Namespace" | "Expr_Isset" | "Stmt_Enum" => {
                self.eval_parse(node, env);
                None
            }
            "Stmt_Expression" => self.eval_stmt_expression(node, env),
            "Expr_Assign" => {
                self.eval_expr_assign(node, env);
                None
            }
            "Expr_Variable" => self.eval_expr_variable(node, env),
            "Expr_FuncCall" => self.eval_expr_func_call(node, env),
            "Stmt_Function" => {
                self.eval_stmt_function(node, env);
                None
            }
            "Stmt_Class" => {
                self.eval_stmt_class(node);
                None
            }
            "Expr_New" => self.eval_expr_new(node, env),
            "Expr_StaticCall" => self.eval_expr_static_call(node, env),
            "Expr_MethodCall" => self.eval_expr_method_call(node, env),
            "Expr_PropertyFetch" => self.eval_expr_property_fetch(node, env),
            "Scalar_String" => self.eval_scalar_string(node),
            "Scalar_InterpolatedString" => self.eval_scalar_interpolated_string(node, env),
            "Scalar_MagicConst_Dir" => self.eval_magic_const("__DIR__"),
            "Scalar_MagicConst_File" => self.eval_magic_const("__FILE__"),
            "Expr_ConstFetch" => self.eval_expr_const_fetch(node),
            "Expr_ClassConstFetch" => self.eval_expr_class_const_fetch(node),
            "Expr_Array" => self.eval_expr_array(node, env),
            "ArrayItem" => {
                // Special: returns (Option<idx>, idx) but we pack into single idx
                // Handle at call site
                None
            }
            "Expr_ArrayDimFetch" => self.eval_expr_array_dim_fetch(node, env),
            "Stmt_If" => {
                self.eval_stmt_if(node, env);
                None
            }
            "Stmt_ElseIf" => {
                self.eval_stmt_elseif(node, env);
                None
            }
            "Stmt_Else" => {
                self.eval_stmt_else(node, env);
                None
            }
            "Stmt_While" => {
                self.eval_stmt_while(node, env);
                None
            }
            "Stmt_For" => {
                self.eval_stmt_for(node, env);
                None
            }
            "Stmt_Foreach" => {
                self.eval_stmt_foreach(node, env);
                None
            }
            "Stmt_TryCatch" => {
                self.eval_stmt_try_catch(node, env);
                None
            }
            "Stmt_Catch" => {
                self.eval_stmt_catch(node, env);
                None
            }
            "Stmt_Finally" => {
                self.eval_stmt_finally(node, env);
                None
            }
            "Stmt_Switch" => {
                self.eval_stmt_switch(node, env);
                None
            }
            "Stmt_Return" => {
                self.eval_stmt_return(node, env);
                None
            }
            "Expr_Exit" => {
                self.eval_expr_exit(node, env);
                None
            }
            "Expr_Clone" => {
                self.eval_expr_clone(node, env);
                None
            }
            "Expr_Yield" => {
                self.eval_expr_yield(node, env);
                None
            }
            "Expr_YieldFrom" => {
                self.eval_expr_yield_from(node, env);
                None
            }
            "Expr_Closure" => self.eval_expr_closure(node),
            "Expr_Ternary" => self.eval_expr_ternary(node),
            "VariadicPlaceholder" => self.eval_variadic_placeholder(),
            "Stmt_Nop" | "Stmt_InlineHTML" | "Expr_Include" | "Expr_Throw" => None,
            "Stmt_Break" => {
                self.eval_stmt_break(env);
                None
            }
            "Stmt_Echo" => {
                self.eval_stmt_echo(node, env);
                None
            }
            "Expr_Print" => {
                self.eval_expr_print(node, env);
                None
            }
            "Name_FullyQualified" => {
                self.eval_name_fully_qualified(node)
            }
            _ => {
                if self.debug {
                    eprintln!(
                        "No handler for: {}",
                        node_type
                    );
                }
                None
            }
        }
    }

    // ---- Environment access helpers ----
    // These abstract over which env (root, function, flow) we're targeting.

    fn env_set_node(&mut self, env: EnvTarget, name: Option<&str>, value: EnvValue) {
        match env {
            EnvTarget::Root => self.master.set_node(name, value),
            EnvTarget::Function(ref fname) => {
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.set_node(name, value);
                }
            }
            EnvTarget::Flow(ref path) => {
                if let Some(flow) = self.get_flow_mut(path) {
                    flow.set_node(name, value);
                }
            }
            EnvTarget::FuncFlow(ref fname, idx) => {
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.flow_branches[idx].set_node(name, value);
                }
            }
            EnvTarget::FuncFlowNested(ref fname, pidx, cidx) => {
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.flow_branches[pidx].flow_branches[cidx].set_node(name, value);
                }
            }
        }
    }

    fn env_set_node_with_rename(&mut self, env: EnvTarget, name: &str, rel_db_idx: usize) {
        match env {
            EnvTarget::Root => self.master.set_node_with_rename(name, rel_db_idx),
            EnvTarget::Function(ref fname) => {
                let rel_db = &mut self.master.relational_database;
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.set_node_with_rename(name, rel_db, rel_db_idx);
                }
            }
            EnvTarget::Flow(ref path) => {
                let path = path.clone();
                // Navigate to the flow directly through master to avoid borrow conflict
                if !path.is_empty() {
                    let rel_db = &mut self.master.relational_database;
                    let mut flow_ref = &mut self.master.flow_branches[path[0]];
                    for &idx in &path[1..] {
                        flow_ref = &mut flow_ref.flow_branches[idx];
                    }
                    flow_ref.set_node_with_rename(name, rel_db, rel_db_idx);
                }
            }
            EnvTarget::FuncFlow(ref fname, idx) => {
                let rel_db = &mut self.master.relational_database;
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.flow_branches[idx].set_node_with_rename(name, rel_db, rel_db_idx);
                }
            }
            EnvTarget::FuncFlowNested(ref fname, pidx, cidx) => {
                let rel_db = &mut self.master.relational_database;
                if let Some(fenv) = self.master.function_envs.get_mut(fname) {
                    fenv.flow_branches[pidx].flow_branches[cidx].set_node_with_rename(name, rel_db, rel_db_idx);
                }
            }
        }
    }

    fn env_get_node_rel_idx(&self, env: &EnvTarget, name: &str) -> Option<usize> {
        match env {
            EnvTarget::Root => self.master.get_node_rel_idx(name),
            EnvTarget::Function(ref fname) => {
                self.master.function_envs.get(fname)?.get_node_rel_idx(name)
            }
            EnvTarget::Flow(ref path) => {
                self.get_flow(path)?.get_node_rel_idx(name)
            }
            EnvTarget::FuncFlow(ref fname, idx) => {
                self.master.function_envs.get(fname)?.flow_branches.get(*idx)?.get_node_rel_idx(name)
            }
            EnvTarget::FuncFlowNested(ref fname, pidx, cidx) => {
                self.master.function_envs.get(fname)?.flow_branches.get(*pidx)?
                    .flow_branches.get(*cidx)?.get_node_rel_idx(name)
            }
        }
    }

    fn env_has_node(&self, env: &EnvTarget, name: &str) -> bool {
        self.env_get_node_rel_idx(env, name).is_some()
    }

    fn env_get_node_env(&self, env: &EnvTarget) -> &IndexMap<EnvKey, EnvValue> {
        match env {
            EnvTarget::Root => &self.master.node_env,
            EnvTarget::Function(ref fname) => {
                &self.master.function_envs.get(fname).unwrap().node_env
            }
            EnvTarget::Flow(ref path) => {
                &self.get_flow(path).unwrap().node_env
            }
            EnvTarget::FuncFlow(ref fname, idx) => {
                &self.master.function_envs.get(fname).unwrap().flow_branches[*idx].node_env
            }
            EnvTarget::FuncFlowNested(ref fname, pidx, cidx) => {
                &self.master.function_envs.get(fname).unwrap().flow_branches[*pidx].flow_branches[*cidx].node_env
            }
        }
    }

    fn env_spawn_flow(&mut self, env: EnvTarget, flow_type: &str) -> (EnvTarget, usize) {
        match env {
            EnvTarget::Root => {
                let idx = self.master.spawn_flow(flow_type);
                (EnvTarget::Flow(vec![idx]), idx)
            }
            EnvTarget::Function(ref fname) => {
                let fname = fname.clone();
                let parent_env = self.master.function_envs.get(&fname).unwrap().node_env.clone();
                let fenv = self.master.function_envs.get_mut(&fname).unwrap();
                let idx = fenv.spawn_flow(flow_type, &parent_env);
                (EnvTarget::FuncFlow(fname, idx), idx)
            }
            EnvTarget::Flow(ref path) => {
                let path = path.clone();
                let flow = self.get_flow_mut(&path).unwrap();
                let idx = flow.spawn_flow(flow_type);
                let mut new_path = path;
                new_path.push(idx);
                (EnvTarget::Flow(new_path), idx)
            }
            EnvTarget::FuncFlow(ref fname, ref parent_idx) => {
                let fname = fname.clone();
                let parent_idx = *parent_idx;
                let fenv = self.master.function_envs.get_mut(&fname).unwrap();
                let parent_flow = &mut fenv.flow_branches[parent_idx];
                let idx = parent_flow.spawn_flow(flow_type);
                (EnvTarget::FuncFlowNested(fname, parent_idx, idx), idx)
            }
            EnvTarget::FuncFlowNested(ref fname, ref parent_idx, ref child_idx) => {
                let fname = fname.clone();
                let parent_idx = *parent_idx;
                let child_idx = *child_idx;
                let fenv = self.master.function_envs.get_mut(&fname).unwrap();
                let parent_flow = &mut fenv.flow_branches[parent_idx];
                let child_flow = &mut parent_flow.flow_branches[child_idx];
                let idx = child_flow.spawn_flow(flow_type);
                // We don't go deeper than 3 levels for simplicity
                (EnvTarget::FuncFlowNested(fname, parent_idx, idx), idx)
            }
        }
    }

    fn get_flow(&self, path: &[usize]) -> Option<&FlowBranch> {
        if path.is_empty() {
            return None;
        }
        let mut flow = self.master.flow_branches.get(path[0])?;
        for &idx in &path[1..] {
            flow = flow.flow_branches.get(idx)?;
        }
        Some(flow)
    }

    fn get_flow_mut(&mut self, path: &[usize]) -> Option<&mut FlowBranch> {
        if path.is_empty() {
            return None;
        }
        let mut flow = self.master.flow_branches.get_mut(path[0])?;
        for &idx in &path[1..] {
            flow = flow.flow_branches.get_mut(idx)?;
        }
        Some(flow)
    }

    fn set_flow_condition(&mut self, env: &EnvTarget, cond: Option<ConditionExpr>, cond_var: String) {
        match env {
            EnvTarget::Flow(ref path) => {
                if let Some(flow) = self.get_flow_mut(path) {
                    flow.set_condition(cond, cond_var);
                }
            }
            EnvTarget::FuncFlow(ref fname, idx) => {
                let fenv = self.master.function_envs.get_mut(fname).unwrap();
                fenv.flow_branches[*idx].set_condition(cond, cond_var);
            }
            EnvTarget::FuncFlowNested(ref fname, pidx, cidx) => {
                let fenv = self.master.function_envs.get_mut(fname).unwrap();
                fenv.flow_branches[*pidx].flow_branches[*cidx].set_condition(cond, cond_var);
            }
            _ => {}
        }
    }

    // ---- Eval methods ----

    fn eval_stmt_expression(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        self.dangling_call = true;
        self.eval_node_in_env(&node["expr"], env)
    }

    fn eval_expr_assign(&mut self, node: &Value, env: EnvTarget) {
        self.dangling_call = false;
        let var = &node["var"];
        let expr = &node["expr"];
        let var_type = Self::get_node_type(var);

        #[allow(unused_assignments)]
        let mut name: Option<String> = None;

        match var_type {
            "Expr_Variable" => {
                name = var["name"].as_str().map(|s| s.to_string());
            }
            "Expr_PropertyFetch" => {
                let mut parts = Vec::new();
                let mut current = var;
                while Self::get_node_type(current) == "Expr_PropertyFetch" {
                    if let Some(n) = current["name"]["name"].as_str() {
                        parts.push(n.to_string());
                    }
                    current = &current["var"];
                }
                if let Some(n) = current["name"].as_str() {
                    parts.push(n.to_string());
                }
                parts.reverse();
                name = Some(parts.join("->"));
            }
            "Expr_ArrayDimFetch" => {
                let array_var_idx = self.eval_node_in_env(&var["var"], env.clone());
                let array_dim_idx = if !var["dim"].is_null() {
                    self.eval_node_in_env(&var["dim"], env.clone())
                } else {
                    let null_node = SeqNode::Generic {
                        value: NodeValue::Str("null".to_string()),
                        node_type: None,
                        name: new_name(),
                        printed: false,
                    };
                    let idx = self.master.set_relation(null_node);
                    self.master.check_in(idx);
                    Some(idx)
                };

                let expr_type = Self::get_node_type(expr);
                if expr_type == "Expr_New" || expr_type == "Expr_MethodCall" || expr_type == "Expr_StaticCall" {
                    // set curr_obj_name - not applicable for array dim fetch
                }
                let evaled_idx = self.eval_node_in_env(expr, env.clone());

                if let (Some(var_idx), Some(dim_idx), Some(eval_idx)) = (array_var_idx, array_dim_idx, evaled_idx) {
                    // Check if var is already an Array node
                    let is_array = self.master.relational_database[var_idx].is_array();
                    if !is_array {
                        // Create a new Array node
                        let arr_node = SeqNode::Array {
                            values: indexmap::IndexMap::new(),
                            name: new_name(),
                            printed: false,
                        };
                        let arr_idx = self.master.set_relation(arr_node);
                        // Replace in env
                        if let Some(var_name) = var["var"]["name"].as_str() {
                            self.env_set_node(env.clone(), Some(var_name), EnvValue::Node(arr_idx));
                        }
                        // Add value to array
                        let _dim_rel = self.master.set_relation(self.master.relational_database[dim_idx].clone());
                        let eval_rel = self.master.set_relation(self.master.relational_database[eval_idx].clone());
                        let dim_key = self.master.relational_database[dim_idx]
                            .get_node_value()
                            .map(|v| v.to_php_string());
                        self.master.relational_database[arr_idx].add_array_value(dim_key, eval_rel);
                    } else {
                        let dim_key = self.master.relational_database[dim_idx]
                            .get_node_value()
                            .map(|v| v.to_php_string());
                        let eval_rel = self.master.set_relation(self.master.relational_database[eval_idx].clone());
                        self.master.relational_database[var_idx].add_array_value(dim_key, eval_rel);
                    }
                }
                return;
            }
            "Expr_List" => {
                // Handle list() assignment - just eval the expression
                self.eval_node_in_env(expr, env);
                return;
            }
            _ => return,
        }

        let expr_type = Self::get_node_type(expr);
        if expr_type == "Expr_New" || expr_type == "Expr_MethodCall" || expr_type == "Expr_StaticCall" {
            self.curr_obj_name = name.clone();
        }

        let evaled_idx = self.eval_node_in_env(expr, env.clone());

        if let (Some(n), Some(idx)) = (name, evaled_idx) {
            self.env_set_node_with_rename(env, &n, idx);
        }
    }

    fn eval_expr_variable(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let name = node["name"].as_str()?;
        self.env_get_node_rel_idx(&env, name)
    }

    fn eval_middle_op(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        let left_idx = self.eval_node_in_env(&node["left"], env.clone())?;
        let right_idx = self.eval_node_in_env(&node["right"], env)?;

        let left_rel = self.master.set_relation(self.master.relational_database[left_idx].clone());
        let right_rel = self.master.set_relation(self.master.relational_database[right_idx].clone());

        let op = self.lang.middle_ops.get(node_type).cloned().unwrap_or_default();
        let ret_node = SeqNode::MiddleOp {
            left_key: left_rel,
            right_key: right_rel,
            value: op,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_front_op(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        let op = self.lang.front_ops.get(node_type).cloned().unwrap_or_default();

        if node_type == "Stmt_Unset" {
            if let Some(vars) = node["vars"].as_array() {
                for v in vars {
                    let node_idx = if Self::get_node_type(v) == "Expr_PropertyFetch" {
                        // Build a property access node
                        let var_name = v["var"]["name"].as_str().unwrap_or("");
                        if let Some(var_idx) = self.env_get_node_rel_idx(&env, var_name) {
                            let prop_name = v["name"]["name"].as_str().unwrap_or("");
                            let var_node_name = self.master.relational_database[var_idx]
                                .get_name()
                                .unwrap_or("")
                                .to_string();
                            let gen_node = SeqNode::Generic {
                                value: NodeValue::Str(format!("{}->{}",  var_node_name, prop_name)),
                                node_type: None,
                                name: new_name(),
                                printed: false,
                            };
                            Some(self.master.set_relation(gen_node))
                        } else {
                            None
                        }
                    } else {
                        let vname = v["name"].as_str().unwrap_or("");
                        self.env_get_node_rel_idx(&env, vname)
                    };

                    if let Some(nidx) = node_idx {
                        let node_rel = self.master.set_relation(
                            self.master.relational_database[nidx].clone(),
                        );
                        let ret_node = SeqNode::FrontOp {
                            node_key: node_rel,
                            value: op.clone(),
                            name: new_name(),
                            printed: false,
                        };
                        let ridx = self.master.set_relation(ret_node);
                        self.master.check_in(ridx);
                        self.env_set_node(env.clone(), None, EnvValue::Node(ridx));
                    }
                }
            }
            return None;
        }

        let node_val_idx = if node.get("expr").is_some() && !node["expr"].is_null() {
            self.eval_node_in_env(&node["expr"], env.clone())
        } else {
            self.eval_node_in_env(&node["var"], env.clone())
        };

        if let Some(nidx) = node_val_idx {
            let node_rel = self.master.set_relation(self.master.relational_database[nidx].clone());
            let ret_node = SeqNode::FrontOp {
                node_key: node_rel,
                value: op,
                name: new_name(),
                printed: false,
            };
            let idx = self.master.set_relation(ret_node);
            self.master.check_in(idx);
            Some(idx)
        } else {
            None
        }
    }

    fn eval_inner_op(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        let op = self.lang.inner_ops.get(node_type).cloned().unwrap_or_default();

        let node_val_idx = self.eval_node_in_env(&node["expr"], env.clone())?;
        let node_rel = self.master.set_relation(self.master.relational_database[node_val_idx].clone());

        let ret_node = SeqNode::InnerOp {
            node_key: node_rel,
            value: op,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.env_set_node(env, None, EnvValue::Node(idx));
        self.master.check_in(idx);
        None
    }

    fn eval_rear_op(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        let op = self.lang.rear_ops.get(node_type).cloned().unwrap_or_default();

        let node_val_idx = self.eval_node_in_env(&node["var"], env.clone())?;
        let node_rel = self.master.set_relation(self.master.relational_database[node_val_idx].clone());

        let ret_node = SeqNode::RearOp {
            node_key: node_rel,
            value: op,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.env_set_node(env, None, EnvValue::Node(idx));
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_constant(&mut self, node: &Value) -> Option<usize> {
        let node_type = Self::get_node_type(node);
        let value = if node_type == "Scalar_Float" {
            NodeValue::Float(node["value"].as_f64().unwrap_or(0.0))
        } else {
            NodeValue::Int(node["value"].as_i64().unwrap_or(0))
        };
        let ret_node = SeqNode::Generic {
            value,
            node_type: Some(node_type.to_string()),
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_scalar_string(&mut self, node: &Value) -> Option<usize> {
        let raw = node["value"].as_str().unwrap_or("");
        // Use repr-like quoting
        let value = NodeValue::Str(format!("'{}'", raw.replace('\'', "\\'")));
        let ret_node = SeqNode::Generic {
            value,
            node_type: Some("Scalar_String".to_string()),
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_scalar_interpolated_string(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let mut parts = Vec::new();
        if let Some(node_parts) = node["parts"].as_array() {
            for part in node_parts {
                if Self::get_node_type(part) == "InterpolatedStringPart" {
                    let raw = part["value"].as_str().unwrap_or("");
                    parts.push(StringPart::Literal(format!("'{}'", raw.replace('\'', "\\'"))));
                } else {
                    if let Some(idx) = self.eval_node_in_env(part, env.clone()) {
                        let rel_idx = self.master.set_relation(
                            self.master.relational_database[idx].clone(),
                        );
                        parts.push(StringPart::NodeRef(rel_idx));
                    }
                }
            }
        }
        let ret_node = SeqNode::InterpolatedString {
            parts,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        Some(idx)
    }

    fn eval_magic_const(&mut self, value: &str) -> Option<usize> {
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str(value.to_string()),
            node_type: Some(value.to_string()),
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_name_fully_qualified(&mut self, node: &Value) -> Option<usize> {
        let name = node["name"].as_str().unwrap_or("");
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str(format!("\\{}", name)),
            node_type: Some("Name_FullyQualified".to_string()),
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_func_call(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let func_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        let dependency = if self.master.function_envs.contains_key(&func_name) {
            Some(func_name.clone())
        } else {
            None
        };

        let mut params = Vec::new();
        if let Some(args) = node["args"].as_array() {
            for v in args {
                let val_node = if v.get("value").is_some() && !v["value"].is_null() {
                    &v["value"]
                } else {
                    v
                };
                if let Some(idx) = self.eval_node_in_env(val_node, env.clone()) {
                    let rel_idx = self.master.set_relation(
                        self.master.relational_database[idx].clone(),
                    );
                    params.push(rel_idx);
                }
            }
        }

        let ret_node = SeqNode::Func {
            func_name: func_name.clone(),
            arguments: params,
            dependency,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);

        if self.dangling_call {
            self.dangling_call = false;
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
        Some(idx)
    }

    fn eval_stmt_function(&mut self, node: &Value, _env: EnvTarget) {
        let func_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        self.master.spawn_function(&func_name);

        // Process params
        let mut func_params: IndexMap<String, usize> = IndexMap::new();
        if let Some(params) = node["params"].as_array() {
            for p in params {
                let param_name = p["var"]["name"].as_str().unwrap_or("").to_string();
                if p["default"].is_null() {
                    let empty_node = SeqNode::Generic {
                        value: NodeValue::None,
                        node_type: Some("param".to_string()),
                        name: new_name(),
                        printed: false,
                    };
                    let idx = self.master.set_relation(empty_node);
                    self.master.check_in(idx);
                    func_params.insert(param_name, idx);
                } else {
                    let v_idx = self.eval_node_in_env(
                        &p["default"],
                        EnvTarget::Function(func_name.clone()),
                    );
                    if let Some(vi) = v_idx {
                        self.master.relational_database[vi].set_node_type("param".to_string());
                        func_params.insert(param_name, vi);
                    }
                }
            }
        }

        // Set params in env
        for (k, v) in &func_params {
            let fenv = self.master.function_envs.get_mut(&func_name).unwrap();
            fenv.set_node(Some(k), EnvValue::Node(*v));
        }

        // Store param rel_db indices
        let mut param_indices = IndexMap::new();
        for (k, v) in &func_params {
            let rel_idx = self.master.set_relation(
                self.master.relational_database[*v].clone(),
            );
            param_indices.insert(k.clone(), rel_idx);
        }
        let fenv = self.master.function_envs.get_mut(&func_name).unwrap();
        fenv.set_params(param_indices);

        // Process body
        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, EnvTarget::Function(func_name.clone()));
            }
        }
    }

    fn eval_stmt_class(&mut self, node: &Value) {
        let start = node["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
        let end = node["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
        let expression_string = self.parse_file_string(start, end);
        let class_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        self.master.add_class_def(class_name, expression_string);
    }

    fn eval_expr_new(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let class_name = node["class"]["name"].as_str().unwrap_or("").to_string();
        let mut arg_list = Vec::new();
        if let Some(args) = node["args"].as_array() {
            for v in args {
                if let Some(idx) = self.eval_node_in_env(&v["value"], env.clone()) {
                    let rel_idx = self.master.set_relation(
                        self.master.relational_database[idx].clone(),
                    );
                    arg_list.push(rel_idx);
                }
            }
        }
        let class_def = self.master.get_class_def(&class_name).cloned();
        let obj_name = self.curr_obj_name.take();

        let ret_node = SeqNode::ObjDec {
            obj_name: obj_name.clone(),
            class_name,
            arguments: arg_list,
            dependency: class_def,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);

        if let Some(ref on) = obj_name {
            self.master.add_object(on.clone(), idx);
        }
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_static_call(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let class_name = node["class"]["name"].as_str().unwrap_or("").to_string();
        let call_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        let full_name = format!("{}::{}", class_name, call_name);

        let mut arg_list = Vec::new();
        if let Some(args) = node["args"].as_array() {
            for v in args {
                if let Some(idx) = self.eval_node_in_env(&v["value"], env.clone()) {
                    let rel_idx = self.master.set_relation(
                        self.master.relational_database[idx].clone(),
                    );
                    arg_list.push(rel_idx);
                }
            }
        }
        let class_def = self.master.get_class_def(&class_name).cloned();
        let obj_name = self.curr_obj_name.take();

        let ret_node = SeqNode::StaticCall {
            obj_name: obj_name.clone(),
            class_name: full_name,
            arguments: arg_list,
            dependency: class_def,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);

        if let Some(ref on) = obj_name {
            self.master.add_object(on.clone(), idx);
        }
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_method_call(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let mut dependency: Option<NodeId> = None;
        #[allow(unused_assignments)]
        let mut obj_name = String::new();

        if Self::get_node_type(&node["var"]) == "Expr_StaticCall" {
            if let Some(dep_idx) = self.eval_node_in_env(&node["var"], env.clone()) {
                dependency = Some(NodeId(dep_idx));
            }
        }

        if let Some(name) = node["var"]["name"].as_str() {
            obj_name = name.to_string();
        } else {
            obj_name = new_name().replace('$', "");
            self.curr_obj_name = Some(obj_name.clone());
            if let Some(obj_dec_idx) = self.eval_node_in_env(&node["var"], env.clone()) {
                self.curr_obj_name = None;
                self.env_set_node(env.clone(), Some(&obj_name), EnvValue::Node(obj_dec_idx));
            }
        }

        let method_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        let mut arg_list = Vec::new();
        if let Some(args) = node["args"].as_array() {
            for v in args {
                if let Some(idx) = self.eval_node_in_env(&v["value"], env.clone()) {
                    let rel_idx = self.master.set_relation(
                        self.master.relational_database[idx].clone(),
                    );
                    arg_list.push(rel_idx);
                }
            }
        }

        // Check if we need to get object dependency
        let is_catch_env = match &env {
            EnvTarget::Flow(path) => {
                self.get_flow(path)
                    .map(|f| f.flow_type == "catch")
                    .unwrap_or(false)
            }
            _ => false,
        };

        let is_foreach_self = match &env {
            EnvTarget::Flow(path) => {
                self.get_flow(path)
                    .map(|f| f.flow_type == "foreach" && f.foreach_var.as_deref() == Some(&obj_name))
                    .unwrap_or(false)
            }
            _ => false,
        };

        if !is_catch_env && dependency.is_none() && !is_foreach_self {
            if let Some(obj_idx) = self.master.get_object(&obj_name) {
                dependency = Some(NodeId(obj_idx));
            }
        }

        let ret_node = SeqNode::Method {
            obj_name: obj_name.clone(),
            method_name,
            arguments: arg_list,
            dependency,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);

        if self.curr_obj_name.is_none() {
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
        Some(idx)
    }

    fn eval_expr_property_fetch(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let obj_name = if Self::get_node_type(&node["var"]) != "Expr_Variable" {
            if let Some(idx) = self.eval_node_in_env(&node["var"], env.clone()) {
                self.master.relational_database[idx]
                    .get_name()
                    .unwrap_or("")
                    .to_string()
            } else {
                return None;
            }
        } else {
            node["var"]["name"].as_str().unwrap_or("").to_string()
        };

        let property_name = node["name"]["name"].as_str().unwrap_or("").to_string();
        let full_name = format!("{}{}", obj_name, property_name);

        if self.env_has_node(&env, &full_name) {
            return self.env_get_node_rel_idx(&env, &full_name);
        }

        let dep = self.master.get_object(&obj_name).map(NodeId);
        let ret_node = SeqNode::PropertyFetch {
            obj_name,
            property_name,
            dependency: dep,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        self.env_set_node(env, Some(&full_name), EnvValue::Node(idx));
        Some(idx)
    }

    fn eval_expr_const_fetch(&mut self, node: &Value) -> Option<usize> {
        let name_node = &node["name"];
        let ret_node = if Self::get_node_type(name_node) == "name_Relative" {
            let value = name_node["name"].as_str().unwrap_or("");
            SeqNode::Generic {
                value: NodeValue::Str(format!("namespace\\{}", value)),
                node_type: Some("Expr_ConstFetch".to_string()),
                name: new_name(),
                printed: false,
            }
        } else {
            let value = name_node["name"].as_str().unwrap_or("");
            let node_value = match value {
                "true" => NodeValue::Bool(true),
                "false" => NodeValue::Bool(false),
                _ => NodeValue::Str(value.to_string()),
            };
            SeqNode::Generic {
                value: node_value,
                node_type: Some("Expr_ConstFetch".to_string()),
                name: new_name(),
                printed: false,
            }
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_class_const_fetch(&mut self, node: &Value) -> Option<usize> {
        let class_name = node["class"]["name"].as_str().unwrap_or("");
        let const_name = node["name"]["name"].as_str().unwrap_or("");
        let full_name = format!("{}::{}", class_name, const_name);
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str(full_name),
            node_type: Some("Expr_ClassConstFetch".to_string()),
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_array(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let mut arr_node = SeqNode::Array {
            values: indexmap::IndexMap::new(),
            name: new_name(),
            printed: false,
        };

        if let Some(items) = node["items"].as_array() {
            for item in items {
                let key_idx = if !item["key"].is_null() {
                    self.eval_node_in_env(&item["key"], env.clone())
                } else {
                    None
                };
                let val_idx = self.eval_node_in_env(&item["value"], env.clone());

                if let Some(vi) = val_idx {
                    let val_rel = self.master.set_relation(
                        self.master.relational_database[vi].clone(),
                    );

                    let key_str = if let Some(ki) = key_idx {
                        let key_rel = self.master.set_relation(
                            self.master.relational_database[ki].clone(),
                        );
                        let _ = key_rel;
                        self.master.relational_database[ki]
                            .get_node_value()
                            .map(|v| v.to_php_string())
                    } else {
                        None
                    };

                    arr_node.add_array_value(key_str, val_rel);
                }
            }
        }

        let idx = self.master.set_relation(arr_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_array_dim_fetch(&mut self, node: &Value, env: EnvTarget) -> Option<usize> {
        let var_idx = self.eval_node_in_env(&node["var"], env.clone())?;
        let dim_idx = self.eval_node_in_env(&node["dim"], env)?;

        if !self.master.relational_database[var_idx].is_array() {
            let ret_node = SeqNode::FlexArrayDimFetch {
                var: NodeId(var_idx),
                dim: NodeId(dim_idx),
                name: new_name(),
                printed: false,
            };
            let idx = self.master.set_relation(ret_node);
            self.master.check_in(idx);
            Some(idx)
        } else {
            // Get value by key from array
            let dim_key = self.master.relational_database[dim_idx]
                .get_node_value()
                .map(|v| v.to_php_string())
                .unwrap_or_default();
            let arr_name = self.master.relational_database[var_idx]
                .get_name()
                .unwrap_or("")
                .to_string();
            let ret_node = SeqNode::ArrayValue {
                value: dim_key,
                arr_name,
                dependency: Some(NodeId(var_idx)),
                name: new_name(),
                printed: false,
            };
            let idx = self.master.set_relation(ret_node);
            self.master.check_in(idx);
            Some(idx)
        }
    }

    fn eval_stmt_if(&mut self, node: &Value, env: EnvTarget) {
        if let Some(cond_idx) = self.eval_node_in_env(&node["cond"], env.clone()) {
            let cond_name = self.master.relational_database[cond_idx]
                .get_name()
                .unwrap_or("")
                .to_string();
            let (flow_env, _flow_idx) = self.env_spawn_flow(env.clone(), "if");
            self.set_flow_condition(
                &flow_env,
                Some(ConditionExpr::Single(cond_idx)),
                cond_name,
            );

            // Process elseifs
            if let Some(elseifs) = node["elseifs"].as_array() {
                for stmt in elseifs {
                    self.eval_node_in_env(stmt, env.clone());
                }
            }
            // Process else
            if !node["else"].is_null() {
                self.eval_node_in_env(&node["else"], env.clone());
            }
            // Process body
            if let Some(stmts) = node["stmts"].as_array() {
                for stmt in stmts {
                    self.eval_node_in_env(stmt, flow_env.clone());
                }
            }
        }
    }

    fn eval_stmt_elseif(&mut self, node: &Value, env: EnvTarget) {
        if let Some(cond_idx) = self.eval_node_in_env(&node["cond"], env.clone()) {
            let cond_name = self.master.relational_database[cond_idx]
                .get_name()
                .unwrap_or("")
                .to_string();
            let (flow_env, _) = self.env_spawn_flow(env, "elseif");
            self.set_flow_condition(
                &flow_env,
                Some(ConditionExpr::Single(cond_idx)),
                cond_name,
            );
            if let Some(stmts) = node["stmts"].as_array() {
                for stmt in stmts {
                    self.eval_node_in_env(stmt, flow_env.clone());
                }
            }
        }
    }

    fn eval_stmt_else(&mut self, node: &Value, env: EnvTarget) {
        let (flow_env, _) = self.env_spawn_flow(env, "else");
        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, flow_env.clone());
            }
        }
    }

    fn eval_stmt_while(&mut self, node: &Value, env: EnvTarget) {
        if let Some(cond_idx) = self.eval_node_in_env(&node["cond"], env.clone()) {
            let cond_name = self.master.relational_database[cond_idx]
                .get_name()
                .unwrap_or("")
                .to_string();
            let (flow_env, _) = self.env_spawn_flow(env, "while");
            self.set_flow_condition(
                &flow_env,
                Some(ConditionExpr::Single(cond_idx)),
                cond_name,
            );
            if let Some(stmts) = node["stmts"].as_array() {
                for stmt in stmts {
                    self.eval_node_in_env(stmt, flow_env.clone());
                }
            }
        }
    }

    fn eval_stmt_for(&mut self, node: &Value, env: EnvTarget) {
        let (flow_env, _) = self.env_spawn_flow(env, "for");

        // Evaluate init
        if let Some(init_arr) = node["init"].as_array() {
            if let Some(init) = init_arr.first() {
                self.eval_node_in_env(init, flow_env.clone());
            }
        }

        // Get for var name and set it
        let for_var_name = node["init"][0]["var"]["name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Set for_var on the flow
        match &flow_env {
            EnvTarget::Flow(path) => {
                if let Some(flow) = self.get_flow_mut(path) {
                    flow.set_for_var(for_var_name.clone());
                }
                // Set the node name to $var_name
                if let Some(node_idx) = self.get_flow(path).and_then(|f| f.get_node_rel_idx(&for_var_name)) {
                    self.master.relational_database[node_idx]
                        .set_name(format!("${}", for_var_name));
                }
            }
            _ => {}
        }

        // Get init node index
        let init_idx = match &flow_env {
            EnvTarget::Flow(path) => {
                self.get_flow(path).and_then(|f| f.get_node_rel_idx(&for_var_name))
            }
            _ => None,
        };

        // Parse cond and loop from file positions
        let cond = if let Some(cond_arr) = node["cond"].as_array() {
            if let Some(c) = cond_arr.first() {
                let start = c["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
                let end = c["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
                self.parse_file_string(start, end)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let loop_str = if let Some(loop_arr) = node["loop"].as_array() {
            if let Some(l) = loop_arr.first() {
                let start = l["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
                let end = l["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
                self.parse_file_string(start, end)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let items = vec![
            if let Some(ii) = init_idx {
                CondItem::NodeRef(ii)
            } else {
                CondItem::Literal(String::new())
            },
            CondItem::Literal(cond),
            CondItem::Literal(loop_str),
        ];

        self.set_flow_condition(
            &flow_env,
            Some(ConditionExpr::Multiple(items)),
            for_var_name,
        );

        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, flow_env.clone());
            }
        }
    }

    fn eval_stmt_foreach(&mut self, node: &Value, env: EnvTarget) {
        if let Some(cond_idx) = self.eval_node_in_env(&node["expr"], env.clone()) {
            let cond_name = self.master.relational_database[cond_idx]
                .get_name()
                .unwrap_or("")
                .to_string();
            let (flow_env, _) = self.env_spawn_flow(env.clone(), "foreach");
            self.set_flow_condition(
                &flow_env,
                Some(ConditionExpr::Single(cond_idx)),
                cond_name,
            );

            let for_var = node["valueVar"]["name"].as_str().unwrap_or("").to_string();
            let by_ref = node["byRef"].as_bool().unwrap_or(false);

            // Add the foreach var to parent env
            let gen_node = SeqNode::Generic {
                value: NodeValue::Str(format!("${}", for_var)),
                node_type: None,
                name: new_name(),
                printed: false,
            };
            let gen_idx = self.master.set_relation(gen_node);
            self.env_set_node(env, Some(&for_var), EnvValue::Node(gen_idx));

            // Add to flow env too
            let gen_node2 = SeqNode::Generic {
                value: NodeValue::Str(format!("${}", for_var)),
                node_type: None,
                name: new_name(),
                printed: false,
            };
            let gen_idx2 = self.master.set_relation(gen_node2);
            self.env_set_node(flow_env.clone(), Some(&for_var), EnvValue::Node(gen_idx2));

            // Set foreach metadata on flow
            match &flow_env {
                EnvTarget::Flow(path) => {
                    if let Some(flow) = self.get_flow_mut(path) {
                        flow.set_foreach_var(for_var);
                        flow.set_by_ref_foreach_var(by_ref);
                    }
                }
                _ => {}
            }

            if let Some(stmts) = node["stmts"].as_array() {
                for stmt in stmts {
                    self.eval_node_in_env(stmt, flow_env.clone());
                }
            }
        }
    }

    fn eval_stmt_try_catch(&mut self, node: &Value, env: EnvTarget) {
        let (try_env, _) = self.env_spawn_flow(env.clone(), "try");
        // Process catches
        if let Some(catches) = node["catches"].as_array() {
            for catch in catches {
                self.eval_node_in_env(catch, env.clone());
            }
        }
        // Process finally
        if !node["finally"].is_null() {
            self.eval_node_in_env(&node["finally"], env);
        }
        // Process try body
        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, try_env.clone());
            }
        }
    }

    fn eval_stmt_catch(&mut self, node: &Value, env: EnvTarget) {
        let types: Vec<String> = node["types"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let var = node["var"]["name"].as_str().unwrap_or("").to_string();

        let (catch_env, _) = self.env_spawn_flow(env, "catch");
        match &catch_env {
            EnvTarget::Flow(path) => {
                if let Some(flow) = self.get_flow_mut(path) {
                    flow.set_catch_types_var(types, var);
                }
            }
            _ => {}
        }

        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, catch_env.clone());
            }
        }
    }

    fn eval_stmt_finally(&mut self, node: &Value, env: EnvTarget) {
        let (finally_env, _) = self.env_spawn_flow(env, "finally");
        if let Some(stmts) = node["stmts"].as_array() {
            for stmt in stmts {
                self.eval_node_in_env(stmt, finally_env.clone());
            }
        }
    }

    fn eval_stmt_switch(&mut self, node: &Value, env: EnvTarget) {
        let mut conditions = Vec::new();
        if let Some(cond_idx) = self.eval_node_in_env(&node["cond"], env.clone()) {
            conditions.push(CondItem::NodeRef(cond_idx));
            let cond_name = self.master.relational_database[cond_idx]
                .get_name()
                .unwrap_or("")
                .to_string();

            let (switch_env, _) = self.env_spawn_flow(env.clone(), "switch");

            if let Some(cases) = node["cases"].as_array() {
                let mut last_case_cond_name = cond_name.clone();
                for case in cases {
                    let case_type = if !case["cond"].is_null() { "case" } else { "case_default" };
                    let (case_env, _) = match &switch_env {
                        EnvTarget::Flow(path) => {
                            let flow = self.get_flow_mut(path).unwrap();
                            let idx = flow.spawn_flow(case_type);
                            let mut new_path = path.clone();
                            new_path.push(idx);
                            (EnvTarget::Flow(new_path), idx)
                        }
                        _ => return,
                    };

                    if !case["cond"].is_null() {
                        if let Some(case_cond_idx) = self.eval_node_in_env(&case["cond"], env.clone()) {
                            conditions.push(CondItem::NodeRef(case_cond_idx));
                            last_case_cond_name = self.master.relational_database[case_cond_idx]
                                .get_name()
                                .unwrap_or("")
                                .to_string();
                        }
                    }

                    self.set_flow_condition(&case_env, None, last_case_cond_name.clone());

                    if let Some(stmts) = case["stmts"].as_array() {
                        for stmt in stmts {
                            self.eval_node_in_env(stmt, case_env.clone());
                        }
                    }
                }
            }

            self.set_flow_condition(
                &switch_env,
                Some(ConditionExpr::Multiple(conditions)),
                cond_name,
            );
        }
    }

    fn eval_stmt_return(&mut self, node: &Value, env: EnvTarget) {
        if let Some(ret_idx) = self.eval_node_in_env(&node["expr"], env.clone()) {
            let call_node = SeqNode::GenericCall {
                call: "return".to_string(),
                argument: Some(NodeId(ret_idx)),
                printed: false,
            };
            let idx = self.master.set_relation(call_node);
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_expr_exit(&mut self, node: &Value, env: EnvTarget) {
        if let Some(ret_idx) = self.eval_node_in_env(&node["expr"], env.clone()) {
            let call_node = SeqNode::GenericCall {
                call: String::new(),
                argument: Some(NodeId(ret_idx)),
                printed: false,
            };
            let idx = self.master.set_relation(call_node);
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_expr_clone(&mut self, node: &Value, env: EnvTarget) {
        if let Some(ret_idx) = self.eval_node_in_env(&node["expr"], env.clone()) {
            let call_node = SeqNode::GenericCall {
                call: "clone".to_string(),
                argument: Some(NodeId(ret_idx)),
                printed: false,
            };
            let idx = self.master.set_relation(call_node);
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_expr_yield(&mut self, node: &Value, env: EnvTarget) {
        if let Some(ret_idx) = self.eval_node_in_env(&node["value"], env.clone()) {
            let call_node = SeqNode::GenericCall {
                call: "yield".to_string(),
                argument: Some(NodeId(ret_idx)),
                printed: false,
            };
            let idx = self.master.set_relation(call_node);
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_expr_yield_from(&mut self, node: &Value, env: EnvTarget) {
        if let Some(ret_idx) = self.eval_node_in_env(&node["value"], env.clone()) {
            let call_node = SeqNode::GenericCall {
                call: "yield from".to_string(),
                argument: Some(NodeId(ret_idx)),
                printed: false,
            };
            let idx = self.master.set_relation(call_node);
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_expr_closure(&mut self, node: &Value) -> Option<usize> {
        let start = node["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
        let end = node["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
        let expression_string = self.parse_file_string(start, end);
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str(expression_string),
            node_type: None,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_expr_ternary(&mut self, node: &Value) -> Option<usize> {
        let start = node["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
        let end = node["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
        let expression_string = self.parse_file_string(start, end);
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str(expression_string),
            node_type: None,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_variadic_placeholder(&mut self) -> Option<usize> {
        let ret_node = SeqNode::Generic {
            value: NodeValue::Str("...".to_string()),
            node_type: None,
            name: new_name(),
            printed: false,
        };
        let idx = self.master.set_relation(ret_node);
        self.master.check_in(idx);
        Some(idx)
    }

    fn eval_stmt_break(&mut self, env: EnvTarget) {
        let node = SeqNode::NonAssignment {
            value: "break;".to_string(),
        };
        self.env_set_node(env, None, EnvValue::Direct(node));
    }

    fn eval_stmt_echo(&mut self, node: &Value, env: EnvTarget) {
        if let Some(exprs) = node["exprs"].as_array() {
            for stmt in exprs {
                if let Some(idx) = self.eval_node_in_env(stmt, env.clone()) {
                    self.env_set_node(env.clone(), None, EnvValue::Node(idx));
                }
            }
        }
    }

    fn eval_expr_print(&mut self, node: &Value, env: EnvTarget) {
        if let Some(idx) = self.eval_node_in_env(&node["expr"], env.clone()) {
            self.env_set_node(env, None, EnvValue::Node(idx));
        }
    }

    fn eval_parse(&mut self, node: &Value, env: EnvTarget) {
        let start = node["attributes"]["startFilePos"].as_u64().unwrap_or(0) as usize;
        let end = node["attributes"]["endFilePos"].as_u64().unwrap_or(0) as usize;
        let expression_string = self.parse_file_string(start, end);
        let non_assign = SeqNode::NonAssignment {
            value: expression_string,
        };
        self.env_set_node(env, None, EnvValue::Direct(non_assign));
    }
}

/// Target environment for eval operations
#[derive(Debug, Clone)]
pub enum EnvTarget {
    Root,
    Function(String),
    Flow(Vec<usize>),       // path of flow indices from root
    FuncFlow(String, usize), // function name + flow index within function
    FuncFlowNested(String, usize, usize), // function name + parent flow + child flow
}

use indexmap::IndexMap;

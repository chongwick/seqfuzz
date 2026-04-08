use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

static VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn new_name() -> String {
    let n = VAR_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    format!("$v_{}", n)
}

pub fn reset_name_counter() {
    VAR_COUNTER.store(0, Ordering::Relaxed);
}

/// Typed index into the relational database (arena).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Value that can be stored in a GenericSeqNode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

impl NodeValue {
    pub fn to_php_string(&self) -> String {
        match self {
            NodeValue::Str(s) => s.clone(),
            NodeValue::Int(i) => i.to_string(),
            NodeValue::Float(f) => f.to_string(),
            NodeValue::Bool(true) => "true".to_string(),
            NodeValue::Bool(false) => "false".to_string(),
            NodeValue::None => "null".to_string(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, NodeValue::None)
    }
}

/// A part of an interpolated string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    NodeRef(usize), // index into relational_database
}

/// The main SeqIR node enum.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SeqNode {
    Func {
        func_name: String,
        arguments: Vec<usize>,
        dependency: Option<String>,
        name: String,
        printed: bool,
    },
    GenericCall {
        call: String,
        argument: Option<NodeId>,
        printed: bool,
    },
    Method {
        obj_name: String,
        method_name: String,
        arguments: Vec<usize>,
        dependency: Option<NodeId>,
        name: String,
        printed: bool,
    },
    PropertyFetch {
        obj_name: String,
        property_name: String,
        dependency: Option<NodeId>,
        name: String,
        printed: bool,
    },
    ObjDec {
        obj_name: Option<String>,
        class_name: String,
        arguments: Vec<usize>,
        dependency: Option<String>,
        name: String,
        printed: bool,
    },
    StaticCall {
        obj_name: Option<String>,
        class_name: String,
        arguments: Vec<usize>,
        dependency: Option<String>,
        name: String,
        printed: bool,
    },
    Generic {
        value: NodeValue,
        node_type: Option<String>,
        name: String,
        printed: bool,
    },
    NonAssignment {
        value: String,
    },
    InterpolatedString {
        parts: Vec<StringPart>,
        name: String,
        printed: bool,
    },
    Array {
        values: indexmap::IndexMap<String, usize>,
        name: String,
        printed: bool,
    },
    ArrayValue {
        value: String,
        arr_name: String,
        dependency: Option<NodeId>,
        name: String,
        printed: bool,
    },
    FlexArrayDimFetch {
        var: NodeId,
        dim: NodeId,
        name: String,
        printed: bool,
    },
    MiddleOp {
        left_key: usize,
        right_key: usize,
        value: String,
        name: String,
        printed: bool,
    },
    FrontOp {
        node_key: usize,
        value: String,
        name: String,
        printed: bool,
    },
    InnerOp {
        node_key: usize,
        value: String,
        name: String,
        printed: bool,
    },
    RearOp {
        node_key: usize,
        value: String,
        name: String,
        printed: bool,
    },
}

#[allow(dead_code)]
impl SeqNode {
    pub fn get_name(&self) -> Option<&str> {
        match self {
            SeqNode::Func { name, .. }
            | SeqNode::Method { name, .. }
            | SeqNode::PropertyFetch { name, .. }
            | SeqNode::ObjDec { name, .. }
            | SeqNode::StaticCall { name, .. }
            | SeqNode::Generic { name, .. }
            | SeqNode::InterpolatedString { name, .. }
            | SeqNode::Array { name, .. }
            | SeqNode::ArrayValue { name, .. }
            | SeqNode::FlexArrayDimFetch { name, .. }
            | SeqNode::MiddleOp { name, .. }
            | SeqNode::FrontOp { name, .. }
            | SeqNode::InnerOp { name, .. }
            | SeqNode::RearOp { name, .. } => Some(name),
            SeqNode::GenericCall { .. } | SeqNode::NonAssignment { .. } => None,
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        match self {
            SeqNode::Func { name, .. }
            | SeqNode::Method { name, .. }
            | SeqNode::PropertyFetch { name, .. }
            | SeqNode::ObjDec { name, .. }
            | SeqNode::StaticCall { name, .. }
            | SeqNode::Generic { name, .. }
            | SeqNode::InterpolatedString { name, .. }
            | SeqNode::Array { name, .. }
            | SeqNode::ArrayValue { name, .. }
            | SeqNode::FlexArrayDimFetch { name, .. }
            | SeqNode::MiddleOp { name, .. }
            | SeqNode::FrontOp { name, .. }
            | SeqNode::InnerOp { name, .. }
            | SeqNode::RearOp { name, .. } => *name = new_name,
            SeqNode::GenericCall { .. } | SeqNode::NonAssignment { .. } => {}
        }
    }

    pub fn get_node_value(&self) -> Option<&NodeValue> {
        match self {
            SeqNode::Generic { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn get_op_value(&self) -> Option<&str> {
        match self {
            SeqNode::MiddleOp { value, .. }
            | SeqNode::FrontOp { value, .. }
            | SeqNode::InnerOp { value, .. }
            | SeqNode::RearOp { value, .. } => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn set_value_str(&mut self, new_value: String) {
        match self {
            SeqNode::Generic { value, .. } => *value = NodeValue::Str(new_value),
            SeqNode::MiddleOp { value, .. }
            | SeqNode::FrontOp { value, .. }
            | SeqNode::InnerOp { value, .. }
            | SeqNode::RearOp { value, .. } => *value = new_value,
            _ => {}
        }
    }

    pub fn get_node_type(&self) -> Option<&str> {
        match self {
            SeqNode::Generic { node_type, .. } => node_type.as_deref(),
            _ => None,
        }
    }

    pub fn set_node_type(&mut self, new_type: String) {
        if let SeqNode::Generic { node_type, .. } = self {
            *node_type = Some(new_type);
        }
    }

    pub fn is_op_node(&self) -> bool {
        matches!(
            self,
            SeqNode::MiddleOp { .. }
                | SeqNode::FrontOp { .. }
                | SeqNode::InnerOp { .. }
                | SeqNode::RearOp { .. }
        )
    }

    pub fn is_generic(&self) -> bool {
        matches!(self, SeqNode::Generic { .. })
    }

    pub fn is_func(&self) -> bool {
        matches!(self, SeqNode::Func { .. })
    }

    pub fn is_array(&self) -> bool {
        matches!(self, SeqNode::Array { .. })
    }

    pub fn get_func_name_str(&self) -> Option<&str> {
        match self {
            SeqNode::Func { func_name, .. } => Some(func_name),
            _ => None,
        }
    }

    pub fn get_func_dependency(&self) -> Option<&str> {
        match self {
            SeqNode::Func { dependency, .. } => dependency.as_deref(),
            _ => None,
        }
    }

    pub fn add_array_value(&mut self, key: Option<String>, value: usize) {
        if let SeqNode::Array { values, .. } = self {
            match key {
                Some(k) => {
                    values.insert(k, value);
                }
                None => {
                    let idx = values.len();
                    values.insert(idx.to_string(), value);
                }
            }
        }
    }

    pub fn get_array_value_by_key(&self, key: &str) -> Option<(String, String, NodeId)> {
        if let SeqNode::Array { values, name, .. } = self {
            if values.contains_key(key) {
                // Return info needed to create an ArrayValue node
                return Some((key.to_string(), name.clone(), NodeId(0))); // placeholder
            }
        }
        None
    }
}

/// Get expression for a node at `idx` in `rel_db`, appending statements to `pp_env`.
/// Returns the variable name of this node.
/// This is a free function to avoid borrow conflicts.
pub fn get_expression(rel_db: &mut Vec<SeqNode>, idx: usize, pp_env: &mut Vec<String>) -> Option<String> {
    // Check if already printed and get info we need before mutating
    let node = &rel_db[idx];
    match node {
        SeqNode::Func { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::GenericCall { printed: true, .. } => return None,
        SeqNode::Method { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::PropertyFetch { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::ObjDec { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::StaticCall { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::Generic { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::InterpolatedString { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::Array { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::ArrayValue { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::FlexArrayDimFetch { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::MiddleOp { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::FrontOp { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::InnerOp { printed: true, name, .. } => return Some(name.clone()),
        SeqNode::RearOp { printed: true, name, .. } => return Some(name.clone()),
        _ => {}
    }

    // Clone the node to work with it without holding a borrow on rel_db
    let node = rel_db[idx].clone();

    match node {
        SeqNode::Func {
            func_name,
            arguments,
            name,
            ..
        } => {
            // Mark as printed
            if let SeqNode::Func { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = format!("{} = {}(", name, func_name);
            for a in &arguments {
                let arg_name = get_expression(rel_db, *a, pp_env).unwrap_or_default();
                ret_expr.push_str(&arg_name);
                ret_expr.push(',');
            }
            ret_expr.push_str(");");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::GenericCall {
            call, argument, ..
        } => {
            if let SeqNode::GenericCall { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            if let Some(arg_id) = argument {
                let arg_name = get_expression(rel_db, arg_id.0, pp_env).unwrap_or_default();
                pp_env.push(format!("{}({});", call, arg_name));
            } else {
                pp_env.push(format!("{};", call));
            }
            None
        }

        SeqNode::Method {
            obj_name,
            method_name,
            arguments,
            dependency,
            name,
            ..
        } => {
            if let SeqNode::Method { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = String::new();
            if let Some(dep_id) = dependency {
                let mut tmp_v = Vec::new();
                let dep_name = get_expression(rel_db, dep_id.0, &mut tmp_v).unwrap_or_default();
                ret_expr = tmp_v.join("\n");
                ret_expr.push_str(&format!("{} = {}->{}(", name, dep_name, method_name));
            } else {
                ret_expr.push_str(&format!("{} = ${}->{}(", name, obj_name, method_name));
            }
            for a in &arguments {
                let arg_name = get_expression(rel_db, *a, pp_env).unwrap_or_default();
                ret_expr.push_str(&arg_name);
                ret_expr.push(',');
            }
            ret_expr.push_str(");");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::PropertyFetch {
            obj_name,
            property_name,
            dependency,
            name,
            ..
        } => {
            if let SeqNode::PropertyFetch { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = String::new();
            if let Some(dep_id) = dependency {
                let mut tmp_v = Vec::new();
                let dep_name = get_expression(rel_db, dep_id.0, &mut tmp_v).unwrap_or_default();
                ret_expr = tmp_v.join("\n");
                ret_expr.push_str(&format!("\n{} = {}->{};\n", name, dep_name, property_name));
            } else {
                ret_expr.push_str(&format!("\n{} = ${}->{};\n", name, obj_name, property_name));
            }
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::ObjDec {
            class_name,
            arguments,
            dependency,
            name,
            ..
        } => {
            if let SeqNode::ObjDec { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = String::new();
            if let Some(dep) = &dependency {
                ret_expr.push_str(dep);
                ret_expr.push('\n');
            }
            ret_expr.push_str(&format!("{} = new {}(", name, class_name));
            for a in &arguments {
                let arg_name = get_expression(rel_db, *a, pp_env).unwrap_or_default();
                ret_expr.push_str(&arg_name);
                ret_expr.push(',');
            }
            ret_expr.push_str(");");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::StaticCall {
            class_name,
            arguments,
            dependency,
            name,
            ..
        } => {
            if let SeqNode::StaticCall { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = String::new();
            if let Some(dep) = &dependency {
                ret_expr.push_str(dep);
                ret_expr.push('\n');
            }
            ret_expr.push_str(&format!("{} = {}(", name, class_name));
            for a in &arguments {
                let arg_name = get_expression(rel_db, *a, pp_env).unwrap_or_default();
                ret_expr.push_str(&arg_name);
                ret_expr.push(',');
            }
            ret_expr.push_str(");");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::Generic {
            mut value,
            name,
            ..
        } => {
            if value.is_none() {
                value = NodeValue::Str(name.clone());
                if let SeqNode::Generic { value: v, .. } = &mut rel_db[idx] {
                    *v = NodeValue::Str(name.clone());
                }
            }
            if let SeqNode::Generic { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let ret_expr = format!("{} = {};", name, value.to_php_string());
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::NonAssignment { value } => {
            pp_env.push(value);
            None
        }

        SeqNode::InterpolatedString {
            parts, name, ..
        } => {
            if let SeqNode::InterpolatedString { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = format!("{} = \"", name);
            for p in &parts {
                match p {
                    StringPart::NodeRef(ref_idx) => {
                        let p_name = get_expression(rel_db, *ref_idx, pp_env)
                            .unwrap_or_default()
                            .replace('\'', "");
                        ret_expr.push_str(&p_name);
                    }
                    StringPart::Literal(s) => {
                        ret_expr.push_str(&s.replace('\'', ""));
                    }
                }
            }
            ret_expr.push_str("\";");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::Array {
            values, name, ..
        } => {
            if let SeqNode::Array { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let mut ret_expr = format!("{} = array(", name);
            for (k, v) in &values {
                let val_name = get_expression(rel_db, *v, pp_env).unwrap_or_default();
                ret_expr.push_str(&format!("{} => {},", k, val_name));
            }
            ret_expr.push_str(");");
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::ArrayValue {
            value,
            arr_name,
            dependency,
            name,
            ..
        } => {
            if let Some(dep_id) = dependency {
                get_expression(rel_db, dep_id.0, pp_env);
            }
            if let SeqNode::ArrayValue { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let ret_expr = format!("{} = {}[{}];", name, arr_name, value);
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::FlexArrayDimFetch {
            var, dim, name, ..
        } => {
            if let SeqNode::FlexArrayDimFetch { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let var_name = get_expression(rel_db, var.0, pp_env).unwrap_or_default();
            let dim_name = get_expression(rel_db, dim.0, pp_env).unwrap_or_default();
            let ret_expr = format!("{} = {}[{}];", name, var_name, dim_name);
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::MiddleOp {
            left_key,
            right_key,
            value,
            name,
            ..
        } => {
            if let SeqNode::MiddleOp { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let left_name = get_expression(rel_db, left_key, pp_env).unwrap_or_default();
            let right_name = get_expression(rel_db, right_key, pp_env).unwrap_or_default();
            let ret_expr = format!("{} = {} {} {};", name, left_name, value, right_name);
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::FrontOp {
            node_key,
            value,
            name,
            ..
        } => {
            if let SeqNode::FrontOp { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let node_name = get_expression(rel_db, node_key, pp_env).unwrap_or_default();
            let ret_expr = format!("{} = {}({});", name, value, node_name);
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::InnerOp {
            node_key,
            value,
            name,
            ..
        } => {
            if let SeqNode::InnerOp { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let node_name = get_expression(rel_db, node_key, pp_env).unwrap_or_default();
            let ret_expr = format!("{} {} ({});", name, value, node_name);
            pp_env.push(ret_expr);
            Some(name)
        }

        SeqNode::RearOp {
            node_key,
            value,
            name,
            ..
        } => {
            if let SeqNode::RearOp { printed, .. } = &mut rel_db[idx] {
                *printed = true;
            }
            let node_name = get_expression(rel_db, node_key, pp_env).unwrap_or_default();
            let mut ret_expr = format!("{} = {};", name, node_name);
            ret_expr.push_str(&format!("\n{}{};", name, value));
            pp_env.push(ret_expr);
            Some(name)
        }
    }
}

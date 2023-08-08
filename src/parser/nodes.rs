use std::{collections::HashMap, fmt::Debug};

use trc::Trc;

use crate::utils::Position;

#[derive(Debug)]
pub struct Node {
    pub pos: Position,
    pub tp: NodeType,
    pub data: Trc<Box<dyn NodeData>>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            pos: self.pos.clone(),
            tp: self.tp.clone(),
            data: self.data.clone(),
        }
    }
}

impl Node {
    pub fn new(pos: Position, tp: NodeType, data: Box<dyn NodeData>) -> Node {
        Node {
            pos,
            tp,
            data: data.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeType {
    I32,
    Binary,
    Identifier,
    Let,
    Store,
    Reference,
    I8,
    I16,
    I64,
    I128,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    Fn,
    Return,
}

#[derive(Debug)]
pub struct NodeValue<'a> {
    pub raw: HashMap<String, String>,
    pub nodes: HashMap<String, &'a Node>,
    pub op: Option<OpType>,
    pub nodearr: Option<&'a Vec<Node>>,
    pub args: Option<Vec<String>>,
    pub mapping: Option<&'a Vec<(Node, Node)>>,
    pub booleans: HashMap<String, bool>,
}

pub trait NodeData {
    fn get_data(&self) -> NodeValue;
}

impl Debug for dyn NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeData{:?}", self.get_data())
    }
}

impl<'a> NodeValue<'a> {
    fn new() -> NodeValue<'a> {
        NodeValue {
            raw: HashMap::new(),
            nodes: HashMap::new(),
            op: None,
            nodearr: None,
            args: None,
            mapping: None,
            booleans: HashMap::new(),
        }
    }
}

//===================================================
//===================================================

pub struct DecimalNode {
    pub value: String,
}

impl NodeData for DecimalNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value
            .raw
            .insert(String::from("value"), self.value.to_owned());

        value
    }
}

// ========================

#[derive(Debug, Copy, Clone)]
pub enum OpType {
    Add,
}

pub struct BinaryNode {
    pub left: Node,
    pub right: Node,
    pub op: OpType,
}

impl NodeData for BinaryNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("left"), &self.left);
        value.nodes.insert(String::from("right"), &self.right);
        value.op = Some(self.op);

        value
    }
}

// ========================

pub struct IdentifierNode {
    pub value: String,
}

impl NodeData for IdentifierNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value
            .raw
            .insert(String::from("value"), self.value.to_owned());

        value
    }
}

// ========================

pub struct LetNode {
    pub name: String,
    pub expr: Node,
    pub is_mut: bool,
}

impl NodeData for LetNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.raw.insert(String::from("name"), self.name.to_owned());
        value.nodes.insert(String::from("expr"), &self.expr);
        value.booleans.insert(String::from("is_mut"), self.is_mut);

        value
    }
}

// ========================

pub struct StoreNode {
    pub name: String,
    pub expr: Node,
}

impl NodeData for StoreNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.raw.insert(String::from("name"), self.name.to_owned());
        value.nodes.insert(String::from("expr"), &self.expr);

        value
    }
}

// ========================

pub struct ReferenceNode {
    pub expr: Node,
}

impl NodeData for ReferenceNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("expr"), &self.expr);

        value
    }
}

// ========================

pub struct BoolNode {
    pub value: bool,
}

impl NodeData for BoolNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.booleans.insert(String::from("value"), self.value);

        value
    }
}

// ========================

pub struct FnNode {
    pub name: String,
    pub args: Vec<String>,
    pub code: Vec<Node>,
}

impl NodeData for FnNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodearr = Some(&self.code);
        value.raw.insert(String::from("name"), self.name.clone());
        value.args = Some(self.args.clone());

        value
    }
}

// ========================

pub struct ReturnNode {
    pub expr: Node,
}

impl NodeData for ReturnNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("expr"), &self.expr);

        value
    }
}

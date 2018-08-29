#![allow(non_snake_case, dead_code)]

use std::any::Any;
use std::marker::Send;
use std::sync::mpsc::{Receiver, Sender};

pub struct OhuaOperator {
    pub input: Vec<Receiver<Box<dyn Any + 'static + Send>>>,
    pub output: Vec<(u32, Vec<Sender<Box<dyn Any + 'static + Send>>>)>,
    pub name: String,
    pub func: Box<fn(Vec<Box<dyn Any + 'static + Send>>) -> Vec<Vec<Box<dyn Any + 'static + Send>>>>,
    pub op_type: OpType,
}

pub struct OhuaData {
    pub graph: DFGraph,
    pub mainArity: i32,
    pub sfDependencies: Vec<SfDependency>,
}

pub struct DFGraph {
    pub operators: Vec<Operator>,
    pub arcs: Vec<Arc>,
    pub return_arc: ArcIdentifier,
    pub input_targets: Vec<ArcIdentifier>,
}

pub struct Operator {
    pub operatorId: i32,
    pub operatorType: OperatorType,
}

pub struct OperatorType {
    pub qbNamespace: Vec<String>,
    pub qbName: String,
    pub func: Box<fn(Vec<Box<dyn Any + 'static + Send>>) -> Vec<Vec<Box<dyn Any + 'static + Send>>>>,
    pub op_type: OpType,
}

#[derive(Clone)]
pub enum OpType {
    SfnWrapper,
    OhuaOperator(Box<fn(OhuaOperator)>)
}

pub struct Arc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

pub struct ArcIdentifier {
    pub operator: i32,
    pub index: i32,
}

pub enum ValueType {
    EnvironmentVal(i32),
    LocalVal(ArcIdentifier),
}

pub struct ArcSource {
    pub s_type: String,
    pub val: ValueType,
}

pub struct SfDependency {
    pub qbNamespace: Vec<String>,
    pub qbName: String,
}

impl OperatorType {
    pub fn qualified_name(&self) -> String {
        let mut name = String::new();
        for ns_part in &self.qbNamespace {
            name += ns_part.as_str();
            name += "::";
        }
        name + self.qbName.as_str()
    }
}

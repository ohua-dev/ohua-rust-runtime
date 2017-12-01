#![allow(non_snake_case)]

use runtime::GenericType;

pub struct OhuaData {
    pub graph: DFGraph,
    pub mainArity: i32,
    pub sfDependencies: Vec<SfDependency>,
}

pub struct DFGraph {
    pub operators: Vec<Operator>,
    pub arcs: Vec<Arc>,
}

pub struct Operator {
    pub operatorId: i32,
    pub operatorType: OperatorType,
}

pub struct OperatorType {
    pub qbNamespace: Vec<String>,
    pub qbName: String,
    pub func: Box<fn(Vec<Box<GenericType>>) -> Vec<Box<GenericType>>>,
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

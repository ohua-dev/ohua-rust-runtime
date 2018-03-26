//! Data types to represent the dataflow graph and the `type-dump` file.
#![allow(non_snake_case)]

use std::fmt;
// TODO: Maybe refactor the data structures (using serde-rename) to make the field names rust-compliant

/// Data structure used to deserialize the argument types from the `type-dump` file.
#[derive(Debug, Deserialize)]
pub struct AlgorithmArguments {
    #[serde(rename(deserialize = "return"))]
    pub return_type: String,
    #[serde(rename(deserialize = "arguments"))]
    pub argument_types: Vec<String>,
}

// all following data structures are part of the DFG specification

/// The all-encapsulating OhuaData structure.
#[derive(Deserialize, Debug)]
pub struct OhuaData {
    pub graph: DFGraph,
    pub mainArity: i32,
    pub sfDependencies: Vec<SfDependency>,
}

/// Representation of an Ohua dataflow graph.
#[derive(Deserialize, Debug)]
pub struct DFGraph {
    pub operators: Vec<Operator>,
    pub arcs: Vec<Arc>,
    pub return_arc: ArcIdentifier,
    #[serde(default)]
    pub input_targets: Vec<ArcIdentifier>,
}

/// A single operator of the DFG. Represents a stateful function that is to be called.
#[derive(Deserialize, Debug)]
pub struct Operator {
    #[serde(rename(deserialize = "id"))]
    pub operatorId: i32,
    #[serde(rename(deserialize = "type"))]
    pub operatorType: OperatorType,
}

/// The inner operator information such as namespace, function name and link to the respective function.
#[derive(Deserialize, Debug)]
pub struct OperatorType {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
    #[serde(default)]
    pub func: String,
}

/// A simple Arc between two points in the graph.
#[derive(Deserialize, Debug)]
pub struct Arc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

/// A local Arc endpoint, an operator.
#[derive(Deserialize, Debug)]
pub struct ArcIdentifier {
    pub operator: i32,
    pub index: i32,
}

/// Describes the type of an Arc source. This can either be an environment value _or_ a local value (another operator).
#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum ValueType {
    EnvironmentVal(i32),
    LocalVal(ArcIdentifier),
}

/// Source for an Arc.
#[derive(Deserialize, Debug)]
pub struct ArcSource {
    #[serde(rename(deserialize = "type"))]
    pub s_type: String,
    pub val: ValueType,
}

/// Represents a dependency to a stateful function.
#[derive(Deserialize, Debug)]
pub struct SfDependency {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
}

impl fmt::Display for OhuaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sf_deps = String::new();
        for dep in &self.sfDependencies {
            sf_deps += format!("{}, ", dep).as_str();
        }

        write!(f, "OhuaData {{graph: {}, mainArity: {}, sfDependencies: vec![{}]}}", self.graph, self.mainArity, sf_deps)
    }
}

impl fmt::Display for DFGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ops = String::new();
        for op in &self.operators {
            ops += format!("{}, ", op).as_str();
        }

        let mut arcs = String::new();
        for arc in &self.arcs {
            arcs += format!("{}, ", arc).as_str();
        }

        let mut inputs = String::new();
        for inp in &self.input_targets {
            inputs += format!("{}, ", inp).as_str();
        }

        write!(f, "DFGraph {{operators: vec![{}], arcs: vec![{}], return_arc: {}, input_targets: vec![{}]}}", ops, arcs, &self.return_arc, inputs)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operator {{operatorId: {}, operatorType: {}}}", self.operatorId, self.operatorType)
    }
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut namesp = String::new();
        for space in &self.qbNamespace {
            namesp += format!("String::from(\"{}\"), ", space).as_str();
        }

        write!(f, "OperatorType {{qbNamespace: vec![{}], qbName: String::from(\"{}\"), func: Box::new({})}}", namesp, self.qbName, self.func)
    }
}

impl fmt::Display for Arc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Arc {{target: {}, source: {}}}", self.target, self.source)
    }
}

impl fmt::Display for ArcIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArcIdentifier {{operator: {}, index: {}}}", self.operator, self.index)
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ValueType::EnvironmentVal(x) => write!(f, "ValueType::EnvironmentVal({})", x),
            &ValueType::LocalVal(ref arc_id)  => write!(f, "ValueType::LocalVal({})", arc_id),
        }
    }
}

impl fmt::Display for ArcSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArcSource {{s_type: String::from(\"{}\"), val: {}}}", self.s_type, self.val)
    }
}

impl fmt::Display for SfDependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut namesp = String::new();
        for space in &self.qbNamespace {
            namesp += format!("String::from(\"{}\"), ", space).as_str();
        }

        write!(f, "SfDependency {{qbNamespace: vec![{}], qbName: String::from(\"{}\")}}", namesp, self.qbName)
    }
}

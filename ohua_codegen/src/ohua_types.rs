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
    pub arcs: Arcs,
    pub return_arc: ArcIdentifier,
    #[serde(default)]
    pub input_targets: Vec<ArcIdentifier>,
}

#[derive(Deserialize, Debug)]
pub struct Arcs {
    pub direct: Vec<DirectArc>,
    pub compound: Vec<CompoundArc>,// FIXME to be removed!
    pub state: Vec<StateArc>,
}

/// A single operator of the DFG. Represents a stateful function that is to be called.
#[derive(Deserialize, Debug)]
pub struct Operator {
    #[serde(rename(deserialize = "id"))]
    pub operatorId: i32,
    #[serde(rename(deserialize = "type"))]
    pub operatorType: OperatorType,
    #[serde(rename(deserialize = "n_type"))]
    pub nodeType: NodeType,
}

/// The inner operator information such as namespace, function name and link to the respective function.
#[derive(Deserialize, Debug)]
pub struct OperatorType {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
    // #[serde(default)]
    // pub func: String
}

/// Type of the operator. It can either be a normal wrapper around a SFN or a full-fledged Ohua operator.
#[derive(Deserialize, Debug)]
pub enum NodeType {
    /// Simple wrapper around a stateful function.
    FunctionNode,
    /// Dedicated Ohua operator, the enclosed string is the function name which is expected to be of type `Box<fn(OhuaOperator)>`.
    OperatorNode,
}

/// A simple Arc between two points in the graph.
#[derive(Deserialize, Debug)]
pub struct DirectArc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

/// A local Arc endpoint, an operator.
#[derive(Deserialize, Debug)]
pub struct ArcIdentifier {
    pub operator: i32,
    pub index: i32,
}

#[derive(Deserialize, Debug)]
pub struct CompoundArc {
    pub target: ArcIdentifier,
    pub source: Vec<ArcSource>,
}

#[derive(Deserialize, Debug)]
pub struct StateArc {
    pub target: i32,
    pub source: ArcSource,
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
#[derive(Deserialize, Debug, Clone)]
pub struct SfDependency {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
}

// impl fmt::Display for OhuaData {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut sf_deps = String::new();
//         for dep in &self.sfDependencies {
//             sf_deps += format!("{}, ", dep).as_str();
//         }
//
//         write!(
//             f,
//             "OhuaData {{graph: {}, mainArity: {}, sfDependencies: vec![{}]}}",
//             self.graph, self.mainArity, sf_deps
//         )
//     }
// }
//
// impl fmt::Display for DFGraph {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut ops = String::new();
//         for op in &self.operators {
//             ops += format!("{}, ", op).as_str();
//         }
//
//         let mut arcs = String::new();
//         for arc in &self.arcs {
//             arcs += format!("{}, ", arc).as_str();
//         }
//
//         let mut inputs = String::new();
//         for inp in &self.input_targets {
//             inputs += format!("{}, ", inp).as_str();
//         }
//
//         write!(f, "DFGraph {{operators: vec![{}], arcs: vec![{}], return_arc: {}, input_targets: vec![{}]}}", ops, arcs, &self.return_arc, inputs)
//     }
// }
//
// impl fmt::Display for Operator {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Operator {{operatorId: {}, operatorType: {}}}",
//             self.operatorId, self.operatorType
//         )
//     }
// }
//
// impl OperatorType {
//     pub fn function_name(&self) -> String {
//         let mut name = String::new();
//         for item in &self.qbNamespace {
//             name += item.as_str();
//             name += "::";
//         }
//         name += self.qbName.as_str();
//
//         name
//     }
// }
//
// impl fmt::Display for OperatorType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut namesp = String::new();
//         for space in &self.qbNamespace {
//             namesp += format!("String::from(\"{}\"), ", space).as_str();
//         }
//
//         write!(f, "OperatorType {{qbNamespace: vec![{namesp}], qbName: String::from(\"{name}\"), func: Box::new({fn}), op_type: {ty}}}", namesp = namesp, name = self.qbName, fn = self.func, ty = self.op_type)
//     }
// }
//
// impl fmt::Display for OpType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             OpType::SfnWrapper => write!(f, "OpType::SfnWrapper"),
//             OpType::OhuaOperator(ref op) => write!(f, "OpType::OhuaOperator(Box::new({}))", op),
//         }
//     }
// }
//
// impl fmt::Display for Arc {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Arc {{target: {}, source: {}}}",
//             self.target, self.source
//         )
//     }
// }
//
// impl fmt::Display for ArcIdentifier {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "ArcIdentifier {{operator: {}, index: {}}}",
//             self.operator, self.index
//         )
//     }
// }
//
// impl fmt::Display for ValueType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             &ValueType::EnvironmentVal(x) => write!(f, "ValueType::EnvironmentVal({})", x),
//             &ValueType::LocalVal(ref arc_id) => write!(f, "ValueType::LocalVal({})", arc_id),
//         }
//     }
// }
//
// impl fmt::Display for ArcSource {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "ArcSource {{s_type: String::from(\"{}\"), val: {}}}",
//             self.s_type, self.val
//         )
//     }
// }
//
// impl fmt::Display for SfDependency {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut namesp = String::new();
//         for space in &self.qbNamespace {
//             namesp += format!("String::from(\"{}\"), ", space).as_str();
//         }
//
//         write!(
//             f,
//             "SfDependency {{qbNamespace: vec![{}], qbName: String::from(\"{}\")}}",
//             namesp, self.qbName
//         )
//     }
// }

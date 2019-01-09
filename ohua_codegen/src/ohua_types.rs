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
#[derive(Deserialize, Debug, Clone)]
pub struct OperatorType {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
    // #[serde(default)]
    // pub func: String
}

/// Type of the operator. It can either be a normal wrapper around a SFN or a full-fledged Ohua operator.
#[derive(Deserialize, Debug, PartialEq)]
pub enum NodeType {
    /// Simple wrapper around a stateful function.
    FunctionNode,
    /// Dedicated Ohua operator, the enclosed string is the function name which is expected to be of type `Box<fn(OhuaOperator)>`.
    OperatorNode,
}

/// A simple Arc between two points in the graph.
#[derive(Deserialize, Debug, Clone)]
pub struct DirectArc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

/// A local Arc endpoint, an operator.
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "val")]
pub enum ArcSource {
    #[serde(rename(deserialize = "env"))]
    Env(Envs),
    #[serde(rename(deserialize = "local"))]
    Local(ArcIdentifier),
}

// The below does not work because UnitLit has no 'content' field.
// #[derive(Deserialize, Debug, Clone)]
// #[serde(tag = "tag", content = "contents")]
// pub enum Envs {
//     NumericLit(i32),
//     EnvRefLit(i32),
//     FunRefLit(String),
//     UnitLit()
// }

//https://serde.rs/enum-representations.html

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "tag")]
pub enum Envs {
    NumericLit { content: i32 },
    EnvRefLit { content: i32 },
    FunRefLit { contents: (OperatorType,i32) },
    // FIXME the above is a hack for now. it should be this:
    // FunRefLit { contents: OperatorType },
    UnitLit {}
}


/// Represents a dependency to a stateful function.
#[derive(Deserialize, Debug, Clone)]
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

        write!(
            f,
            "OhuaData {{graph: {}, mainArity: {}, sfDependencies: vec![{}]}}",
            self.graph, self.mainArity, sf_deps
        )
    }
}

impl fmt::Display for DFGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ops = String::new();
        for op in &self.operators {
            ops += format!("{}, ", op).as_str();
        }

        let mut arcs = String::new();
        for arc in &self.arcs.direct {
            arcs += format!("{}, ", arc).as_str();
        }

        let mut states = String::new();
        for state in &self.arcs.state {
            states += format!("{}, ", state).as_str();
        }

        let mut inputs = String::new();
        for inp in &self.input_targets {
            inputs += format!("{}, ", inp).as_str();
        }

        write!(f, "DFGraph {{operators: vec![{}], arcs: vec![{}], states: vec![{}], return_arc: {}, input_targets: vec![{}]}}", ops, arcs, states, &self.return_arc, inputs)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Operator {{operatorId: {}, operatorType: {}, nodeType: {}}}",
            self.operatorId, self.operatorType, self.nodeType
        )
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeType::FunctionNode => write!(f, "FunctionNode"),
            NodeType::OperatorNode => write!(f, "OperatorNode"),
        }
    }
}

impl OperatorType {
    pub fn function_name(&self) -> String {
        let mut name = String::new();
        for item in &self.qbNamespace {
            name += item.as_str();
            name += "::";
        }
        name += self.qbName.as_str();

        name
    }
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut namesp = String::new();
        for space in &self.qbNamespace {
            namesp += format!("String::from(\"{}\"), ", space).as_str();
        }

        write!(f, "OperatorType {{qbNamespace: vec![{namesp}], qbName: String::from(\"{name}\")}}", namesp = namesp, name = self.qbName)
    }
}

// impl fmt::Display for OpType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             OpType::SfnWrapper => write!(f, "OpType::SfnWrapper"),
//             OpType::OhuaOperator(ref op) => write!(f, "OpType::OhuaOperator(Box::new({}))", op),
//         }
//     }
// }

impl fmt::Display for DirectArc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DirectArc {{target: {}, source: {}}}",
            self.target, self.source
        )
    }
}

impl fmt::Display for StateArc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StateArc {{target: {}, source: {}}}",
            self.target, self.source
        )
    }
}

impl fmt::Display for ArcIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ArcIdentifier {{operator: {}, index: {}}}",
            self.operator, self.index
        )
    }
}

impl fmt::Display for ArcSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ArcSource::Env(ref x) => write!(f, "ValueType::EnvironmentVal({})", x),
            &ArcSource::Local(ref arc_id) => write!(f, "ValueType::LocalVal({})", arc_id),
        }
    }
}

impl fmt::Display for Envs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Envs::NumericLit{ content:i } => write!(f, "NumericLit({})", i),
            Envs::EnvRefLit{ content:i } => write!(f, "EnvRefLit({})", i),
            Envs::FunRefLit{ contents:s } => write!(f, "FunRefLit({})", s.0),
            Envs::UnitLit{} => write!(f, "UnitLit()"),
        }
    }
}


impl fmt::Display for SfDependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut namesp = String::new();
        for space in &self.qbNamespace {
            namesp += format!("String::from(\"{}\"), ", space).as_str();
        }

        write!(
            f,
            "SfDependency {{qbNamespace: vec![{}], qbName: String::from(\"{}\")}}",
            namesp, self.qbName
        )
    }
}

#![allow(non_snake_case)]

use std::fmt;
// TODO: Maybe refactor the data structures (using serde-rename) to make the field names rust-compliant

#[derive(Serialize, Deserialize, Debug)]
pub struct OhuaData {
    pub graph: DFGraph,
    pub mainArity: i32,
    pub sfDependencies: Vec<SfDependency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DFGraph {
    pub operators: Vec<Operator>,
    pub arcs: Vec<Arc>,
    pub return_arc: ArcIdentifier,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Operator {
    #[serde(rename(deserialize = "id"))]
    pub operatorId: i32,
    #[serde(rename(deserialize = "type"))]
    pub operatorType: OperatorType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OperatorType {
    #[serde(rename(deserialize = "namespace"))]
    pub qbNamespace: Vec<String>,
    #[serde(rename(deserialize = "name"))]
    pub qbName: String,
    #[serde(default = "empty_fn")]
    pub func: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Arc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArcIdentifier {
    pub operator: i32,
    pub index: i32,
}

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ValueType {
    EnvironmentVal(i32),
    LocalVal(ArcIdentifier),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArcSource {
    #[serde(rename(deserialize = "type"))]
    pub s_type: String,
    pub val: ValueType,
}

#[derive(Serialize, Deserialize, Debug)]
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

        write!(f, "DFGraph {{operators: vec![{}], arcs: vec![{}], return_arc: {}}}", ops, arcs, &self.return_arc)
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

fn empty_fn() -> String {
    "".to_string()
}

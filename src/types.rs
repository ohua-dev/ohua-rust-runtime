#![allow(non_snake_case)]
// TODO: Maybe refactor the data structures (using serde-rename) to make the field names rust-compliant

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OhuaData {
    pub graph: DFGraph,
    pub mainArity: i32,
    pub sfDependencies: Vec<SfDependency>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DFGraph {
    pub operators: Vec<Operator>,
    pub arcs: Vec<Arc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operator {
    pub operatorId: i32,
    pub operatorType: OperatorType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperatorType {
    pub qbNamespace: Vec<String>,
    pub qbName: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Arc {
    pub target: ArcIdentifier,
    pub source: ArcSource,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArcIdentifier {
    pub operator: i32,
    pub index: i32,
}

#[serde(untagged)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ValueType {
    EnvironmentVal(i32),
    LocalVal(ArcIdentifier),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArcSource {
    #[serde(rename(deserialize = "type"))]
    pub s_type: String,
    pub val: ValueType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SfDependency {
    pub qbNamespace: Vec<String>,
    pub qbName: String,
}

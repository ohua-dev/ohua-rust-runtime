#![allow(non_snake_case)]
// TODO: Maybe refactor the data structures (using serde-rename) to make the field names rust-compliant

#[derive(Serialize, Deserialize, Debug)]
pub struct OhuaData {
    graph: DFGraph,
    mainArity: i32,
    sfDependencies: Vec<SfDependency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DFGraph {
    operators: Vec<Operator>,
    arcs: Vec<Arc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Operator {
    operatorId: i32,
    operatorType: OperatorType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OperatorType {
    qbNamespace: Vec<String>,
    qbName: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Arc {
    target: ArcIdentifier,
    source: ArcSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArcIdentifier {
    operator: i32,
    index: i32,
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
    s_type: String,
    val: ValueType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SfDependency {
    qbNamespace: Vec<String>,
    qbName: String,
}

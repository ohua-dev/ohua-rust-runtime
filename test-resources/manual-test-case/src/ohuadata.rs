use types::*;
use wrappers::*;

pub fn generate() -> OhuaData {
    // CAVEATS: make sure the operator vector is sorted by operatorId!
    OhuaData {
        graph: DFGraph {
            operators: vec![Operator {
                                operatorId: 1,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("calc"),
                                    func: Box::new(calc_wrapped),
                                },
                            },
                            Operator {
                                operatorId: 2,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("world"),
                                    func: Box::new(world_wrapped),
                                },
                            }],
            arcs: vec![Arc {
                           target: ArcIdentifier {
                               operator: 1,
                               index: 0,
                           },
                           source: ArcSource {
                               s_type: String::from("env"),
                               val: ValueType::EnvironmentVal(0),
                           },
                       },
                       Arc {
                           target: ArcIdentifier {
                               operator: 2,
                               index: 0,
                           },
                           source: ArcSource {
                               s_type: String::from("local"),
                               val: ValueType::LocalVal(ArcIdentifier {
                                                            operator: 1,
                                                            index: -1,
                                                        }),
                           },
                       }],
        },
        mainArity: 0,
        sfDependencies: vec![SfDependency {
                                 qbNamespace: vec![String::from("hello")],
                                 qbName: String::from("calc"),
                             },
                             SfDependency {
                                 qbNamespace: vec![String::from("hello")],
                                 qbName: String::from("world"),
                             }],
    }
}

use ohua_types::{AlgorithmArguments, OhuaData};
use std::env::current_dir;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{ErrorKind, Read};
use syn;
use serde_json;

mod parser;
mod algo_io_imports;

pub use self::parser::{FunctionInfo, Type};
use ohuac::OhuaProduction;
use errors::TypeExtractionError;

/*
 * what this module does:
 * take an algo
 * find the correlating modules
 * parse them & extract the type information contained within
 * return as nice package
*/

/// Knowledge base for type information of a single algorithm
#[derive(Debug)]
pub struct TypeKnowledgeBase {
    /// per-function type information
    pub functions: Vec<FunctionInfo>,
    /// Input-Output information for the algorithm
    pub algo_io: AlgorithmArguments,
}

impl TypeKnowledgeBase {
    pub fn generate_from(algo_info: &OhuaProduction) -> Result<Self, TypeExtractionError> {
        // read the files generated beforehand
        let algo_file = File::open(algo_info.ohuao.as_path())?;
        let mut algo_spec: OhuaData = serde_json::from_reader(algo_file).unwrap();
        let algo_io_file = File::open(algo_info.typedump.as_path())?;
        let algo_io_data: AlgorithmArguments = serde_json::from_reader(algo_io_file).unwrap();

        // construct the initial knowledge base
        let mut knowledgebase = TypeKnowledgeBase {
            functions: vec![],
            algo_io: algo_io_data,
        };

        for dep in algo_spec.sfDependencies.drain(..) {
            // ignore any `ohua/lang` dependencies here, these are compiler builtins
            if dep.qbNamespace == vec!["ohua".to_string(), "lang".to_string()] {
                continue;
            }

            // find correlating module
            let module_content: String =
                find_module(dep.qbNamespace.clone(), algo_info.src.as_path())?;

            // parse it
            let ast = syn::parse_file(module_content.as_str())?;

            // extract the type information contained within
            knowledgebase
                .functions
                .push(FunctionInfo::extract(dep, ast)?);
        }

        Ok(knowledgebase)
    }

    /// Collects import paths for all types mentioned in the `algo_io` member
    pub fn find_imports_for_algo_io(&self) -> Vec<String> {
        let mut imports = Vec::new();

        for input in &self.algo_io.argument_types {
            let parsed: syn::Type = match syn::parse_str(input) {
                Ok(tokens) => tokens,
                Err(e) => panic!("[Stage 4] A critical error occured while parsing the ohuac type dump output.\n This is an internal error. \n{}", e),
            };

            imports.append(&mut algo_io_imports::find_paths_for_used_types(&parsed, &self.functions));
        }

        imports
    }

    /// Searches the knowledge base for type information matching the function specified.
    pub fn info_for_function<'a>(&'a self, name: &str, namespace: &[String]) -> Option<&'a FunctionInfo> {
        for info in &self.functions {
            if info.name == name && info.namespace == namespace {
                return Some(info)
            }
        }

        None
    }
}

/// Takes a dependency and tries to locate the module within the working tree. Returns the file's content.
fn find_module(
    mut lookup_path: Vec<String>,
    ohuac_path: &Path,
) -> Result<String, TypeExtractionError> {
    // TODO: (later) What about functions from external crates?
    // TODO: Modules defined from within a file?

    // this is never supposed to happen!
    if lookup_path.is_empty() {
        #[cold]
        return Err(TypeExtractionError::MalformedAlgoImportSpec(
            ohuac_path.to_str().unwrap().to_string(),
        ));
    }

    // check whether we are supposed to resolve the path from the crate root or relatively
    let mut mod_path = match lookup_path[0].to_lowercase().as_str() {
        "self" => {
            lookup_path.remove(0);
            PathBuf::from(ohuac_path.parent().expect("Encountered malformed path!"))
        }
        "super" => {
            // FIXME: Doesn't this need to remove the first element as well?
            PathBuf::from(ohuac_path.parent().expect("Encountered malformed path!"))
        },
        _ => {
            let mut p = current_dir()?;
            p.push("src");
            p
        }
    };

    // consume the path as tokens and modify the module path
    for atom in lookup_path.drain(..) {
        match atom.as_str() {
            "super" => {
                mod_path.pop();
            }
            segment => mod_path.push(segment),
        }
    }

    // if path turns out to be a dir, access the mod.rs inside
    if mod_path.is_dir() {
        mod_path.push("mod.rs");
    } else {
        mod_path.set_extension("rs");
    }

    // open the module, raising an error if not successful
    let mut file = match File::open(mod_path.as_path()) {
        Ok(fil) => fil,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Err(
                    TypeExtractionError::ModuleNotFound(mod_path.to_str().unwrap().to_string()).into(),
                );
            } else {
                return Err(TypeExtractionError::from(e).into());
            }
        }
    };
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    println!("[Phase 2] Parsing module {}", mod_path.to_str().unwrap());

    Ok(content)
}

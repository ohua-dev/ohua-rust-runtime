use syn;

use super::parser::{collect_type, FunctionInfo};

/// Given a type, look through the `FunctionInfo` vector and extract any imports that have to be made explicitly
pub fn find_paths_for_used_types(ty: &syn::Type, infos: &Vec<FunctionInfo>) -> Vec<String> {
    let wanted_ty = collect_type(ty);

    for info in infos {
        for arg in &info.arguments {
            if arg.name == wanted_ty {
                return arg.path.clone().unwrap_or(vec![]);
            }
        }

        if info.return_val.name == wanted_ty {
            return info.return_val.path.clone().unwrap_or(vec![]);
        }
    }

    // TODO: if nothing was found, should we error or pretend it's alright?
    panic!("No type info for type {} was found.", wanted_ty);
}

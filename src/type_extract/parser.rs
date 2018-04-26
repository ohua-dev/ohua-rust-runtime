use std::collections::{HashMap, HashSet};
use ohua_types::SfDependency;
use errors::TypeExtractionError;
use syn;
use quote::{ToTokens, Tokens};

/// Type that contains type lookup information for a module. `K` is the outside name from `V`, when existing.
type LookupTable = HashMap<String, LookupInfo>;

/// Information about a single type. As types in Rust can be composited of many other types (e.g., using tuples), the `path` member might contain multiple imports
#[derive(Debug)]
pub struct Type {
    /// name of the type
    pub name: String,
    /// absolute import paths of the types, _including_ the type names themself. Is `None` if it's a type from the `prelude`.
    /// There are multiple items in the vector when a composited type is used.
    pub path: Option<Vec<String>>,
}

/// Name and type information for a single function
#[derive(Debug)]
pub struct FunctionInfo {
    /// the function name
    pub name: String,
    /// namespace the function lives in
    pub namespace: Vec<String>,
    /// argument types the function takes
    pub arguments: Vec<Type>,
    /// return type of the function
    pub return_val: Type,
}

/// Lookup information for a single `use` statement
#[derive(Clone, Debug)]
pub struct LookupInfo {
    /// The aliased name used within the module (produced by `use T as Foo`)
    pub outside_name: Option<String>,
    /// The original item name that forms the last segment of the import path
    pub original_name: String,
    /// Full import path, including the name of the imported item
    pub import_path: String,
}

impl LookupInfo {
    pub fn is_renamed(&self) -> bool {
        self.outside_name.is_some()
    }
}

impl FunctionInfo {
    /// Extracts the necessary type information for a given function from an AST.
    ///
    /// TODO: expand me
    pub fn extract(dep: SfDependency, tree: syn::File) -> Result<Self, TypeExtractionError> {
        if let Some((args, retval)) = traverse_item_tree(&tree.items, dep.clone()) {
            // return the collected information
            Ok(FunctionInfo {
                name: dep.qbName,
                namespace: dep.qbNamespace,
                arguments: args,
                return_val: retval,
            })
        } else {
            Err(TypeExtractionError::FunctionNotFoundInModule(
                dep.qbName,
                dep.qbNamespace.join("::"),
            ))
        }
    }
}

/// Traverses the token subtree for a type `T` in an attempt to extract all data types that are locally defined in this file.
///
/// The `ty` argument is supposed to be a type from an `impl T` or `impl Trait for T` statement.
/// This function is intended to be called from within `traverse_item_tree`.
fn bare_type_names(ty: &syn::Type) -> HashSet<String> {
    /* STRATEGY:
     * return all types found (even primitive ones!)
     * then compare returned set to lookup and prune all existing types as well as primitive ones (in another fn)
     */

    match *ty {
        syn::Type::Slice(ref ty_slice) => bare_type_names(&ty_slice.elem),
        syn::Type::Array(ref ty_array) => bare_type_names(&ty_array.elem),
        syn::Type::Ptr(ref ty_ptr) => bare_type_names(&ty_ptr.elem),
        syn::Type::Reference(ref ty_ref) => bare_type_names(&ty_ref.elem),
        syn::Type::BareFn(ref ty_bare_fn) => {
            let mut set: HashSet<String> = HashSet::new();
            for inp in &ty_bare_fn.inputs {
                set.extend(bare_type_names(&inp.ty));
            }

            if let syn::ReturnType::Type(_, ref t) = ty_bare_fn.output {
                set.extend(bare_type_names(&t));
            }

            set
        }
        syn::Type::Tuple(ref ty_tuple) => {
            let mut set: HashSet<String> = HashSet::new();
            for inp in &ty_tuple.elems {
                set.extend(bare_type_names(&inp));
            }
            set
        }
        syn::Type::Path(ref ty_path) => {
            // ASSUMPTION: if this type is longer than 1 element, don't do anything, it already is imported
            let mut set: HashSet<String> = HashSet::new();
            if ty_path.path.segments.len() == 1 {
                set.insert(
                    ty_path
                        .path
                        .segments
                        .first()
                        .unwrap()
                        .value()
                        .ident
                        .to_string(),
                );
            }
            set
        }
        syn::Type::Paren(ref ty_paren) => bare_type_names(&ty_paren.elem),
        syn::Type::Group(ref ty_group) => bare_type_names(&ty_group.elem),
        _ => HashSet::new(),
    }
}

/// Takes an identifier and the namespace it lives in to construct a `LookupInfo` entry for the LookupTable
fn lookup_info_from(ident: &syn::Ident, dep: &SfDependency) -> (String, LookupInfo) {
    let name = ident.to_string();
    let mut path = dep.qbNamespace
        .iter()
        .fold(String::new(), |mut acc, ref x| {
            acc += format!("{}::", x).as_str();
            acc
        });
    path += name.as_str();

    let info = LookupInfo {
        outside_name: None,
        original_name: name.clone(),
        import_path: path,
    };

    (name, info)
}

/// Starts a recursive search on the item tree of a module.
fn traverse_item_tree(items: &[syn::Item], target: SfDependency) -> Option<(Vec<Type>, Type)> {
    // TODO: Implement searches for nested modules (this will require a lookup in a mutable `target`)
    // TODO: Member functions of structs/enums do *not* work yet! Can be solved by implementing the one above
    // TODO: When recursively handing through the lookup info, provide a `Cow` value to reduce mem usage?
    let mut lookup_table = LookupTable::new();
    for item in items {
        use syn::Item;
        let result = match *item {
            Item::Use(ref item_use) => {
                let stmt = resolve_use_stmt(&item_use.tree, None);
                println!("{:#?}", stmt);
                lookup_table.extend(stmt);
                None
            }
            Item::Fn(ref item_fn) => check_function(&item_fn, target.clone(), &lookup_table),
            Item::Type(ref item_ty) => {
                let (name, info) = lookup_info_from(&item_ty.ident, &target);
                lookup_table.insert(name, info);
                None
            }
            Item::Struct(ref item_struct) => {
                let (name, info) = lookup_info_from(&item_struct.ident, &target);
                lookup_table.insert(name, info);
                None
            }
            Item::Enum(ref item_enum) => {
                let (name, info) = lookup_info_from(&item_enum.ident, &target);
                lookup_table.insert(name, info);
                None
            }
            Item::Union(ref item_union) => {
                let (name, info) = lookup_info_from(&item_union.ident, &target);
                lookup_table.insert(name, info);
                None
            }
            Item::Trait(_) => {
                // Does it even make sense to scan this?
                // -> disabling this for now. otherwise we might run into generic type bounds and we do not want that.
                // traverse_trait_tree(&item_trait, target.clone(), &lookup_table)
                None
            }
            Item::Impl(ref item_impl) => {
                traverse_impl_tree(&item_impl, target.clone(), lookup_table.clone())
            }
            Item::Mod(ref item_mod) => {
                println!(
                    "[INFO] Ignoring module {:?}, module search not implemented yet",
                    item_mod
                );
                None
            }
            Item::ForeignMod(ref item_foreign) => {
                eprintln!("Ignoring FFI block {:?}", item_foreign);
                None
            }
            _ => None,
        };

        if result.is_some() {
            return result;
        }
    }

    None
}

/// Recursively resolve a `use` statement by traversing the subtree and generating lookup info for every leaf
fn resolve_use_stmt(item_use: &syn::UseTree, current_state: Option<LookupInfo>) -> LookupTable {
    let mut table = LookupTable::new();
    let mut info = if let Some(cont) = current_state {
        cont
    } else {
        LookupInfo {
            outside_name: None,
            original_name: "".into(),
            import_path: "".into(),
        }
    };

    match *item_use {
        syn::UseTree::Path(ref use_path) => {
            // append the path info to the import path *with* trailing `::`
            info.import_path += format!("{}::", use_path.ident.to_string()).as_str();
            // hand down to next recursion level
            table = resolve_use_stmt(&use_path.tree, Some(info));
        }
        syn::UseTree::Name(ref use_name) => {
            // store the type info and stop recursion
            if use_name.ident.to_string() != "self" {
                info.original_name = use_name.ident.to_string();
                info.import_path += use_name.ident.as_ref();
            } else {
                info.original_name = info.import_path.split("::").last().unwrap().to_string();
            }
            table.insert(info.original_name.clone(), info);
        }
        syn::UseTree::Rename(ref use_rename) => {
            // store the type and renaming information and stop recursion
            info.outside_name = Some(use_rename.rename.to_string());
            info.original_name = use_rename.ident.to_string();
            info.import_path += use_rename.ident.as_ref();
            table.insert(info.outside_name.clone().unwrap(), info);
        }
        syn::UseTree::Glob(_) => {
            // TODO: Come up for a solution for global imports
            return table;
        }
        syn::UseTree::Group(ref use_group) => {
            // break recursion into pieces (one arm for every group member)
            for item in &use_group.items {
                table.extend(resolve_use_stmt(item, Some(info.clone())));
            }
        }
    }

    table
}

fn collect_type(argument: &syn::Type) -> String {
    let mut tokens = Tokens::new();
    argument.to_tokens(&mut tokens);

    tokens.to_string().replace(" ", "")
}

/// Using a previously generated lookup table, try to collect all necessary imports for a (possibly composited) type
fn find_paths_for_type(ty: &syn::Type, lookup_table: &LookupTable) -> Vec<String> {
    let mut imports = Vec::new();

    use syn::Type::*;
    match *ty {
        Slice(ref ty_slice) => imports = find_paths_for_type(&ty_slice.elem, lookup_table),
        Array(ref ty_array) => imports = find_paths_for_type(&ty_array.elem, lookup_table),
        Ptr(ref ty_ptr) => imports = find_paths_for_type(&ty_ptr.elem, lookup_table),
        Reference(ref ty_reference) => {
            imports = find_paths_for_type(&ty_reference.elem, lookup_table)
        }
        BareFn(ref ty_bare_fn) => {
            for arg in &ty_bare_fn.inputs {
                imports.append(&mut find_paths_for_type(&arg.ty, lookup_table));
            }

            if let syn::ReturnType::Type(_, ref t) = ty_bare_fn.output {
                imports.append(&mut find_paths_for_type(&t, lookup_table));
            }
        }
        Never(_) => panic!("Cannot use never-returning function inside an algorithm!"),
        Tuple(ref ty_tuple) => for ty in &ty_tuple.elems {
            imports.append(&mut find_paths_for_type(ty, lookup_table));
        },
        Path(ref ty_path) => {
            // check whether the first part of the imported path is found in the DB
            if let Some(val) = lookup_table.get(&ty_path
                .path
                .segments
                .first()
                .unwrap()
                .value()
                .ident
                .to_string())
            {
                let mut import = val.import_path.clone();
                if val.is_renamed() {
                    import += format!(" as {}", val.outside_name.clone().unwrap()).as_str();
                }
                imports.push(import);
            }
        }
        TraitObject(_) => {
            panic!("Function arguments bound by Traits are not supported by ohua yet.")
        }
        ImplTrait(_) => panic!("Function arguments bound by Traits are not supported by ohua yet."),
        Paren(ref ty_paren) => imports = find_paths_for_type(&ty_paren.elem, lookup_table),
        Group(ref ty_group) => imports = find_paths_for_type(&ty_group.elem, lookup_table),
        Infer(_) => panic!("Type placeholders are not allowed in function signatures!"),
        Macro(_) => panic!("Macros in type positions are currently not supported!"),
        Verbatim(_) => (),
    }

    imports
}

fn get_paths(ty: &syn::Type, lookup_table: &LookupTable) -> Option<Vec<String>> {
    let res = find_paths_for_type(ty, lookup_table);

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn check_function(
    item_fn: &syn::ItemFn,
    target: SfDependency,
    lookup_table: &LookupTable,
) -> Option<(Vec<Type>, Type)> {
    // abort if the function is not the one we were looking for
    if item_fn.ident.to_string() != target.qbName {
        return None;
    }

    // from here on we can be sure that this is the function we've been looking for
    let mut inputs = Vec::with_capacity(item_fn.decl.inputs.len());
    for arg in &item_fn.decl.inputs {
        match *arg {
            syn::FnArg::Captured(ref arg_captured) => inputs.push(Type {
                name: collect_type(&arg_captured.ty),
                path: get_paths(&arg_captured.ty, lookup_table),
            }),
            _ => eprintln!(
                "Encountered unsupported argument types in function {}",
                target.qbName
            ),
        }
    }

    // get the return value
    let retval = match item_fn.decl.output {
        syn::ReturnType::Default => Type {
            name: "()".into(),
            path: None,
        },
        syn::ReturnType::Type(_, ref ret_ty) => Type {
            name: collect_type(&ret_ty),
            path: get_paths(&ret_ty, lookup_table),
        },
    };

    Some((inputs, retval))
}

// TODO: move the parts from above and below into an `inspect_decl` fn

fn check_method_sig(
    method_sig: &syn::MethodSig,
    target: SfDependency,
    lookup_table: &LookupTable,
) -> Option<(Vec<Type>, Type)> {
    if method_sig.ident.to_string() != target.qbName {
        return None;
    }

    // from here on we can be sure that we've hit a match
    let mut inputs = Vec::with_capacity(method_sig.decl.inputs.len());
    for arg in &method_sig.decl.inputs {
        match *arg {
            syn::FnArg::Captured(ref arg_captured) => inputs.push(Type {
                name: collect_type(&arg_captured.ty),
                path: get_paths(&arg_captured.ty, lookup_table),
            }),
            _ => eprintln!(
                "Encountered unsupported argument types in function {}",
                target.qbName
            ),
        }
    }

    // get the return value
    let retval = match method_sig.decl.output {
        syn::ReturnType::Default => Type {
            name: "()".into(),
            path: None,
        },
        syn::ReturnType::Type(_, ref ret_ty) => Type {
            name: collect_type(&ret_ty),
            path: get_paths(&ret_ty, lookup_table),
        },
    };

    Some((inputs, retval))
}

/// traverse the contents of a trait and search for a function declaration
#[allow(dead_code)]
fn traverse_trait_tree(
    item_trait: &syn::ItemTrait,
    target: SfDependency,
    lookup_table: &LookupTable,
) -> Option<(Vec<Type>, Type)> {
    for item in &item_trait.items {
        let res = match *item {
            syn::TraitItem::Method(ref trait_method) => {
                check_method_sig(&trait_method.sig, target.clone(), lookup_table)
            }
            _ => None,
        };

        if res.is_some() {
            return res;
        }
    }

    None
}

/// Simple function that checks whether the provided type is a primitive one.
///
/// This is implemented as a lookup in a slice containing all primitive types used to date.
/// I am very ashamed of this function, but as was pointed out in various IRC discussions,
/// primitive types are not reserved keywords and therefore can't be checked otherwise as easily.
fn is_primitive_type(ty: &str) -> bool {
    let primitives = [
        "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64",
        "u128", "isize", "usize", "str", "()",
    ];

    primitives.contains(&ty)
}

fn traverse_impl_tree(
    item_impl: &syn::ItemImpl,
    target: SfDependency,
    mut lookup_table: LookupTable,
) -> Option<(Vec<Type>, Type)> {
    // add the type this `impl` is for to the lookup data

    // retrieve the name of the Type we `impl` for and construct an import path
    let mut ty_names = bare_type_names(&item_impl.self_ty);

    // sort out primitive types and types already in the lookup table
    ty_names.retain(|ref name| {
        !is_primitive_type(name.as_str()) && !lookup_table.contains_key(name.as_str())
    });

    // construct the import path once
    let path = target
        .qbNamespace
        .iter()
        .fold(String::new(), |mut acc, ref x| {
            acc += format!("{}::", x).as_str();
            acc
        });

    // add all new types (should usually only be one) to the lookup table
    for name in ty_names.drain() {
        lookup_table.insert(
            name.clone(),
            LookupInfo {
                outside_name: None,
                original_name: name.clone(),
                import_path: format!("{}{}", path, name),
            },
        );
    }

    // iteratively check every item of the impl block, inspect methods
    // the first matching method signature will be analyzed and returned
    for item in &item_impl.items {
        let res = match *item {
            syn::ImplItem::Method(ref item_method) => {
                check_method_sig(&item_method.sig, target.clone(), &lookup_table)
            }
            _ => None,
        };

        if res.is_some() {
            return res;
        }
    }

    None
}

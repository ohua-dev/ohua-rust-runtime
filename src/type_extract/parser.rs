use ohua_types::SfDependency;
use syn;
use quote::{Tokens, ToTokens};

/// Information about a single type
#[derive(Debug)]
pub struct Type {
    /// name of the type
    pub name: String,
    /// absolute import path of the type
    pub path: String, // TODO: Ending w/ or w/o the type name?
}

/// Name and type information for a single function
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

impl FunctionInfo {
    pub fn extract(dep: SfDependency, tree: syn::File) -> Self {
        if let Some(res) = traverse_item_tree(&tree.items, dep.clone()) {
            println!("{:?}", res);
        }

        FunctionInfo {
            name: dep.qbName,
            namespace: dep.qbNamespace,
            arguments: vec![],
            return_val: Type {
                name: "".into(),
                path: "".into(),
            },
        }
    }
}

fn traverse_item_tree(items: &[syn::Item], target: SfDependency) -> Option<(Vec<Type>, Type)> {
    // TODO: Implement searches for nested modules (this will require a lookup in a mutable `target`)
    for item in items {
        use syn::Item;
        let result = match *item {
            Item::Fn(ref item_fn) => check_function(&item_fn, target.clone()),
            Item::Trait(ref item_trait) => traverse_trait_tree(&item_trait, target.clone()),
            Item::Impl(ref item_impl) => traverse_impl_tree(&item_impl, target.clone()),
            Item::Mod(ref item_mod) => {
                println!("[INFO] Ignoring module {:?}, module search not implemented yet", item_mod);
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

fn collect_type(argument: &syn::Type) -> String {
    let mut tokens = Tokens::new();
    argument.to_tokens(&mut tokens);

    tokens.to_string()
}

// TODO: Continue the `check` functions
// TODO: Add the import paths to the Type structs

fn check_function(item_fn: &syn::ItemFn, target: SfDependency) -> Option<(Vec<Type>, Type)> {
    if item_fn.ident.to_string() != target.qbName {
        return None;
    }

    // from here on we can be sure that this is the function we've been looking for
    let mut inputs = Vec::with_capacity(item_fn.decl.inputs.len());
    for arg in &item_fn.decl.inputs {
        match *arg {
            syn::FnArg::Captured(ref arg_captured) => inputs.push(Type { name: collect_type(&arg_captured.ty), path: "".into() }),
            _ => eprintln!("Encountered unsupported argument types in function {}", target.qbName),
        }
    }

    // get the return value
    let retval = match item_fn.decl.output {
        syn::ReturnType::Default => Type { name: "()".into(), path: "".into() },
        syn::ReturnType::Type(_, ref ret_ty) => Type { name: collect_type(&ret_ty), path: "".into() },
    };

    Some((inputs, retval))
}

// TODO: move the parts from above and below into an `inspect_decl` fn

fn check_method_sig(method_sig: &syn::MethodSig, target: SfDependency) -> Option<(Vec<Type>, Type)> {
    if method_sig.ident.to_string() != target.qbName {
        return None;
    }

    // from here on we can be sure that we've hit a match
    let mut inputs = Vec::with_capacity(method_sig.decl.inputs.len());
    for arg in &method_sig.decl.inputs {
        match *arg {
            syn::FnArg::Captured(ref arg_captured) => inputs.push(Type { name: collect_type(&arg_captured.ty), path: "".into() }),
            _ => eprintln!("Encountered unsupported argument types in function {}", target.qbName),
        }
    }

    // get the return value
    let retval = match method_sig.decl.output {
        syn::ReturnType::Default => Type { name: "()".into(), path: "".into() },
        syn::ReturnType::Type(_, ref ret_ty) => Type { name: collect_type(&ret_ty), path: "".into() },
    };

    Some((inputs, retval))
}

/// traverse the contents of a trait and search for a function declaration
fn traverse_trait_tree(item_trait: &syn::ItemTrait, target: SfDependency) -> Option<(Vec<Type>, Type)> {
    for item in &item_trait.items {
        let res = match *item {
            syn::TraitItem::Method(ref trait_method) => check_method_sig(&trait_method.sig, target.clone()),
            _ => None,
        };

        if res.is_some() {
            return res;
        }
    }

    None
}

fn traverse_impl_tree(item_impl: &syn::ItemImpl, target: SfDependency) -> Option<(Vec<Type>, Type)> {
    // iteratively check every item of the impl block, inspect methods
    // the first matching method signature will be analyzed and returned
    for item in &item_impl.items {
        let res = match *item {
            syn::ImplItem::Method(ref item_method) => check_method_sig(&item_method.sig, target.clone()),
            _ => None,
        };

        if res.is_some() {
            return res;
        }
    }

    None
}

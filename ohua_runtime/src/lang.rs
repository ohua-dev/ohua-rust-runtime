use std::sync::mpsc::{Receiver, Sender};

// TODO the minimal implementation works on the Ord trait
#[allow(non_snake_case)]
pub fn smapFun<T>(inp: Receiver<Vec<T>>, out: Sender<T>) -> () {
    let vs = inp.recv().unwrap();
    for v in vs {
        out.send(v).unwrap();
    }
}

// a fully explicit operator version
pub fn collect<T>(n: &Receiver<i32>, data: &Receiver<T>, out: &Sender<Vec<T>>) -> () {
    match n.recv() {
        Err(_) => {
            // channels are closed by Rust itself
        }
        Ok(num) => {
            let mut buffered = Vec::new();
            for _x in 0..num {
                buffered.push(data.recv().unwrap());
            }
            out.send(buffered).unwrap();
        }
    }
}

pub fn select<T>(
    decision: Receiver<bool>,
    true_branch: Receiver<T>,
    else_branch: Receiver<T>,
    out: Sender<T>,
) -> () {
    let branch = if decision.recv().unwrap() {
        true_branch
    } else {
        else_branch
    };
    out.send(branch.recv().unwrap()).unwrap();
}

// that's also a stateful function -> in fact this thing needs variadic arguments and therefore needs to be a macro
// FIXME this probably wants to become a procedural macro! (note that proc macros can also have the form 'scope!()')
// macro_rules! scope {
//     // FIXME this is not as trivial as it seems because we need different type parameters! and therefore need a recursive macro!
//     ( $($input),+ ) => {
//         pub fn <$(T),+>scope($($input),+) -> ($(T),+) {
//             ($($input),+)
//         }
//     }
// }

/*
#[proc_macro]
pub fn scope(args: TokenStream, input: TokenStream) -> TokenStream {
    // FIXME refactor this code with the one in lib.rs!

    // Parse the input tokens into a syntax tree, extract necessary information
    let ast: Stmt = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e),
    };

    let expression: Expr = match ast {
        Stmt::Expr(e) => e,
        Stmt::Semi(e, _) => e,
        _ => panic!("Invariant broken: 'scope!' not an expr!"),
    };

    let call: ExprCall = if let Expr::Call(fn_call) = expression {
        fn_call
    } else {
        panic!("Invariant broken: 'scope!' not a function call!");
    };

    if !args.is_empty() {
        panic!("The #[ohua] macro does currently not support macro arguments.");
    }

    let algo_name: ExprPath = match *algo_call.func {
        Expr::Path(path) => path,
        _ => panic!("Malformed algorithm invocation. Expected a qualified path."),
    };
    let algo_args: Punctuated<Expr, Token![,]> = algo_call.args; // https://docs.serde.rs/syn/punctuated/index.html


    // TODO
    let type_params = unimplemented!();
    let type_params_ret = type_params.clone(); // can't reference var more than once in quasi-quote.
    let params = unimplemented!();
    let params_ret = params_ret.clone(); // can't reference var more than once in quasi-quote.

    // build the list of type parameters: T0, T1, ...
    quote!{
        pub fn <#(#type_params),+>scope(#(#params),+) -> (#(type_params_ret),+) {
            (#(#params_ret),+)
        }
    }
} */

pub fn one_to_n<T: Clone>(n: Receiver<i32>, val: Receiver<T>, out: Sender<T>) -> () {
    // TODO 2 more efficient implementations exist:
    //      1. send the key and the value once -> requires special input ports that are sensitive to that.
    //      2. send a batch -> requires input ports to understand the concept of a batch.
    // feels like the second option is the more general one (but also creates more data).
    // note: sharing the value is only possible of the function using it, does not mutate it! this can be yet another application for our knowledge base to find out which version to choose.
    let v = val.recv().unwrap();
    for _ in 0..(n.recv().unwrap()) {
        out.send(v.clone()).unwrap();
    }
}

// that's actually a stateful function
pub fn size<T>(data: &Vec<T>) -> usize {
    data.len()
}

// stateful function
pub fn id<T>(data: T) -> T {
    data
}

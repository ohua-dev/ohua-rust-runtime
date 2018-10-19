use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::Error;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, ExprCall, ExprPath, Local, Stmt};

// pub fn parse_ohua_call2(args: TokenStream2, input: TokenStream2) -> (ExprPath, Punctuated<Expr, Comma>){
//     if !args.is_empty() {
//         panic!("The #[ohua] macro does currently not support macro arguments.");
//     }
//
//     parse_call(syn::parse2(input))
// }

pub fn parse_ohua_call(
    args: TokenStream,
    input: TokenStream,
) -> ((ExprPath, Punctuated<Expr, Comma>), Option<Local>) {
    if !args.is_empty() {
        panic!("The #[ohua] macro does currently not support macro arguments.");
    }

    // Parse the input tokens into a syntax tree, extract necessary information
    let ast: Stmt = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e),
    };

    let (expression, assignment) = match ast {
        Stmt::Local(mut l) => {
            println!("found local assignment");
            let e = if let Some(exp) = l.init {
                *exp.1
            } else {
                panic!("Assignments must be initialized for use with the #[ohua] macro.")
            };
            l.init = None;
            (e, Some(l))
        }
        Stmt::Expr(e) => (e, None),
        Stmt::Semi(e, _) => (e, None),
        _ => panic!("Encountered unsupported statement after #[ohua] macro"),
    };

    (parse_expr(expression), assignment)
}

fn parse_expr(expression: Expr) -> (ExprPath, Punctuated<Expr, Comma>) {
    let algo_call: ExprCall = if let Expr::Call(fn_call) = expression {
        fn_call
    } else {
        panic!("The #[ohua] macro may only be applied to a function call.");
    };

    let algo_name: ExprPath = match *algo_call.func {
        Expr::Path(path) => path,
        _ => panic!("Malformed algorithm invocation. Expected a qualified path."),
    };
    let algo_args: Punctuated<Expr, Token![,]> = algo_call.args; // https://docs.serde.rs/syn/punctuated/index.html
    (algo_name, algo_args)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn parse_call(expression: &str) -> (ExprPath, Punctuated<Expr, Comma>) {
        let expr = match syn::parse_str::<Expr>(expression) {
            Ok(ast) => ast,
            Err(e) => panic!("{}", e),
        };
        parse_expr(expr)
    }
}

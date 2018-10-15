use syn::parse::Parse;
use syn::token::Comma;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprCall, ExprPath, Stmt};
use syn::parse::Error;


// pub fn parse_ohua_call2(args: TokenStream2, input: TokenStream2) -> (ExprPath, Punctuated<Expr, Comma>){
//     if !args.is_empty() {
//         panic!("The #[ohua] macro does currently not support macro arguments.");
//     }
//
//     parse_call(syn::parse2(input))
// }

pub fn parse_ohua_call(args: TokenStream, input: TokenStream) -> (ExprPath, Punctuated<Expr, Comma>){

    if !args.is_empty() {
        panic!("The #[ohua] macro does currently not support macro arguments.");
    }

    // Parse the input tokens into a syntax tree, extract necessary information
    let ast: Stmt = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e),
    };

    let expression: Expr = match ast {
        Stmt::Expr(e) => e,
        Stmt::Semi(e, _) => e,
        _ => panic!("Encountered unsupported statement after #[ohua] macro"),
    };

    parse_expr(expression)
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
        let expr = match syn::parse_str::<Expr>(expression){
            Ok(ast) => ast,
            Err(e) => panic!("{}", e),
        };
        parse_expr(expr)
    }
}

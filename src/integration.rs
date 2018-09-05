#![feature(proc_macro)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use self::proc_macro::TokenStream;
use syn::{ExprCall, Expr, Ident};
use syn::punctuated::Punctuated;

use codegen::typedgen::*;

#[proc_macro_attribute]
// #[proc_macro]
pub fn ohua(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let algoCall: ExprCall = syn::parse(input).unwrap();
    let algoName: Ident = algoCall.method;
    let args: Punctuated<Expr, Token![,]> = algoCall.args; // https://docs.serde.rs/syn/punctuated/index.html


    // perform code generation right here
    // TODO Felix: - locate and load the algo file
    //             - run the ohua-core compiler to generate the output (catch it as a string)
    //             - create the OhuaData structure from the compiler output
    let compiled_ohua = unimplemented!();
    // let stream = TokenStream::new();
    // let tokens: Result<TokenStream, LexError> = stream.from_str(&code);

    // TODO relocated the overall structure into a quote
    // let header_code = unimplemented!();
    let arc_code = generate_arcs(compiled_ohua);
    let op_code = generate_sfns(compiled_ohua); // Vec<String>
    let final_code = quote!{
        // FIXME we can not have header code. all functions/identifiers need to be fully qualified.
        // #header_code
        {
            #arc_code

            #op_code

            run_ohua(tasks)
        }
    };

    // Hand the output tokens back to the compiler
    final_code.into()
}

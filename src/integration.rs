extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::ExprCall;
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn ohua(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let algoCall: ExprCall = syn::parse(input).unwrap();
    let algoName: Ident = algoCall.method;
    let args: Punctuated<Expr, Token![,]> = algoCall.args; //https://docs.serde.rs/syn/punctuated/index.html


    // perform code generation right here
    // TODO Felix: - locate and load the algo file
    //             - run the ohua-core compiler to generate the output (catch it as a string)
    //             - create the OhuaData structure from the compiler output
    let compiledOhua = unimplemented!();
    let code = code_generation(compiledOhua);
    let stream = TokenStream::new();
    let tokens: Result<TokenStream, LexError> = stream.from_str(&code);

    // Hand the output tokens back to the compiler
    match tokens {
        Err(lex_error) => { unimplemented!() } // TODO Felix
        Ok(s) => s
    }
}

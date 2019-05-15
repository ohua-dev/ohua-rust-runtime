use proc_macro2::{Ident, Span, TokenStream};

//
// The ctrl operator needs to be stateful but can not define its own state.
// The state of the operator would have to capture the data retrieved from the
// arcs of the vars. But what would be their type???
// For a generic backend, we don't ever deal with types. We rely solely on the
// type inference mechanisms of the target language.
// As such, we can not write or even generate a function that creates this state.
// THIS IS A GENERAL INSIGHT: state in operators (which are meant to be polymorph with
// respect to the data in the graph) can never contain anything that is related to the
// data in the graph!
// As such, the only way to write such an operator is using tail recursion as shown below.
//
pub fn generate_ctrl_operator(num_args: usize) -> TokenStream {
    assert!(num_args != 0);

    let fn_name = Ident::new(&format!("ctrl_{}", num_args), Span::call_site());

    let ref vars_in: Vec<Ident> = (0..num_args)
        .map(|arg_idx| {
            Ident::new(
                &format!("var_in_{}", arg_idx.to_string()),
                Span::call_site(),
            )
        })
        .collect();
    let ref vars_out: Vec<Ident> = (0..num_args)
        .map(|arg_idx| {
            Ident::new(
                &format!("var_out_{}", arg_idx.to_string()),
                Span::call_site(),
            )
        })
        .collect();
    let ref vars: Vec<Ident> = (0..num_args)
        .map(|arg_idx| Ident::new(&format!("var_{}", arg_idx.to_string()), Span::call_site()))
        .collect();
    let ref type_vars: Vec<Ident> = (0..num_args)
        .map(|arg_idx| Ident::new(&format!("T{}", arg_idx.to_string()), Span::call_site()))
        .collect();

    // The following block is necessary due to https://github.com/dtolnay/quote/issues/8
    let type_vars2 = type_vars;
    let type_vars3 = type_vars;
    let vars2 = vars;
    let vars_in2 = vars_in;
    let vars_in3 = vars_in;
    let vars_out2 = vars_out;

    quote! {
        fn #fn_name<#(#type_vars:Clone + Send),*>(
            ctrl_inp:&Receiver<(bool,isize)>,
            #(#vars_in:&Receiver<#type_vars2>),* ,
            #(#vars_out:&dyn ArcInput<#type_vars3>),*
        ) -> Result<(), RunError> {
            let mut renew = false;
            let mut state = ( #(#vars_in2.recv()? , )* );

            loop {
                let (renew_next_time, count) = ctrl_inp.recv()?;
                if renew {
                    state = ( #(#vars_in3.recv()? , )* );
                }

                for _ in 0..count {
                    let (#(#vars , )*) = state.clone();
                    #(#vars_out2.dispatch(#vars2.clone())?;)*
                }

                renew = renew_next_time;
            }
        };
    }
}

// // Instead of making the code above more complex, I write it here again.
// pub fn generate_ctrl_1_operator() -> TokenStream {
//     quote!{
//         fn ctrl_1<T1:Clone>(
//             ctrl_inp:&Receiver<(bool,isize)>,
//             var_in_1:&Receiver<T1>,
//             var_out_1:&dyn ArcInput<T1>) {
//           let (renew_next_time, count) = ctrl_inp.recv().unwrap();
//           let var_1 = var_in_1.recv().unwrap();
//           for _ in 0..count {
//               var_out_1.dispatch(var_1.clone()).unwrap();
//           };
//           ctrl_sf_1(ctrl_inp,
//                     var_in_1,
//                     var_out_1 ,
//                     renew_next_time,
//                     var_1)
//         };
//
//         fn ctrl_sf_1<T1:Clone>(
//             ctrl_inp:&Receiver<(bool,isize)>,
//             var_in_1:&Receiver<T1>,
//             var_out_1:&dyn ArcInput<T1>,
//             renew: bool,
//             state_var: T1) {
//           let (renew_next_time, count) = ctrl_inp.recv().unwrap();
//           let var_1 = if renew {
//                           var_in_1.recv().unwrap()
//                      } else {
//                          // reuse the captured var
//                          state_var
//                      };
//           for _ in 0..count {
//               var_out_1.dispatch(var_1.clone()).unwrap();
//           };
//           ctrl_sf_1(ctrl_inp,
//                     var_in_1,
//                     var_out_1,
//                     renew_next_time,
//                     var_1)
//         };
//     }
// }

pub fn generate_nth(op: &i32, num: &i32, len: &i32) -> TokenStream {
    let fn_name = Ident::new(&format!("nth_op{}_{}_{}", op, num, len), Span::call_site());
    let ref vars: Vec<Ident> = (0..*len)
        .map(|var_idx| Ident::new(&format!("var_{}", var_idx.to_string()), Span::call_site()))
        .collect();
    let ref type_vars: Vec<Ident> = (0..*len)
        .map(|arg_idx| Ident::new(&format!("T{}", arg_idx.to_string()), Span::call_site()))
        .collect();
    let type_vars2 = type_vars;
    let type_var = Ident::new(&format!("T{}", num), Span::call_site());
    let var = Ident::new(&format!("var_{}", num), Span::call_site());
    quote! {
        fn #fn_name< #(#type_vars),* >(t:(#(#type_vars2),*)) -> #type_var {
            let (#(#vars),*) = t;
            #var
        };
    }
}

// This does not work either because the compiler wants to have the type of the closure parameter!
// pub fn generate_nth(num:i32) -> TokenStream {
//     let fn_name = Ident::new(&format!("nth_{}", num), Span::call_site());
//     let n = Literal::i32_unsuffixed(num);
//     quote!{
//         let #fn_name = |t| { t.#n };
//     }
// }

pub mod generate_recur {

    use proc_macro2::{Ident, Span, TokenStream};

    type Len = usize;

    fn std_ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }

    pub fn generate_fun_name(len: Len) -> String {
        format!("recur_{}", len)
    }

    pub fn generate(len: Len) -> TokenStream {
        let fn_name = std_ident(&generate_fun_name(len));
        let ref initial_args: Vec<Ident> = (0..len)
            .map(|idx| std_ident(&format!("init_{}", idx.to_string())))
            .collect();
        let ref arg_types: Vec<Ident> = (0..len)
            .map(|idx| std_ident(&format!("T{}", idx.to_string())))
            .collect();
        let ref loop_args: Vec<Ident> = (0..len)
            .map(|idx| std_ident(&format!("loop_in_{}", idx.to_string())))
            .collect();
        let ref loop_out_args: Vec<Ident> = (0..len)
            .map(|idx| std_ident(&format!("loop_out_{}", idx.to_string())))
            .collect();
        let ref return_type = std_ident("R");

        let return_type0 = return_type;
        let return_type1 = return_type;
        let return_type2 = return_type;

        let arg_types0 = arg_types;
        let arg_types1 = arg_types;
        let arg_types2 = arg_types;

        let loop_args0 = loop_args;

        let loop_out_args0 = loop_out_args;
        let loop_out_args1 = loop_out_args;

        let initial_args0 = initial_args;

        // TODO: No tuple here!

        quote! {
            fn #fn_name<#(#arg_types0 : Send),*, #return_type2 : Send>
                (condition: &Receiver<bool>,
                 result_arc: &Receiver<#return_type0>,
                 #(#initial_args0 : &Receiver<#arg_types1>),*,
                 #(#loop_args0 : &Receiver<#arg_types2>),*,
                 ctrl_arc: &dyn ArcInput<(bool, isize)>,
                 #(#loop_out_args0 : &dyn ArcInput<#arg_types>),*,
                 finish_arc: &dyn ArcInput<#return_type1>,
                ) -> Result<(), RunError>
            {
                ctrl_arc.dispatch((true, 1));
                #(#loop_out_args1.dispatch(#initial_args.recv()?));*;
                /* cont_arc.dispatch(
                    (#(#initial_args.recv()?),*)
                ); */
                while (condition.recv()?) {
                    // FIXME: Pull and discard contents from the result_arc until an upstream fix is deployed.
                    let _ = result_arc.recv()?;

                    ctrl_arc.dispatch((true, 1));
                    #(#loop_out_args.dispatch(#loop_args.recv()?));*;
                    // cont_arc.dispatch((#(#loop_args.recv()?),*));
                }
                ctrl_arc.dispatch((false, 0));
                finish_arc.dispatch(result_arc.recv()?);
                Ok(())
            }
        }
    }
}

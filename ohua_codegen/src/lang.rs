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
    let fn_name = Ident::new(&format!("ctrl_{}", num_args), Span::call_site());
    let sfn_name = Ident::new(&format!("ctrl_sf_{}", num_args), Span::call_site());

    // FIXME: Hack! This adds a fallback case for `ctrl` operators that do not lead anywhere.
    // *Must* be replaced with an assertion that this does not happen anymore once the issue
    // is resolved in core.
    if num_args == 0 {
        println!("[DEBUG] Found 0 operators, generating workaround operators");

        return quote! {
            fn #fn_name(
                ctrl_inp:&Receiver<(bool,isize)>) -> Result<(), RunError> {
              let (renew_next_time, count) = ctrl_inp.recv()?;
              #sfn_name(ctrl_inp,
                        (),
                        (),
                        renew_next_time,
                        ())
            };

            fn #sfn_name(
                ctrl_inp:&Receiver<(bool,isize)>,
                _: (),
                _: (),
                renew: bool,
                state_vars: ()) -> Result<(), RunError> {
              let (renew_next_time, count) = ctrl_inp.recv()?;
              #sfn_name(ctrl_inp,
                        (),
                        (),
                        renew_next_time,
                        ())
            };
        }
    }

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

    // The following block is necessary until https://github.com/dtolnay/quote/issues/8 is closed (which will hopefully happen eventually)
    let type_vars2 = type_vars;
    let type_vars3 = type_vars;
    let type_vars4 = type_vars;
    let type_vars5 = type_vars;
    let type_vars6 = type_vars;
    let type_vars7 = type_vars;
    let vars2 = vars;
    let vars3 = vars;
    let vars4 = vars;
    let vars5 = vars;
    let vars6 = vars;
    let vars_in2 = vars_in;
    let vars_in3 = vars_in;
    let vars_in4 = vars_in;
    let vars_in5 = vars_in;
    let vars_in6 = vars_in;
    let vars_out2 = vars_out;
    let vars_out3 = vars_out;
    let vars_out4 = vars_out;
    let vars_out5 = vars_out;
    let vars_out6 = vars_out;

    quote! {
        fn #fn_name<#(#type_vars:Clone + Send),*>(
            ctrl_inp:&Receiver<(bool,isize)>,
            #(#vars_in:&Receiver<#type_vars2>),* ,
            #(#vars_out:&Sender<#type_vars3>),*) -> Result<(), RunError> {
          let (renew_next_time, count) = ctrl_inp.recv()?;
          let (#(#vars , )*) = ( #(#vars_in2.recv()? , )* );
          for _ in 0..count {
              #(#vars_out2.send(#vars2.clone())?;)*
          };
          #sfn_name(ctrl_inp,
                    #(#vars_in3),* ,
                    #(#vars_out3),* ,
                    renew_next_time,
                    (#(#vars3 , )*))
        };

        fn #sfn_name<#(#type_vars4:Clone + Send),*>(
            ctrl_inp:&Receiver<(bool,isize)>,
            #(#vars_in4:&Receiver<#type_vars5>),* ,
            #(#vars_out4:&Sender<#type_vars6>),* ,
            renew: bool,
            state_vars:(#(#type_vars7 , )*)) -> Result<(), RunError> {
          let (renew_next_time, count) = ctrl_inp.recv()?;
          let (#(#vars4,)*) = if renew {
                          ( #(#vars_in5.recv()? , )* )
                     } else {
                         // reuse the captured vars
                         state_vars
                     };
          for _ in 0..count {
              #(#vars_out5.send(#vars5.clone())?;)*
          };
          #sfn_name(ctrl_inp,
                    #(#vars_in6),* ,
                    #(#vars_out6),* ,
                    renew_next_time,
                    (#(#vars6 , )*))
        };
    }
}

// // Instead of making the code above more complex, I write it here again.
// pub fn generate_ctrl_1_operator() -> TokenStream {
//     quote!{
//         fn ctrl_1<T1:Clone>(
//             ctrl_inp:&Receiver<(bool,isize)>,
//             var_in_1:&Receiver<T1>,
//             var_out_1:&Sender<T1>) {
//           let (renew_next_time, count) = ctrl_inp.recv().unwrap();
//           let var_1 = var_in_1.recv().unwrap();
//           for _ in 0..count {
//               var_out_1.send(var_1.clone()).unwrap();
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
//             var_out_1:&Sender<T1>,
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
//               var_out_1.send(var_1.clone()).unwrap();
//           };
//           ctrl_sf_1(ctrl_inp,
//                     var_in_1,
//                     var_out_1,
//                     renew_next_time,
//                     var_1)
//         };
//     }
// }

pub fn generate_nth(num: &i32, len: &i32) -> TokenStream {
    let fn_name = Ident::new(&format!("nth_{}_{}", num, len), Span::call_site());
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
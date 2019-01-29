mod smap_fns;

use ohua_codegen::ohua;

#[test]
fn smap() {
    #[ohua]
    let x = smap::algorithms::smap_test();

    assert!(
        x == vec![
            "I hate giant spiders",
            "Why are there everywhere giant spiders",
            "there is a huge pile of giant spiders"
        ]
    );
}

#[test]
fn smap_with_lambdas() {
    #[ohua]
    let x = smap::algorithms::lambda_test();

    assert!(x == vec![8, 168, 28, 48, 740, 3772, 1500]);
}

#[test]
fn smap_with_envarc_input() {
    let inputs: Vec<String> = vec![
        "I hate".into(),
        "Why are there everywhere".into(),
        "there is a huge pile of".into(),
    ];

    #[ohua]
    let x = smap::algorithms::smap_env_test(inputs);

    assert!(
        x == vec![
            "I hate giant spiders",
            "Why are there everywhere giant spiders",
            "there is a huge pile of giant spiders"
        ]
    );
}

// FIXME: Frozen until closure of ohua-dev/ohua-code#28
#[test]
fn smap_with_envarc_in_loop() {
    unimplemented!("Frozen until closure of ohua-dev/ohua-code#28")
//     #[ohua]
//     let x = smap::algorithms::smap_env_in_loop(String::from(" giant spiders"));
//
//     assert!(
//         x == vec![
//             "I hate giant spiders",
//             "Why are there everywhere giant spiders",
//             "there is a huge pile of giant spiders"
//         ]
//     );
}

#[test]
fn smap_seq_test() {
    #[ohua]
    let x = smap::algorithms::seq_test();

    assert!(x == vec![4, 4, 4, 4, 4, 4, 4]);
}

#[test]
fn if_in_smap() {
    unimplemented!("Requires Bugfix: too many arguments for smapFun")
//     // #[ohua]
//     // let x = smap::algorithms::if_in_smap();
//
//     let x = {
//         use ohua_runtime::lang::collect;
//         use ohua_runtime::lang::ifFun;
//         use ohua_runtime::lang::select;
//         use ohua_runtime::lang::smapFun;
//         use ohua_runtime::lang::unitFn;
//         use ohua_runtime::lang::{send_once, Unit};
//         use ohua_runtime::*;
//         use smap::smap_fns::calculate;
//         use smap::smap_fns::generate_data;
//         use smap::smap_fns::generate_value;
//         use smap::smap_fns::is_even;
//         use std::boxed::FnBox;
//         use std::sync::mpsc::Receiver;
//         fn ctrl_1<T0: Clone + Send>(
//             ctrl_inp: &Receiver<(bool, isize)>,
//             var_in_0: &Receiver<T0>,
//             var_out_0: &dyn ArcInput<T0>,
//         ) -> Result<(), RunError> {
//             let (renew_next_time, count) = ctrl_inp.recv()?;
//             let (var_0,) = (var_in_0.recv()?,);
//             for _ in 0..count {
//                 var_out_0.dispatch(var_0.clone())?;
//             }
//             ctrl_sf_1(ctrl_inp, var_in_0, var_out_0, renew_next_time, (var_0,))
//         };
//         fn ctrl_sf_1<T0: Clone + Send>(
//             ctrl_inp: &Receiver<(bool, isize)>,
//             var_in_0: &Receiver<T0>,
//             var_out_0: &dyn ArcInput<T0>,
//             renew: bool,
//             state_vars: (T0,),
//         ) -> Result<(), RunError> {
//             let (renew_next_time, count) = ctrl_inp.recv()?;
//             let (var_0,) = if renew {
//                 (var_in_0.recv()?,)
//             } else {
//                 state_vars
//             };
//             for _ in 0..count {
//                 var_out_0.dispatch(var_0.clone())?;
//             }
//             ctrl_sf_1(ctrl_inp, var_in_0, var_out_0, renew_next_time, (var_0,))
//         };
//         fn ctrl_2<T0: Clone + Send, T1: Clone + Send>(
//             ctrl_inp: &Receiver<(bool, isize)>,
//             var_in_0: &Receiver<T0>,
//             var_in_1: &Receiver<T1>,
//             var_out_0: &dyn ArcInput<T0>,
//             var_out_1: &dyn ArcInput<T1>,
//         ) -> Result<(), RunError> {
//             let (renew_next_time, count) = ctrl_inp.recv()?;
//             let (var_0, var_1) = (var_in_0.recv()?, var_in_1.recv()?);
//
//             for _ in 0..count {
//                 var_out_0.dispatch(var_0.clone())?;
//                 var_out_1.dispatch(var_1.clone())?;
//             }
//             ctrl_sf_2(
//                 ctrl_inp,
//                 var_in_0,
//                 var_in_1,
//                 var_out_0,
//                 var_out_1,
//                 renew_next_time,
//                 (var_0, var_1),
//             )
//         };
//         fn ctrl_sf_2<T0: Clone + Send, T1: Clone + Send>(
//             ctrl_inp: &Receiver<(bool, isize)>,
//             var_in_0: &Receiver<T0>,
//             var_in_1: &Receiver<T1>,
//             var_out_0: &dyn ArcInput<T0>,
//             var_out_1: &dyn ArcInput<T1>,
//             renew: bool,
//             state_vars: (T0, T1),
//         ) -> Result<(), RunError> {
//             let (renew_next_time, count) = ctrl_inp.recv()?;
//             let (var_0, var_1) = if renew {
//                 (var_in_0.recv()?, var_in_1.recv()?)
//             } else {
//                 state_vars
//             };
//
//             for _ in 0..count {
//                 var_out_0.dispatch(var_0.clone())?;
//                 var_out_1.dispatch(var_1.clone())?;
//             }
//             ctrl_sf_2(
//                 ctrl_inp,
//                 var_in_0,
//                 var_in_1,
//                 var_out_0,
//                 var_out_1,
//                 renew_next_time,
//                 (var_0, var_1),
//             )
//         };
//         let (sf_2_out_0__sf_3_in_0, sf_3_in_0) = std::sync::mpsc::channel();
//         let (sf_3_out_1__sf_7_in_0, sf_7_in_0) = std::sync::mpsc::channel();
//         let (sf_1_out_0__sf_7_in_1, sf_7_in_1) = std::sync::mpsc::channel();
//         let (sf_3_out_0__sf_10_in_0, sf_10_in_0) = std::sync::mpsc::channel();
//
//         let (sf_10_out_0__sf_11_in_0, sf_11_in_0) = std::sync::mpsc::channel();
//         let (sf_11_out_0__sf_14_in_0, sf_14_in_0) = std::sync::mpsc::channel();
//         let (sf_3_out_0__sf_14_in_1, sf_14_in_1) = std::sync::mpsc::channel();
//         let (sf_7_out_0__sf_14_in_2, sf_14_in_2) = std::sync::mpsc::channel();
//         let (sf_14_out_0__sf_17_in_0, sf_17_in_0) = std::sync::mpsc::channel();
//         let (sf_14_out_1__sf_17_in_1, sf_17_in_1) = std::sync::mpsc::channel();
//
//         let (sf_11_out_1__sf_18_in_0, sf_18_in_0) = std::sync::mpsc::channel();
//         let (sf_7_out_1__sf_18_in_1, sf_18_in_1) = std::sync::mpsc::channel();
//         let (sf_10_out_0__sf_20_in_0, sf_20_in_0) = std::sync::mpsc::channel();
//         let (sf_17_out_0__sf_20_in_1, sf_20_in_1) = std::sync::mpsc::channel();
//         let (sf_18_out_0__sf_20_in_2, sf_20_in_2) = std::sync::mpsc::channel();
//         let (sf_3_out_2__sf_21_in_0, sf_21_in_0) = std::sync::mpsc::channel();
//
//         let (sf_20_out_0__sf_21_in_1, sf_21_in_1) = std::sync::mpsc::channel();
//         let (result_snd, result_rcv) = std::sync::mpsc::channel();
//
//         let mut tasks: Vec<Box<FnBox() -> Result<(), RunError> + Send + 'static>> = Vec::new();
//         tasks.push(Box::new(move || {
//             let r = unitFn(generate_value, Unit {});
//             sf_1_out_0__sf_7_in_1.dispatch(r)?;
//             Ok(())
//         }));
//         tasks.push(Box::new(move || {
//             let r = unitFn(generate_data, Unit {});
//             sf_2_out_0__sf_3_in_0.dispatch(r)?;
//             Ok(())
//         }));
//         tasks.push(Box::new(move || loop {
//             let r = is_even(sf_10_in_0.recv()?);
//             sf_10_out_0__sf_11_in_0.dispatch(r.clone())?;
//             sf_10_out_0__sf_20_in_0.dispatch(r.clone())?;
//         }));
//         tasks.push(Box::new(move || loop {
//             let r = calculate(sf_17_in_0.recv()?, sf_17_in_1.recv()?);
//
//             sf_17_out_0__sf_20_in_1.dispatch(r)?
//         }));
//         tasks.push(Box::new(move || loop {
//             smapFun(
//                 &sf_3_in_0,
//                 &sf_3_out_0__sf_10_in_0,
//                 &sf_3_out_0__sf_14_in_1,
//                 &sf_3_out_1__sf_7_in_0,
//                 &sf_3_out_2__sf_21_in_0,
//             )?;
//         }));
//         tasks.push(Box::new(move || loop {
//             ctrl_2(
//                 &sf_7_in_0,
//                 &sf_7_in_1,
//                 &send_once(0),
//                 &sf_7_out_0__sf_14_in_2,
//                 &sf_7_out_1__sf_18_in_1,
//             )?;
//         }));
//         tasks.push(Box::new(move || loop {
//             ifFun(
//                 &sf_11_in_0,
//                 &sf_11_out_0__sf_14_in_0,
//                 &sf_11_out_1__sf_18_in_0,
//             )?;
//         }));
//         tasks.push(Box::new(move || loop {
//             ctrl_2(
//                 &sf_14_in_0,
//                 &sf_14_in_1,
//                 &sf_14_in_2,
//                 &sf_14_out_0__sf_17_in_0,
//                 &sf_14_out_1__sf_17_in_1,
//             )?;
//         }));
//         tasks.push(Box::new(move || loop {
//             ctrl_1(&sf_18_in_0, &sf_18_in_1, &sf_18_out_0__sf_20_in_2)?;
//         }));
//         tasks.push(Box::new(move || loop {
//             select(
//                 &sf_20_in_0,
//                 &sf_20_in_1,
//                 &sf_20_in_2,
//                 &sf_20_out_0__sf_21_in_1,
//             )?;
//         }));
//         tasks.push(Box::new(move || loop {
//             collect(&sf_21_in_0, &sf_21_in_1, &result_snd)?;
//         }));
//         run_tasks(tasks);
//         result_rcv.recv().unwrap()
//     };
//
//     assert!(x == vec![8, 168, 0, 48, 0, 0, 0]);
}

#[test]
fn smap_in_smap() {
    unimplemented!("Not implemented yet")
}

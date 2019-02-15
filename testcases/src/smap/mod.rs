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

#[test]
fn smap_with_envarc_in_loop() {
    unimplemented!("Requires Bugfix: (same as smap_seq_test -> see comments in testcase!)");
    // #[ohua]
    // let x = smap::algorithms::smap_env_in_loop(String::from(" giant spiders"));

    // let x = {
    //     use ohua_runtime::lang::collect;
    //     use ohua_runtime::lang::id;
    //     use ohua_runtime::lang::smapFun;
    //     use ohua_runtime::lang::unitFn;
    //     use ohua_runtime::lang::{send_once, Unit};
    //     use ohua_runtime::*;
    //     use smap::smap_fns::fuse;
    //     use smap::smap_fns::gen_input;
    //     use std::boxed::FnBox;
    //     use std::sync::mpsc::Receiver;
    //     let (sf_1_out_0__sf_2_in_0, sf_2_in_0) = std::sync::mpsc::channel();
    //
    //     let (sf_2_out_0__sf_7_in_0, sf_7_in_0) = std::sync::mpsc::channel();
    //     let (sf_9_out_0__sf_7_in_1, sf_7_in_1) = std::sync::mpsc::channel();
    //     let (sf_2_out_2__sf_8_in_0, sf_8_in_0) = std::sync::mpsc::channel();
    //     let (sf_7_out_0__sf_8_in_1, sf_8_in_1) = std::sync::mpsc::channel();
    //     let sf_2_out_1__sf_6_in_0 = DeadEndArc::default();
    //     let (result_snd, result_rcv) = std::sync::mpsc::channel();
    //     let mut tasks: Vec<Box<FnBox() -> Result<(), RunError> + Send + 'static>> = Vec::new();
    //     tasks.push(Box::new(move || {
    //         let r = unitFn(gen_input, Unit {});
    //         sf_1_out_0__sf_2_in_0.dispatch(r)?;
    //         Ok(())
    //     }));
    //     tasks.push(Box::new(move || loop {
    //         let r = fuse(sf_7_in_0.recv()?, sf_7_in_1.recv()?);
    //
    //         sf_7_out_0__sf_8_in_1.dispatch(r)?
    //     }));
    //     tasks.push(Box::new(move || {
    //         // FIXME: This is only pushed into the channel once but `collect` tries to pull several times -> Error
    //         let r = id(String::from(" giant spiders"));
    //         sf_9_out_0__sf_7_in_1.dispatch(r)?;
    //         Ok(())
    //     }));
    //     tasks.push(Box::new(move || loop {
    //         smapFun(
    //             &sf_2_in_0,
    //             &sf_2_out_0__sf_7_in_0,
    //             &sf_2_out_1__sf_6_in_0,
    //             &sf_2_out_2__sf_8_in_0,
    //         )?;
    //     }));
    //     tasks.push(Box::new(move || loop {
    //         collect(&sf_8_in_0, &sf_8_in_1, &result_snd)?;
    //     }));
    //     run_tasks(tasks);
    //     result_rcv.recv().unwrap()
    // };

    // assert!(
    //     x == vec![
    //         "I hate giant spiders",
    //         "Why are there everywhere giant spiders",
    //         "there is a huge pile of giant spiders"
    //     ]
    // );
}

#[test]
fn smap_independent_fns_test() {
    unimplemented!("Requires Bugfix: Too few data sent from unitFn to channel `collect` pulls from. (See comments in testcase!)");
    // #[ohua]
    // let x = smap::algorithms::independent_fns();

    // let x = {
    //     use ohua_runtime::lang::collect;
    //     use ohua_runtime::lang::smapFun;
    //     use ohua_runtime::lang::unitFn;
    //     use ohua_runtime::lang::{send_once, Unit};
    //     use ohua_runtime::*;
    //     use smap::smap_fns::generate_data;
    //     use smap::smap_fns::generate_value;
    //     use std::boxed::FnBox;
    //     use std::sync::mpsc::Receiver;
    //     let (sf_1_out_0__sf_2_in_0, sf_2_in_0) = std::sync::mpsc::channel();
    //     let (sf_2_out_2__sf_8_in_0, sf_8_in_0) = std::sync::mpsc::channel();
    //
    //     let (sf_7_out_0__sf_8_in_1, sf_8_in_1) = std::sync::mpsc::channel();
    //     let sf_2_out_1__sf_6_in_0 = DeadEndArc::default();
    //     let sf_2_out_0__sf_0_in_0 = DeadEndArc::default();
    //     let (result_snd, result_rcv) = std::sync::mpsc::channel();
    //     let mut tasks: Vec<Box<FnBox() -> Result<(), RunError> + Send + 'static>> = Vec::new();
    //     tasks.push(Box::new(move || {
    //         let r = unitFn(generate_data, Unit {});
    //         sf_1_out_0__sf_2_in_0.dispatch(r)?;
    //         Ok(())
    //     }));
    //     tasks.push(Box::new(move || {
    //         // FIXME: this code pushes the output of `unitFn` once into the channel and then exits.
    //         // Problem: `collect` will pull 7 times from the channel and thus fail, yielding nothing
    //         // in the result channel.
    //         let r = unitFn(generate_value, Unit {});
    //         sf_7_out_0__sf_8_in_1.dispatch(r)?;
    //         Ok(())
    //     }));
    //     tasks.push(Box::new(move || loop {
    //         smapFun(
    //             &sf_2_in_0,
    //             &sf_2_out_0__sf_0_in_0,
    //             &sf_2_out_1__sf_6_in_0,
    //             &sf_2_out_2__sf_8_in_0,
    //         )?;
    //     }));
    //     tasks.push(Box::new(move || loop {
    //         collect(&sf_8_in_0, &sf_8_in_1, &result_snd)?;
    //     }));
    //     run_tasks(tasks);
    //     match result_rcv.recv() {
    //         Ok(x) => x,
    //         Err(e) => {println!("{}", e); vec![]}
    //     }
    // };

    // assert!(x == vec![4, 4, 4, 4, 4, 4, 4]);
}

#[test]
fn if_in_smap() {
    #[ohua]
    let x = smap::algorithms::if_in_smap();

    assert!(x == vec![8, 168, 0, 48, 0, 0, 0]);
}

#[test]
fn smap_in_smap() {
    #[ohua]
    let x = smap::algorithms::smap_in_smap();

    assert!(
        x == vec![
            vec!["original giant spiders", "modified giant spiders"],
            vec!["modified giant spiders", "original giant spiders"]
        ]
    )
}

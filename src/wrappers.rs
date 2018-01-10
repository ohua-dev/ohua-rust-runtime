pub fn wrap_function(name: &str, incoming_arcs: u16, outgoing_arcs: u16) {
    let mut skeleton = String::from(include_str!("templates/snippets/wrapper.in"));
    let unpack_arg = "let arg{n} = Box::from(args.pop().unwrap());\n";

    // set the wrapper function name
    skeleton = skeleton.replace("{escaped_name}", name.replace("::", "_").as_ref());
    
    // incoming arguments
    let mut incoming = String::new();
    for i in 0..incoming_arcs {
        incoming.push_str(&unpack_arg.replace("{n}", format!("{}", i).as_ref()));
    }
    skeleton = skeleton.replace("{incoming_args}", &incoming);

    // function call (name and arguments)
    let mut fn_args = String::new();
    for i in 0..incoming_arcs {
        if i > 0 {
            fn_args.push_str(", ")
        }
        fn_args.push_str(format!("*arg{}", i).as_ref());
    }

    skeleton = skeleton.replace("{name}", name).replace("{fn_args}", &fn_args);

    // outgoing values (unwrapping from tuple (if necessary) and vec appending)
    if outgoing_arcs != 1 {
        let mut outgoing = String::new();
        for i in 0..outgoing_arcs {
            if i > 0 {
            outgoing.push_str(", ")
        }
            outgoing.push_str(format!("Box::from(Box::new(res.{}))", i).as_ref());
        }
        skeleton = skeleton.replace("{outgoing_args}", &outgoing);
    } else {
        skeleton = skeleton.replace("{outgoing_args}", "Box::from(Box::new(res))");
    }

    println!("{}", skeleton);
}

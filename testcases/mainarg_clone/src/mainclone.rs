pub fn concat(mut arg1: String, arg2: String) -> String {
    arg1 += &arg2;
    arg1
}

pub fn append_foo(mut arg: String) -> String {
    arg += " foo";
    arg
}

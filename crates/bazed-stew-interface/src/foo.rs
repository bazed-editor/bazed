/*
stew_plugin! {
    api_version = "1.1",
    version = b"1.1.0\0",
    name = b"I am an epic plugin\0",
    init = init,
    main = main,
    imports = [
        foo::bar: fn(x: *const ::std::ffi::c_char) -> usize,
        foo::baz: fn(x: *const ::std::ffi::c_char) -> usize,
    ]
}
*/

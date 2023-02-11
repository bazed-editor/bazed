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

extern "C" fn init(_stew: *const crate::StewVft0, _data: *mut *mut std::ffi::c_void) -> bool {
    true
}
extern "C" fn main(_stew: *const crate::StewVft0, _data: *mut *mut std::ffi::c_void) {}

fn foo() {
    let m: PluginMetadata = metadata();
}
*/

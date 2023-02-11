/*
#![allow(unused)]

use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CStr, CString},
    sync::Arc,
};

use dashmap::DashMap;
use parking_lot::RwLock;
use uuid::Uuid;

unsafe fn convert_str(ptr: *const c_char) -> String {
    CStr::from_ptr(ptr).to_str().unwrap().to_string()
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, derive_more::Display)]
#[display(fmt = transparent)]
#[repr(transparent)]
#[no_mangle]
pub struct FnRef(Uuid);
impl FnRef {
    fn new() -> Self {
        FnRef(Uuid::new_v4())
    }
}

type Userdata = *mut c_void;

pub type StewCallback =
    unsafe extern "C" fn(args: *const c_void, userdata: *mut Userdata) -> *const c_void;

pub type StewPluginInitFn = extern "C" fn(
    stew: *mut Stew,
    register: StewRegisterFn,
    get_fn: StewGetFnFn,
    request: StewCallFn,
);

#[derive(Clone)]
pub struct Stew {
    functions: DashMap<FnRef, (StewCallback, Userdata)>,
    fn_path_to_ids: DashMap<String, FnRef>,
}

impl Stew {
    fn load_plugin(&mut self, plugin_init_fn: StewPluginInitFn) {
        plugin_init_fn(self, stew_register, stew_get_fn, stew_call);
    }
}

pub type StewRegisterFn = unsafe extern "C" fn(
    stew: *mut Stew,
    fn_path: *const c_char,
    cb: StewCallback,
    userdata: Userdata,
);

/// # Safety
/// lmao
unsafe extern "C" fn stew_register(
    stew: *mut Stew,
    fn_path: *const c_char,
    cb: StewCallback,
    userdata: Userdata,
) {
    let id = FnRef::new();
    let fn_path = convert_str(fn_path);
    (*stew).functions.insert(id, (cb, userdata));
    (*stew).fn_path_to_ids.insert(fn_path, id);
}

pub type StewGetFnFn = unsafe extern "C" fn(stew: *const Stew, fn_path: *const c_char) -> FnRef;

/// # Safety
/// lmao
unsafe extern "C" fn stew_get_fn(stew: *const Stew, fn_path: *const c_char) -> FnRef {
    let fn_path = convert_str(fn_path);
    let mut fn_ref = (*stew)
        .fn_path_to_ids
        .get(&fn_path)
        .unwrap_or_else(|| panic!("no function registered under {fn_path}"));
    let fn_ref = fn_ref.value();
    *fn_ref
}

pub type StewCallFn =
    unsafe extern "C" fn(stew: *const Stew, fn_ref: FnRef, arg: *const c_void) -> *const c_void;

/// # Safety
/// lmao
unsafe extern "C" fn stew_call(
    stew: *const Stew,
    fn_ref: FnRef,
    arg: *const c_void,
) -> *const c_void {
    let mut entry = (*stew)
        .functions
        .get_mut(&fn_ref)
        .unwrap_or_else(|| panic!("no function registered under {fn_ref}"));
    let (function, userdata) = entry.value_mut();
    function(arg, &mut *userdata)
}
*/

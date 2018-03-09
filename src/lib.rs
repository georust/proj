#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
extern "C" {
    #[link_name = "_pj_strerrno"]
    pub fn pj_strerrno(arg1: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char;
}

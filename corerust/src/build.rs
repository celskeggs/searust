extern crate bindgen;

use std::env;
use std::path::Path;

fn main() {
    let sysroot = env::var("SYSROOT").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let _ = bindgen::builder()
        .header(format!("{}/usr/include/sel4/sel4.h", sysroot))
        .use_core()
        .ignore_functions()
        .ignore_methods()
        .clang_arg(format!("-I{}/usr/include/", sysroot))
        .ctypes_prefix("::sel4::coretypes")
        .hide_type("seL4_CapRights")
        .hide_type("seL4_CapRights_t")
        .hide_type("seL4_PageFaultIpcRegisters")
        .constified_enum("seL4_Syscall_ID")
        .constified_enum("invocation_label")
        .constified_enum("arch_invocation_label")
        .constified_enum("seL4_LookupFailureType")
        .generate().unwrap()
        .write_to_file(Path::new(&out_dir).join("libsel4.rs"));
}

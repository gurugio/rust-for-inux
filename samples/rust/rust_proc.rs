// SPDX-License-Identifier: GPL-2.0

//! Rust LDD
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

// core is from Rust compiler, not from kernel
use core::ffi;
use core::marker::PhantomPinned;
use core::ptr;

use kernel::bindings;
use kernel::prelude::*;
use kernel::proc::{ProcOperations, RustProcRegistration};
use kernel::str::CString;

module! {
    type: RustProc,
    name: "rust_proc",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch04 proc_fs_basic",
    license: "GPL",
}

#[no_mangle]
pub unsafe extern "C" fn proc_show(
    _m: *mut bindings::seq_file,
    _v: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    pr_info!("proc_read is invoked\n");
    unsafe {
        bindings::seq_printf(
            _m,
            CString::try_from_fmt(fmt!("Hello, world!\n"))
                .unwrap()
                .as_char_ptr(),
        );
    }
    0
}

struct Token;

#[vtable]
impl ProcOperations for Token {
    type OpenData = ();
    type Data = ();

    fn proc_open(_inode: *mut bindings::inode, _file: *mut bindings::file) -> Result<i32> {
        pr_info!("proc_open is invoked\n");
        unsafe {
            let ret = bindings::single_open(_file, Some(proc_show), ptr::null_mut());
            pr_info!("single_open: ret={}\n", ret);
        }

        Ok(0)
    }

    fn proc_release(_inode: *mut bindings::inode, _file: *mut bindings::file) {
        pr_info!("proc_release is invoked\n");
        unsafe {
            let ret = bindings::single_release(_inode, _file);
            pr_info!("single_release: ret={}\n", ret);
        }
    }
}

struct RustProc {
    _reg: RustProcRegistration,
}

impl kernel::Module for RustProc {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("{} is loaded\n", name.to_str()?);
        pr_info!("proc_show={:#x}\n", proc_show as *mut ffi::c_void as usize);

        let reg = RustProcRegistration::new();
        reg.register::<Token>(ptr::null_mut())?;

        Ok(Self { _reg: reg })
    }
}

impl Drop for RustProc {
    fn drop(&mut self) {
        pr_info!("rust_proc is unloaded\n");
    }
}

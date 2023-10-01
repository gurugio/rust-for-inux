// SPDX-License-Identifier: GPL-2.0

//! Rust LDD
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

// core is from Rust compiler, not from kernel
use core::ptr;

use kernel::bindings;
use kernel::prelude::*;
use kernel::str::CString;

static SUB_DIR_NAME: &'static str = "rust_demo";
static PROC_FS_NAME: &'static str = "rust_proc_fs";
static _PROC_FS_NAME_MUL: &'static str = "rust_proc_fs_mul";

module! {
    type: RustProc,
    name: "rust_proc",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch04 proc_fs_basic",
    license: "GPL",
}

struct RustProc {}

impl RustProc {
    unsafe extern "C" fn proc_open(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> i32 {
        0 as i32
    }

    unsafe extern "C" fn proc_read(
        _file: *mut bindings::file,
        _buf: *mut core::ffi::c_char,
        _len: usize,
        _off: *mut bindings::loff_t,
    ) -> isize {
        0 as isize
    }
}

impl kernel::Module for RustProc {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("{} is loaded\n", name.to_str()?);

        unsafe {
            let dir_name = CString::try_from_fmt(fmt!("{}", SUB_DIR_NAME))?;
            let parent = bindings::proc_mkdir(dir_name.as_char_ptr(), ptr::null_mut());

            let proc_ops = bindings::proc_ops {
                proc_flags: 0,                // mandatory to prevent build error
                proc_get_unmapped_area: None, // mandatory to prevent build error
                proc_read_iter: None,         // mandatory to prevent build error
                proc_open: Some(Self::proc_open),
                proc_read: Some(Self::proc_read),
                proc_write: None,
                proc_lseek: None,
                proc_release: None,
                proc_poll: None,
                proc_ioctl: None,
                proc_mmap: None,
            };

            bindings::proc_create(
                CString::try_from_fmt(fmt!("{}", PROC_FS_NAME))?.as_char_ptr(),
                0o644,
                parent,
                &proc_ops,
            );
        }

        Ok(RustProc {})
    }
}

impl Drop for RustProc {
    fn drop(&mut self) {
        pr_info!("rust_proc is unloaded\n");
    }
}

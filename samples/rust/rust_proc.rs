// SPDX-License-Identifier: GPL-2.0

//! Rust LDD
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

// core is from Rust compiler, not from kernel
use core::marker::PhantomPinned;
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

struct RustProc {
    ops: bindings::proc_ops,
    parent: *mut bindings::proc_dir_entry,
    _entry: *mut bindings::proc_dir_entry,
    _pin: PhantomPinned,
}

impl RustProc {
    unsafe extern "C" fn proc_open(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> i32 {
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_info!("proc_open is invoked\n");
        pr_err!("proc_open is invoked\n");
        pr_err!("proc_open is invoked\n");
        pr_err!("proc_open is invoked\n");

        while true {
            pr_info!("proc_open is invoked\n");
        }

        unsafe {
            let ret = bindings::single_open(_file, Some(Self::proc_show), ptr::null_mut());
        }
        0 as i32
    }

    unsafe extern "C" fn proc_show(_m: *mut bindings::seq_file, _v: *mut core::ffi::c_void) -> i32 {
        pr_info!("proc_read is invoked\n");
        0 as i32
    }
}

impl kernel::Module for RustProc {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("{} is loaded\n", name.to_str()?);

        unsafe {
            let dir_name = CString::try_from_fmt(fmt!("{}", SUB_DIR_NAME))?;
            let parent = bindings::proc_mkdir(dir_name.as_char_ptr(), ptr::null_mut());

            let ret = Self {
                parent,
                ops: bindings::proc_ops {
                    proc_flags: 0,                // mandatory to prevent build error
                    proc_get_unmapped_area: None, // mandatory to prevent build error
                    proc_read_iter: None,         // mandatory to prevent build error
                    proc_open: Some(Self::proc_open),
                    proc_read: None,
                    proc_write: None,
                    proc_lseek: None,
                    proc_release: None,
                    proc_poll: None,
                    proc_ioctl: None,
                    proc_mmap: None,
                },
                _entry: ptr::null_mut(),
                _pin: PhantomPinned,
            };

            let entry_name = CString::try_from_fmt(fmt!("{}", PROC_FS_NAME))?;
            let entry: *mut bindings::proc_dir_entry =
                bindings::proc_create(entry_name.as_char_ptr(), 0o644, parent, &ret.ops);
            // How to check entry?
            if entry.is_null() {
                pr_info!("failed to create a proc entry\n");
            } else {
                pr_info!("succeeded to create a proc entry: {:p}\n", entry);
            }

            Ok(ret)
        }
    }
}

impl Drop for RustProc {
    fn drop(&mut self) {
        unsafe {
            let entry_name = CString::try_from_fmt(fmt!("{}", PROC_FS_NAME)).unwrap();
            bindings::remove_proc_entry(entry_name.as_char_ptr(), self.parent);

            let dir_name = CString::try_from_fmt(fmt!("{}", SUB_DIR_NAME)).unwrap();
            bindings::remove_proc_entry(dir_name.as_char_ptr(), ptr::null_mut());
        }
        pr_info!("rust_proc is unloaded\n");
    }
}

unsafe impl Sync for RustProc {}

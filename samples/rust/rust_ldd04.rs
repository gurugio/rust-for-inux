// SPDX-License-Identifier: GPL-2.0

//! Rust LDD
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

// core is from Rust compiler, not from kernel
use core::ffi;

use kernel::proc::{ProcOperations, RustProcRegistration};
use kernel::{bindings, fmt, prelude::*, str::CString};

module! {
    type: RustProc,
    name: "rust_proc",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch04 proc_fs_basic",
    license: "GPL",
}

/// TBD
#[no_mangle]
pub unsafe extern "C" fn proc_show(
    m: *mut bindings::seq_file,
    _v: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    pr_info!("proc_show is invoked\n");
    unsafe {
        let count: usize = (*m).private as *mut ffi::c_void as usize;
        pr_info!("priv={}", count);
        bindings::seq_printf(
            m,
            CString::try_from_fmt(fmt!("Hello World!\n"))
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
        let ret;
        unsafe {
            ret = bindings::single_open(_file, Some(proc_show), (*_inode).i_private);
            pr_info!("single_open: ret={}\n", ret);
        }

        Ok(ret)
    }

    fn proc_release(_inode: *mut bindings::inode, _file: *mut bindings::file) {
        pr_info!("proc_release is invoked\n");
        unsafe {
            let ret = bindings::single_release(_inode, _file);
            pr_info!("single_release: ret={}\n", ret);
        }
    }

    fn proc_read(
        _file: *mut bindings::file,
        _buf: *mut ffi::c_char,
        _size: usize,
        _ppos: *mut bindings::loff_t,
    ) -> Result<isize> {
        pr_info!("proc_read is invoked\n");
        let ret;
        unsafe {
            ret = bindings::seq_read(_file, _buf, _size, _ppos);
            pr_info!("seq_read: ret={}\n", ret);
        }
        Ok(ret)
    }

    fn proc_lseek(
        _file: *mut bindings::file,
        _offset: bindings::loff_t,
        _whence: core::ffi::c_int,
    ) -> Result<bindings::loff_t> {
        pr_info!("proc_lseek is invoked\n");
        let ret;
        unsafe {
            ret = bindings::seq_lseek(_file, _offset, _whence);
            pr_info!("seq_lseek: ret={}\n", ret);
        }
        Ok(ret)
    }
}

struct RustProc {
    _regs: Vec<RustProcRegistration>,
}

impl kernel::Module for RustProc {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_ldd04 is loaded\n");

        let dirname: CString = CString::try_from_fmt(fmt!("proc_demo")).unwrap();
        let filename: CString = CString::try_from_fmt(fmt!("proc_fs")).unwrap();
        let filename_mul = CString::try_from_fmt(fmt!("proc_fs_mul")).unwrap();

        let dir = RustProcRegistration::mkdir(&dirname, core::ptr::null_mut())?;

        let mut regs = Vec::new();

        let mut reg = RustProcRegistration::new(dir);
        reg.register::<Token>(&filename, None)?;
        regs.try_push(reg)?;

        let mut reg = RustProcRegistration::new(dir);
        reg.register::<Token>(&filename_mul, Some(3))?;
        regs.try_push(reg)?;

        Ok(Self { _regs: regs })
    }
}

impl Drop for RustProc {
    fn drop(&mut self) {
        pr_info!("rust_proc is unloaded\n");
    }
}

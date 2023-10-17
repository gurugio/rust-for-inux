// SPDX-License-Identifier: GPL-2.0

//! Rust LDD
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

// core is from Rust compiler, not from kernel
use core::ffi;

use core::ptr;
use kernel::proc::{ProcOperations, RustProcRegistration};
use kernel::{bindings, fmt, prelude::*, str::CString};

module! {
    type: RustProcFsIterator,
    name: "rust_proc_fs_iterator",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch05 proc_fs_iterator",
    license: "GPL",
}

const MESSAGES: [&'static str; 7] = [
    "Day 1: God creates the heavens and the earth.",
    "Day 2: God creates the sky.",
    "Day 3: God creates dry land and all plant life both large and small.",
    "Day 4: God creates all the stars and heavenly bodies.",
    "Day 5: God creates all life that lives in the water.",
    "Day 6: God creates all the creatures that live on dry land.",
    "Day 7: God rests.",
];

const PROC_SEQ_OPS: bindings::seq_operations = bindings::seq_operations {
    start: Some(proc_seq_start),
    next: Some(proc_seq_next),
    stop: Some(proc_seq_stop),
    show: Some(proc_seq_show),
};

/// TBD
pub unsafe extern "C" fn proc_seq_start(
    _s_file: *mut bindings::seq_file,
    pos: *mut bindings::loff_t,
) -> *mut core::ffi::c_void {
    pr_info!("proc_seq_start is invoked\n");

    let pos: usize = unsafe { *pos as usize };

    pr_info!("pos={}\n", pos);

    if pos >= MESSAGES.len() {
        return ptr::null_mut();
    }

    CString::try_from_fmt(fmt!("{}", MESSAGES[pos]))
        .unwrap()
        .as_char_ptr() as *mut core::ffi::c_void
}

/// TBD
pub unsafe extern "C" fn proc_seq_next(
    _s_file: *mut bindings::seq_file,
    _v: *mut core::ffi::c_void,
    pos: *mut bindings::loff_t,
) -> *mut core::ffi::c_void {
    pr_info!("proc_seq_next is invoked\n");

    unsafe {
        pr_info!("pos={}\n", *pos);

        (*pos) += 1;

        if (*pos) as usize >= MESSAGES.len() {
            return ptr::null_mut();
        }

        CString::try_from_fmt(fmt!("{}", MESSAGES[*pos as usize]))
            .unwrap()
            .as_char_ptr() as *mut core::ffi::c_void
    }
}

/// TBD
pub unsafe extern "C" fn proc_seq_stop(
    _s_file: *mut bindings::seq_file,
    _v: *mut core::ffi::c_void,
) {
    pr_info!("proc_seq_stop is invoked\n");
}

/// TBD
pub unsafe extern "C" fn proc_seq_show(
    s_file: *mut bindings::seq_file,
    _v: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    pr_info!("proc_seq_show is invoked\n");
    unsafe {
        bindings::seq_printf(
            s_file,
            CString::try_from_fmt(fmt!("seq_printf in proc_seq_show\n"))
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
            ret = bindings::seq_open(_file, &PROC_SEQ_OPS);
            pr_info!("seq_open: ret={}\n", ret);
        }

        Ok(ret)
    }

    fn proc_release(_inode: *mut bindings::inode, _file: *mut bindings::file) {
        pr_info!("proc_release is invoked\n");
        unsafe {
            let ret = bindings::seq_release(_inode, _file);
            pr_info!("seq_release: ret={}\n", ret);
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

struct RustProcFsIterator {
    _reg: RustProcRegistration,
}

impl kernel::Module for RustProcFsIterator {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_ldd05 is loaded\n");

        let filename: CString = CString::try_from_fmt(fmt!("proc_fs_iterator")).unwrap();
        let mut reg = RustProcRegistration::new(ptr::null_mut());

        reg.register::<Token>(&filename, None)?;

        Ok(Self { _reg: reg })
    }
}

impl Drop for RustProcFsIterator {
    fn drop(&mut self) {
        pr_info!("rust_proc is unloaded\n");
    }
}

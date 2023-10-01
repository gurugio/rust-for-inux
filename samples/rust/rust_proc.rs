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
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    str::CString,
    sync::{Arc, ArcBorrow, Mutex, UniqueArc},
};

module! {
    type: RustProc,
    name: "rust_proc",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch04 proc_fs_basic",
    license: "GPL",
}

struct RustProc {}

impl kernel::Module for RustProc {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("{} is loaded\n", name.to_str()?);

        unsafe {
            let parent_dir = CString::try_from_fmt(fmt!("rust_proc"))?;
            let proc_dir = bindings::proc_mkdir(parent_dir.as_char_ptr(), ptr::null_mut());
        }

        Ok(RustProc {})
    }
}

impl Drop for RustProc {
    fn drop(&mut self) {
        pr_info!("rust_proc is unloaded\n");
    }
}

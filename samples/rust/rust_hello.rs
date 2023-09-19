// SPDX-License-Identifier: GPL-2.0

//! Rust Hello world sample.
//! How to build only modules:
//! make LLVM=1 M=samples/rust

use kernel::prelude::*;
use kernel::task::Task;

module! {
    type: RustHello, // struct name below
    name: "rust_hello",
    author: "Rust for Linux Contributors",
    description: "Rust Hello world sample",
    license: "GPL",
}

struct RustHello {}

impl kernel::Module for RustHello {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello Rust!\n");

        pr_info!(
            r#"The process is "{}" (pid {})\n"#,
            Task::current().comm(),
            Task::current().group_leader().pid()
        );
        Ok(RustHello {})
    }
}

impl Drop for RustHello {
    fn drop(&mut self) {
        pr_info!("See you soon\n");
    }
}

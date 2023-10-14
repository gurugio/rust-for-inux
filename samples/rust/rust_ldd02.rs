// SPDX-License-Identifier: GPL-2.0

//! Rust minimal sample.
//! make LLVM=1 M=samples/rust/

use kernel::prelude::*;
use kernel::{task::Task, types::ARef};

module! {
    type: RustMinimal,
    name: "rust_minimal",
    author: "Rust for Linux Contributors",
    description: "Rust minimal sample",
    license: "GPL",
}

struct RustMinimal {
    numbers: Vec<i32>,
}

impl kernel::Module for RustMinimal {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal sample\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));
        let task: ARef<Task> = current!().into();
        pr_info!("pid={}\n", task.pid());
        pr_info!("comm={}\n", task.comm());

        let mut numbers = Vec::new();
        numbers.try_push(72)?;
        numbers.try_push(108)?;
        numbers.try_push(200)?;

        Ok(RustMinimal { numbers })
    }
}

impl Drop for RustMinimal {
    fn drop(&mut self) {
        pr_info!("My numbers are {:?}\n", self.numbers);
        pr_info!("Rust minimal sample (exit)\n");
    }
}

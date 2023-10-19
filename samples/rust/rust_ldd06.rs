// SPDX-License-Identifier: GPL-2.0

//! Rust LDD scull
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/eg_03_scull_basic
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust
use core::marker::PhantomPinned;
use kernel::prelude::*;
use kernel::{
    bindings,
    file::{self, File},
    fmt,
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev, pin_init,
    sync::{Arc, ArcBorrow},
    types::Opaque,
};

module! {
    type: RustCompletion,
    name: "rust_completion",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch06 scull",
    license: "GPL",
}

// internal info between file operations
#[pin_data]
struct CompletionDev {
    pub completion: bindings::completion,

    #[pin]
    _pin: PhantomPinned,
}

// TODO: impl CompletionDev::try_new
impl CompletionDev {
    fn try_new() -> Result<Arc<Self>> {
        pr_info!("completion_dev created\n");

        Ok(Arc::try_new(Self {
            _pin: PhantomPinned,
            completion: bindings::completion::default(),
        })?)
    }
}

unsafe impl Sync for CompletionDev {}
unsafe impl Send for CompletionDev {}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<CompletionDev>;
    type OpenData = Arc<CompletionDev>;

    fn open(shared: &Arc<CompletionDev>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open is invoked\n",);
        Ok(shared.clone())
    }

    //
    fn read(
        shared: ArcBorrow<'_, CompletionDev>,
        _: &File,
        data: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("read is invoked\n");

        Ok(0)
    }

    fn write(
        shared: ArcBorrow<'_, CompletionDev>,
        _: &File,
        data: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        pr_debug!("write is invoked\n");

        let len: usize = data.len();
        pr_info!("write: {} bytes\n", len);
        Ok(len)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release is invoked\n");
    }
}

struct RustCompletion {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustCompletion {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_ldd06 is loaded\n");

        let dev: Arc<CompletionDev> = CompletionDev::try_new()?;
        let reg = miscdev::Registration::new_pinned(fmt!("rust_ldd06"), dev)?;

        Ok(RustCompletion { _dev: reg })
    }
}

impl Drop for RustCompletion {
    fn drop(&mut self) {
        pr_info!("rust_ldd06 is unloaded\n");
    }
}

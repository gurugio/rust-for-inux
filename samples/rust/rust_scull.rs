// SPDX-License-Identifier: GPL-2.0

//! Rust LDD scull
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/eg_03_scull_basic
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust

use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Arc, ArcBorrow, Mutex},
    {chrdev, PAGE_SIZE},
};

module! {
    type: RustScull
,
    name: "rust_scull",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch03 scull",
    license: "GPL",
}

const SCULL_NR_DEVS: usize = 3;
static _SCULL_BLOCK_SIZE: usize = PAGE_SIZE;

struct _ScullBlock {
    offset: usize,
    data: Vec<u8>,
}

struct ScullDevInner {
    block_counter: usize,
}

// internal info between file operations
struct ScullDev {
    inner: Mutex<ScullDevInner>,
    // mutex
    // cdev -> Registration.inner.cdevs[]
    // list of ScullBlock
}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<ScullDev>;
    //type OpenData = Arc<ScullDev>;

    // called when open() is called
    fn open(_shared: &(), _file: &file::File) -> Result<Self::Data> {
        pr_info!("open is invoked\n");
        Ok(Self::Data::try_new(ScullDev {
            inner: unsafe { Mutex::new(ScullDevInner { block_counter: 0 }) },
        })?)
    }

    //
    fn read(
        shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        _data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read is invoked\n");
        {
            let inner = shared.inner.lock();
            pr_info!("block_counter={}", inner.block_counter);
        }
        Ok(0)
    }

    fn write(
        shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        _data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_debug!("write is invoked\n");
        {
            let mut inner = shared.inner.lock();
            inner.block_counter += 1;
            pr_info!("block_counter={}", inner.block_counter);
        }
        Ok(0)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release is invoked\n");
    }
}

struct RustScull {
    _dev: Pin<Box<chrdev::Registration<SCULL_NR_DEVS>>>,
}

impl kernel::Module for RustScull {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_scull is loaded\n");

        let mut chrdev_reg: Pin<Box<chrdev::Registration<SCULL_NR_DEVS>>> =
            chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        let _ = (0..SCULL_NR_DEVS)
            .map(|_| chrdev_reg.as_mut().register::<RustFile>().unwrap())
            .collect::<()>();

        Ok(RustScull { _dev: chrdev_reg })
    }
}

impl Drop for RustScull {
    fn drop(&mut self) {
        pr_info!("rust_scull is unloaded\n");

        // No need to call unregister_chrdev_region?
    }
}

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
    miscdev,
    sync::{Arc, ArcBorrow, Mutex, UniqueArc},
    PAGE_SIZE,
};

module! {
    type: RustScull,
    name: "rust_scull",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch03 scull",
    license: "GPL",
}

const SCULL_NR_DEVS: usize = 3;
static _SCULL_BLOCK_SIZE: usize = PAGE_SIZE;

struct ScullDevInner {
    counter: usize,
    offset: usize,
    data: Vec<u8>,
}

// internal info between file operations
struct ScullDev {
    minor: usize,
    inner: Mutex<ScullDevInner>,
}

impl ScullDev {
    fn try_new(num: usize) -> Result<Arc<Self>> {
        let mut dev = Pin::from(UniqueArc::try_new(Self {
            minor: num,
            inner: unsafe {
                Mutex::new(ScullDevInner {
                    counter: 0,
                    offset: 0,
                    data: Vec::new(),
                })
            },
        })?);

        let pinned = unsafe { dev.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        kernel::mutex_init!(pinned, "ScullDev::inner");

        Ok(dev.into())
    }
}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<ScullDev>;
    type OpenData = Arc<ScullDev>;

    fn open(shared: &Arc<ScullDev>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open is invoked\n");
        pr_info!("minor: {}\n", shared.minor);
        Ok(shared.clone())
    }

    //
    fn read(
        _shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        _data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read is invoked\n");
        Ok(0)
    }

    fn write(
        _shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        _data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_debug!("write is invoked\n");
        Ok(0)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release is invoked\n");
    }
}

struct RustScull {
    _dev: Vec<Pin<Box<miscdev::Registration<RustFile>>>>,
}

impl kernel::Module for RustScull {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("{} is loaded\n", name.to_str()?);

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        let mut devs = Vec::try_with_capacity(SCULL_NR_DEVS)?;
        for i in 0..SCULL_NR_DEVS {
            let dev = ScullDev::try_new(i)?;
            let reg = miscdev::Registration::new_pinned(fmt!("rust_scull{}", i), dev)?;
            devs.try_push(reg)?;
        }

        Ok(RustScull { _dev: devs })
    }
}

impl Drop for RustScull {
    fn drop(&mut self) {
        pr_info!("rust_scull is unloaded\n");

        // No need to call unregister_chrdev_region?
    }
}

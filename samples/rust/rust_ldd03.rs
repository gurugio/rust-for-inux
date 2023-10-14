// SPDX-License-Identifier: GPL-2.0

//! Rust LDD scull
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/eg_03_scull_basic
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust
use kernel::prelude::*;
use kernel::{
    file::{self, File},
    fmt,
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    str::CString,
    sync::{Arc, ArcBorrow, Mutex},
    {new_mutex, pin_init},
};

module! {
    type: RustScull,
    name: "rust_scull",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch03 scull",
    license: "GPL",
}

const SCULL_NR_DEVS: usize = 3;

// internal info between file operations
#[pin_data]
struct ScullDev {
    name: CString,
    number: usize,

    #[pin]
    contents: Mutex<Vec<u8>>,
}

impl ScullDev {
    fn try_new(name: CString, num: usize) -> Result<Arc<Self>> {
        let dev = Arc::pin_init(pin_init!(Self {
            name,
            number: num,
            contents <- new_mutex!(Vec::new()),
        }))?;

        pr_info!(
            "scull_dev created: name={} number={}\n",
            dev.name.to_str()?,
            dev.number
        );
        Ok(dev)
    }
}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<ScullDev>;
    type OpenData = Arc<ScullDev>;

    fn open(shared: &Arc<ScullDev>, _file: &file::File) -> Result<Self::Data> {
        pr_info!(
            "open is invoked: name={} number={}\n",
            shared.name.to_str()?,
            shared.number
        );
        Ok(shared.clone())
    }

    //
    fn read(
        shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        data: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("[{}] read is invoked\n", shared.name.to_str()?);
        let offset = offset as usize;

        let buffer = shared.contents.lock();
        if offset >= buffer.len() {
            return Ok(0);
        }

        let mut len = buffer.len();
        pr_info!("buffer.len() = {}\n", len);
        if offset + len > buffer.len() {
            len = buffer.len() - offset;
        }
        data.write_slice(&buffer[offset..offset + len])?;

        pr_info!("read: {} bytes\n", len);
        Ok(len)
    }

    fn write(
        shared: ArcBorrow<'_, ScullDev>,
        _: &File,
        data: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        pr_debug!("[{}] write is invoked\n", shared.name.to_str()?);

        //let offset: usize = offset.try_into().map_err(|_| Error::EINVAL)?;
        let offset = offset as usize;
        let len: usize = data.len();
        let new_len = len + offset;

        // TODO: fix after applying Mutex to ScullDev.contents
        let mut buffer = shared.contents.lock();
        if new_len > buffer.len() {
            buffer.try_resize(new_len, 0)?;
        }
        data.read_slice(&mut buffer[offset..new_len])?;

        pr_info!("write: {} bytes\n", len);
        Ok(len)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release is invoked\n");
    }
}

struct RustScull {
    _dev: Vec<Pin<Box<miscdev::Registration<RustFile>>>>,
}

impl kernel::Module for RustScull {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_ldd03 is loaded\n");

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        let mut devs = Vec::try_with_capacity(SCULL_NR_DEVS)?;
        for i in 0..SCULL_NR_DEVS {
            let dev: Arc<ScullDev> =
                ScullDev::try_new(CString::try_from_fmt(fmt!("rust_ldd03{}", i))?, i)?;
            let reg = miscdev::Registration::new_pinned(fmt!("rust_ldd03{}", i), dev)?;
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

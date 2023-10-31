// SPDX-License-Identifier: GPL-2.0

//! Rust LDD scull
//! reference: https://github.com/d0u9/Linux-Device-Driver/tree/master/eg_03_scull_basic
//!
//! How to build only modules:
//! make LLVM=1 M=samples/rust
//!
//! How to use:
//! / # insmod rust_ldd06.ko
//! / # mknod /dev/rust_ldd06 c 10 124
//! / # cat /dev/rust_ldd06 &
//! / # echo "hello" > /dev/rust_ldd06
use kernel::prelude::*;
use kernel::{
    bindings, c_str,
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
    description: "Rust LDD ch06",
    license: "GPL",
}

#[pin_data]
struct CompletionDev {
    #[pin]
    completion: Opaque<bindings::completion>,
}

impl CompletionDev {
    fn try_new() -> Result<Arc<Self>> {
        pr_info!("completion_dev created\n");

        let mut compl = bindings::completion::default();
        let compl_name = c_str!("completion_dev");
        let mut key: bindings::lock_class_key = bindings::lock_class_key::default();
        compl.done = 0;

        // IMPORTANT!
        // I used Opaque::new() to allocate an opaque object that only created the object in stack memory.
        // Opaque::ffi_init creates the object directly on heap memory.
        let dev = Arc::pin_init(pin_init!(Self {
            completion <-
                Opaque::ffi_init(|slot: *mut bindings::completion| {
                    // see init_completion function in include/linux/completion.h
                    unsafe {
                        (*slot).done = 0;
                        bindings::__init_swait_queue_head(&mut (*slot).wait, compl_name.as_char_ptr(), &mut key);
                    }
                })
        }))?;

        Ok(dev)
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
        _data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read is invoked\n");

        unsafe {
            bindings::wait_for_completion(Opaque::raw_get(&shared.completion));
        }

        pr_info!("wait is done\n");
        Ok(0)
    }

    fn write(
        shared: ArcBorrow<'_, CompletionDev>,
        _: &File,
        data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("write is invoked\n");

        unsafe {
            bindings::complete(Opaque::raw_get(&shared.completion));
        }

        // return non-zero value to avoid infinite re-try
        Ok(data.len())
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

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
    miscdev, new_mutex, pin_init,
    sync::{Arc, ArcBorrow, Mutex},
};

module! {
    type: RustCompletion,
    name: "rust_completion",
    author: "Rust for Linux Contributors",
    description: "Rust LDD ch06",
    license: "GPL",
}

// Fist I made CompletionDev { completion: bindings::completion }.
// But it didn't work because we cannot get mutable reference to CompletionDev in read/write functions.
// The argument of read/write functions is ArcBorrow<'_, CompletionDev>.
// So it's not allowed to get the mutable reference to CompletionDev.
// The only way to mutate through ArcBorrow is to use Mutex/RwLock/Atomic types.
// (see doc.rust-lang.org/std/sync/struct.Mutex.html)
// Finally I makde CompletionInner struct and put it into Mutex.
struct CompletionInner {
    completion: bindings::completion,
}

// internal info between file operations
#[pin_data]
struct CompletionDev {
    #[pin]
    inner: Mutex<CompletionInner>,
}

// TODO: impl CompletionDev::try_new
impl CompletionDev {
    fn try_new() -> Result<Arc<Self>> {
        pr_info!("completion_dev created\n");

        //
        // #define init_swait_queue_head(q)				\
        // do {							\
        //    static struct lock_class_key __key;		\
        //    __init_swait_queue_head((q), #q, &__key);	\
        //} while (0)
        let mut compl = bindings::completion::default();
        let compl_name = c_str!("completion_dev");
        let mut key: bindings::lock_class_key = bindings::lock_class_key::default();
        compl.done = 0;
        unsafe {
            bindings::__init_swait_queue_head(
                &mut compl.wait,
                compl_name.as_char_ptr() as *mut core::ffi::c_char,
                &mut key,
            );
        }

        let dev = Arc::pin_init(pin_init!(Self {
            inner <- new_mutex!(CompletionInner {
                completion: compl,
            }),
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

        let mut inner_guard = shared.inner.lock();
        unsafe {
            bindings::wait_for_completion(&mut inner_guard.completion);
        }

        Ok(0)
    }

    fn write(
        shared: ArcBorrow<'_, CompletionDev>,
        _: &File,
        data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("write is invoked\n");

        let mut inner_guard = shared.inner.lock();
        unsafe {
            bindings::complete(&mut inner_guard.completion);
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

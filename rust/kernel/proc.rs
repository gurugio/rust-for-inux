use crate::{
    bindings,
    cred::Credential,
    error::{code::*, from_kernel_result, Error, Result},
    fmt,
    io_buffer::{IoBufferReader, IoBufferWriter},
    iov_iter::IovIter,
    mm,
    str::CString,
    sync::CondVar,
    types::ForeignOwnable,
    user_ptr::{UserSlicePtr, UserSlicePtrReader, UserSlicePtrWriter},
    ARef, AlwaysRefCounted, {pr_info, pr_warn},
};
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomPinned;
use core::{cell::UnsafeCell, marker, mem, ptr};
use macros::vtable;

struct ProcOperationsVtable<T>(marker::PhantomData<T>);

impl<T: ProcOperations> ProcOperationsVtable<T> {
    unsafe extern "C" fn proc_open(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> core::ffi::c_int {
        from_kernel_result! {
            pr_info!("OperationsVtable::proc_open is invoked\n");
            let _ = unsafe {T::proc_open(_inode, _file)};
            Ok(0)
        }
    }

    unsafe extern "C" fn proc_release(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> core::ffi::c_int {
        from_kernel_result! {
        pr_info!("OperationVtable::proc_release is invoked\n");
        let _ = unsafe {T::proc_release(_inode, _file)};
        Ok(0)
        }
    }

    unsafe extern "C" fn proc_read(
        _file: *mut bindings::file,
        _buf: *mut core::ffi::c_char,
        _size: usize,
        _ppos: *mut bindings::loff_t,
    ) -> isize {
        from_kernel_result! {
            pr_info!("OperationVtable::read is invoked\n");
            let _ = unsafe {T::proc_read(_file, _buf, _size, _ppos)};
            Ok(0)
        }
    }

    const VTABLE: bindings::proc_ops = bindings::proc_ops {
        proc_flags: 0,                // mandatory to prevent build error
        proc_get_unmapped_area: None, // mandatory to prevent build error
        proc_read_iter: None,         // mandatory to prevent build error
        proc_open: Some(Self::proc_open),
        proc_read: Some(Self::proc_read),
        proc_write: None,
        proc_lseek: None,
        proc_release: Some(Self::proc_release),
        proc_poll: None,
        proc_ioctl: None,
        #[cfg(CONFIG_COMPAT)]
        proc_compat_ioctl: None,
        proc_mmap: None,
    };

    const unsafe fn build() -> &'static bindings::proc_ops {
        &Self::VTABLE
    }
}

static _SUB_DIR_NAME: &'static str = "rust_demo";
static PROC_FS_NAME: &'static str = "rust_proc_fs";
static _PROC_FS_NAME_MUL: &'static str = "rust_proc_fs_mul";

#[vtable]
pub trait ProcOperations {
    type OpenData: Sync = ();
    type Data: Send + Sync = ();
    fn proc_open(_inode: *mut bindings::inode, _file: *mut bindings::file) -> Result<i32>;
    fn proc_read(_file: *mut bindings::file, _buf: *mut core::ffi::c_char, _size: usize, _ppos: *mut bindings::loff_t) -> Result<i32>;
    fn proc_release(_inode: *mut bindings::inode, _file: *mut bindings::file) {}
}

pub struct RustProcRegistration {
    //ops: bindings::proc_ops,
    parent: *mut bindings::proc_dir_entry,
    _entry: *mut bindings::proc_dir_entry,
    _pin: PhantomPinned,
}

impl RustProcRegistration {
    pub fn new() -> Self {
        pr_info!("RustProcregistration::new is invoked\n");
        Self {
            parent: ptr::null_mut(),
            _entry: ptr::null_mut(),
            _pin: PhantomPinned,
        }
    }

    pub fn register<T: ProcOperations<OpenData = ()>>(
        &self,
        _parent: *mut bindings::proc_dir_entry,
    ) -> Result<()> {
        pr_info!("RustProcregistration::register is invoked\n");
        let entry_name = CString::try_from_fmt(fmt!("{}", PROC_FS_NAME))?;

        let entry: *mut bindings::proc_dir_entry = unsafe {
            bindings::proc_create(
                entry_name.as_char_ptr(),
                0o644,
                ptr::null_mut(), // parent
                ProcOperationsVtable::<T>::build(),
            )
        };
        // How to check entry?
        if entry.is_null() {
            pr_info!("failed to create a proc entry\n");
        }

        Ok(())
    }
}

unsafe impl Sync for RustProcRegistration {}
unsafe impl Send for RustProcRegistration {}

impl Drop for RustProcRegistration {
    fn drop(&mut self) {
        unsafe {
            let entry_name = CString::try_from_fmt(fmt!("{}", PROC_FS_NAME)).unwrap();
            bindings::remove_proc_entry(entry_name.as_char_ptr(), ptr::null_mut());
        }
        pr_info!("drop RustProcRegistration\n");
    }
}

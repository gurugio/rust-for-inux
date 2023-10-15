// SPDX-License-Identifier: GPL-2.0

//! Printing facilities.
//!
//! C header: [`include/linux/printk.h`](../../../../include/linux/printk.h)
//!
//! Reference: <https://www.kernel.org/doc/html/latest/core-api/printk-basics.html>

use crate::{
    bindings,
    error::{from_err_ptr, from_result, Result},
    str::CString,
};
use core::marker::PhantomPinned;
use core::{marker, ptr};
use macros::vtable;

struct ProcOperationsVtable<T>(marker::PhantomData<T>);

impl<T: ProcOperations> ProcOperationsVtable<T> {
    unsafe extern "C" fn proc_open(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> core::ffi::c_int {
        from_result(|| T::proc_open(_inode, _file))
    }

    unsafe extern "C" fn proc_release(
        _inode: *mut bindings::inode,
        _file: *mut bindings::file,
    ) -> core::ffi::c_int {
        from_result(|| {
            let _ = T::proc_release(_inode, _file);
            Ok(0)
        })
    }

    unsafe extern "C" fn proc_read(
        _file: *mut bindings::file,
        _buf: *mut core::ffi::c_char,
        _size: usize,
        _ppos: *mut bindings::loff_t,
    ) -> isize {
        from_result(|| T::proc_read(_file, _buf, _size, _ppos))
    }

    unsafe extern "C" fn proc_lseek(
        _file: *mut bindings::file,
        _offset: bindings::loff_t,
        _whence: core::ffi::c_int,
    ) -> bindings::loff_t {
        from_result(|| T::proc_lseek(_file, _offset, _whence))
    }

    const VTABLE: bindings::proc_ops = bindings::proc_ops {
        proc_flags: 0,
        proc_get_unmapped_area: None,
        proc_read_iter: None,
        proc_open: Some(Self::proc_open),
        proc_read: Some(Self::proc_read),
        proc_write: None,
        proc_lseek: Some(Self::proc_lseek),
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

/// Corresponds to the kernel's `struct proc_ops`.
///
/// You implement this trait whenever you would create a `struct proc_ops`.
///
/// Proc-entry may be used from multiple threads/processes concurrently, so your type must be
/// [`Sync`]. It must also be [`Send`] because [`ProcOperations::release`] will be called from the
/// thread that decrements that associated file's refcount to zero.
#[vtable]
pub trait ProcOperations {
    /// TBD
    type OpenData: Sync = ();

    /// TBD
    type Data: Send + Sync = ();

    /// TBD
    fn proc_open(_inode: *mut bindings::inode, _file: *mut bindings::file) -> Result<i32>;

    /// TBD
    fn proc_read(
        _file: *mut bindings::file,
        _buf: *mut core::ffi::c_char,
        _size: usize,
        _ppos: *mut bindings::loff_t,
    ) -> Result<isize>;

    /// TBD
    fn proc_release(_inode: *mut bindings::inode, _file: *mut bindings::file) {}

    /// TBD
    fn proc_lseek(
        _file: *mut bindings::file,
        _offset: bindings::loff_t,
        _whence: core::ffi::c_int,
    ) -> Result<bindings::loff_t>;
}

/// TBD
pub struct RustProcRegistration {
    dir: *mut bindings::proc_dir_entry,
    entry: *mut bindings::proc_dir_entry,
    _pin: PhantomPinned,
}

impl RustProcRegistration {
    /// TBD
    pub fn new(dir: *mut bindings::proc_dir_entry) -> Self {
        Self {
            dir,
            entry: ptr::null_mut(),
            _pin: PhantomPinned,
        }
    }

    /// TBD
    pub fn mkdir(
        name: &CString,
        parent: *mut bindings::proc_dir_entry,
    ) -> Result<*mut bindings::proc_dir_entry> {
        // TODO: setting parent generated panic!!!
        //unsafe { from_err_ptr(bindings::proc_mkdir(name.as_char_ptr(), parent)) }
        unsafe {
            from_err_ptr(bindings::proc_mkdir(
                name.as_char_ptr(),
                core::ptr::null_mut(),
            ))
        }
    }

    /// TBD
    pub fn register<T: ProcOperations<OpenData = ()>>(
        &mut self,
        filename: &CString,
        data: Option<usize>,
    ) -> Result<()> {
        let entry: *mut bindings::proc_dir_entry = if data.is_none() {
            unsafe {
                from_err_ptr(bindings::proc_create(
                    filename.as_char_ptr(),
                    0o644,
                    self.dir,
                    ProcOperationsVtable::<T>::build(),
                ))
            }?
        } else {
            unsafe {
                from_err_ptr(bindings::proc_create_data(
                    filename.as_char_ptr(),
                    0o644,
                    self.dir,
                    ProcOperationsVtable::<T>::build(),
                    data.unwrap() as *mut usize as *mut core::ffi::c_void,
                ))
            }?
        };

        self.entry = entry;
        Ok(())
    }
}

/// TBD
unsafe impl Sync for RustProcRegistration {}

impl Drop for RustProcRegistration {
    fn drop(&mut self) {
        unsafe {
            bindings::proc_remove(self.entry);
            bindings::proc_remove(self.dir)
        }
    }
}

use alloc::sync::Arc;
use core::{fmt, marker::PhantomData, ptr};

use crate::{
    ffi::{self},
    renderer::Renderer,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// Indicates that the Rive file is not supported by this runtime.
    UnsupportedVersion,
    /// Indicates that the there is a formatting problem in the file itself.
    Malformed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedVersion => f.write_str("unsupported Rive version"),
            Error::Malformed => f.write_str("file is incorrectly encoded"),
        }
    }
}

#[cfg(feature = "vello")]
impl std::error::Error for Error {}

#[derive(Debug)]
pub(crate) struct FileInner {
    pub raw_file: *const ffi::File,
    raw_factory: *mut ffi::Factory,
}

impl Drop for FileInner {
    fn drop(&mut self) {
        unsafe {
            ffi::rive_rs_file_release(self.raw_file, self.raw_factory);
        }
    }
}

unsafe impl Send for FileInner {}
unsafe impl Sync for FileInner {}

pub struct File<R: Renderer> {
    inner: Arc<FileInner>,
    _phantom: PhantomData<R>,
}

impl<R: Renderer> File<R> {
    #[inline]
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        let mut result = ffi::FileResult::Success;
        let mut raw_factory = ptr::null_mut();

        let raw_file = unsafe {
            ffi::rive_rs_file_new(
                data.as_ptr(),
                data.len(),
                ffi::RendererEntries::<R>::ENTRIES as *const ffi::RendererEntries<R> as *const (),
                &mut result as *mut ffi::FileResult,
                &mut raw_factory as *mut *mut ffi::Factory,
            )
        };

        match result {
            ffi::FileResult::Success => Ok(Self {
                inner: Arc::new(FileInner {
                    raw_file,
                    raw_factory,
                }),
                _phantom: PhantomData,
            }),
            ffi::FileResult::UnsupportedVersion => Err(Error::UnsupportedVersion),
            ffi::FileResult::Malformed => Err(Error::Malformed),
        }
    }

    pub(crate) fn as_inner(&self) -> &Arc<FileInner> {
        &self.inner
    }
}

impl<R: Renderer> fmt::Debug for File<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").finish()
    }
}

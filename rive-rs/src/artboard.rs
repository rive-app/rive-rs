use alloc::sync::Arc;
use core::{fmt, marker::PhantomData, ptr::NonNull};

use crate::{
    ffi,
    file::{File, FileInner},
    instantiate::{Handle, Instantiate},
    renderer::Renderer,
};

#[derive(Debug)]
pub(crate) struct ArtboardInner {
    _file: Arc<FileInner>,
    pub(crate) raw_artboard: *mut ffi::Artboard,
}

impl Drop for ArtboardInner {
    fn drop(&mut self) {
        unsafe {
            ffi::rive_rs_artboard_instance_release(self.raw_artboard);
        }
    }
}

unsafe impl Send for ArtboardInner {}
unsafe impl Sync for ArtboardInner {}

pub struct Artboard<R: Renderer> {
    inner: Arc<ArtboardInner>,
    _phantom: PhantomData<R>,
}

impl<R: Renderer> Artboard<R> {
    pub(crate) fn as_inner(&self) -> &Arc<ArtboardInner> {
        &self.inner
    }
}

impl<R: Renderer> Instantiate for Artboard<R> {
    type From = File<R>;

    #[inline]
    fn instantiate(file: &Self::From, handle: Handle) -> Option<Self> {
        let mut raw_artboard: Option<NonNull<ffi::Artboard>> = None;

        match handle {
            Handle::Default => unsafe {
                ffi::rive_rs_instantiate_artboard(file.as_inner().raw_file, None, &mut raw_artboard)
            },
            Handle::Index(ref index) => unsafe {
                ffi::rive_rs_instantiate_artboard(
                    file.as_inner().raw_file,
                    Some(index.into()),
                    &mut raw_artboard,
                )
            },
            Handle::Name(name) => unsafe {
                ffi::rive_rs_instantiate_artboard_by_name(
                    file.as_inner().raw_file,
                    name.as_ptr(),
                    name.len(),
                    &mut raw_artboard,
                )
            },
        }

        raw_artboard.map(|raw_artboard| Artboard {
            inner: Arc::new(ArtboardInner {
                _file: file.as_inner().clone(),
                raw_artboard: raw_artboard.as_ptr(),
            }),
            _phantom: PhantomData,
        })
    }
}

impl<R: Renderer> fmt::Debug for Artboard<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ardboard").finish()
    }
}

unsafe impl<R: Renderer> Send for Artboard<R> {}
unsafe impl<R: Renderer> Sync for Artboard<R> {}

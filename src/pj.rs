use crate::context::ThreadContext;
use crate::errno::Errno;
use std::{ffi, ptr, rc};
use thiserror::Error;

/// A safe wrapper around `proj_sys::PJ`.
pub(crate) struct Pj {
    pj: ptr::NonNull<proj_sys::PJ>,
    ctx: rc::Rc<ThreadContext>,
}

impl Pj {
    pub fn from_definition(ctx: rc::Rc<ThreadContext>, definition: &str) -> Result<Self, PjCreateError> {
        let definition =
            ffi::CString::new(definition).map_err(|e| PjCreateError::ArgumentNulError(e))?;
        let pj_ptr = unsafe { proj_sys::proj_create(ctx.as_ptr(), definition.as_ptr()) };
        Pj::from_pj_ptr(ctx, pj_ptr)
    }

    pub fn from_crs_to_crs(
        ctx: rc::Rc<ThreadContext>,
        source_crs: &str,
        target_crs: &str,
    ) -> Result<Self, PjCreateError> { // TODO: return Result<Self, Errno>
        let source_crs = ffi::CString::new(source_crs).unwrap(); // TODO
        let target_crs = ffi::CString::new(target_crs).unwrap(); // TODO
        let pj_ptr = unsafe {
            proj_sys::proj_create_crs_to_crs(
                ctx.as_ptr(),
                source_crs.as_ptr(),
                target_crs.as_ptr(),
                ptr::null_mut(),
            )
        };
        Pj::from_pj_ptr(ctx, pj_ptr)
    }

    pub fn from_pj_ptr(
        ctx: rc::Rc<ThreadContext>,
        pj_ptr: *mut proj_sys::PJconsts,
    ) -> Result<Self, PjCreateError> {
        ptr::NonNull::new(pj_ptr)
            .ok_or_else(|| match ctx.errno().message(&ctx) {
                Ok(s) => PjCreateError::ProjError(s),
                Err(err) => PjCreateError::ProjErrorMessageUtf8Error(err),
            })
            .map(|ptr| Pj{pj: ptr, ctx: ctx})
    }

    pub fn as_ptr(&self) -> *mut proj_sys::PJ {
        self.pj.as_ptr()
    }

    pub fn errno_reset(&mut self) -> Errno {
        Errno(unsafe { proj_sys::proj_errno_reset(self.as_ptr()) })
    }

    pub fn errno(&self) -> Errno {
        Errno(unsafe { proj_sys::proj_errno(self.as_ptr()) })
    }

    pub fn trans(
        &self,
        direction: proj_sys::PJ_DIRECTION,
        coord: proj_sys::PJ_COORD,
    ) -> proj_sys::PJ_COORD {
        unsafe { proj_sys::proj_trans(self.as_ptr(), direction, coord) }
    }
}

impl Drop for Pj {
    fn drop(&mut self) {
        unsafe {
            proj_sys::proj_destroy(self.as_ptr());
        }
    }
}

#[derive(Error, Debug)]
pub enum PjCreateError {
    #[error("A nul byte was found in the PROJ string definition or CRS argument: {0}")]
    ArgumentNulError(ffi::NulError),
    #[error("The underlying PROJ call failed: {0}")]
    ProjError(String),
    #[error("A UTF8 error occurred when constructing a PROJ error message")]
    ProjErrorMessageUtf8Error(std::str::Utf8Error),
}

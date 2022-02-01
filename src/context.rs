use std::ptr;
use crate::errno::Errno;

const PROJ_SYS_TRUE: i32 = 1;

/// PROJ thread context
pub struct ThreadContext(ptr::NonNull<proj_sys::PJ_CONTEXT>);

impl ThreadContext {
    pub fn new() -> Self {
        // Safety: `proj_context_clone` always returns a valid pointer to a thread context.
        unsafe {
            let ctx_ptr = proj_sys::proj_context_create();
            ThreadContext::from_raw(ctx_ptr)
        }
    }

    /// # Safety
    ///
    /// Must provide a non-null pointer to a PROJ thread context.
    unsafe fn from_raw(ctx_ptr: *mut proj_sys::PJ_CONTEXT) -> Self {
        debug_assert!(!ctx_ptr.is_null());
        ThreadContext(ptr::NonNull::new_unchecked(ctx_ptr))
    }

    pub fn as_ptr(&self) -> *mut proj_sys::PJ_CONTEXT {
        self.0.as_ptr()
    }

    pub fn errno(&self) -> Errno {
        Errno(unsafe { proj_sys::proj_context_errno(self.0.as_ptr()) })
    }

    pub fn set_enable_network(&mut self, enable: bool) -> bool {
        unsafe {
            proj_sys::proj_context_set_enable_network(self.0.as_ptr(), libc::c_int::from(enable))
                == PROJ_SYS_TRUE
        }
    }

    pub fn is_network_enabled(&self) -> bool {
        unsafe { proj_sys::proj_context_is_network_enabled(self.0.as_ptr()) == PROJ_SYS_TRUE }
    }
}

impl Clone for ThreadContext {
    fn clone(&self) -> Self {
        // Safety: `proj_context_clone` always returns a valid pointer to a thread context.
        unsafe {
            let ctx_ptr = proj_sys::proj_context_clone(self.0.as_ptr());
            ThreadContext::from_raw(ctx_ptr)
        }
    }
}

impl Default for ThreadContext {
    fn default() -> Self {
        ThreadContext::new()
    }
}

impl Drop for ThreadContext {
    fn drop(&mut self) {
        // Safety: The pointer being provided to `proj_context_destroy` will always be a valid
        // thread context, so long as the same `ThreadContext` doesn't get dropped twice.
        unsafe { proj_sys::proj_context_destroy(self.0.as_ptr()) };
    }
}

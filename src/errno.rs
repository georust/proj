use crate::context::ThreadContext;
use std::str;

pub(crate) struct Errno(pub libc::c_int);

impl Errno {
    pub fn message(&self, context: &ThreadContext) -> Result<String, str::Utf8Error> {
        unsafe {
            crate::_string(proj_sys::proj_context_errno_string(context.as_ptr(), self.0))
        }
    }
}

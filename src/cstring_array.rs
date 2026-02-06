use libc::c_char;
use std::ffi::{CString, NulError};
use std::ptr;

/// A null-terminated array of C strings for passing to PROJ functions.
///
/// PROJ functions that accept options expect a null-terminated array of
/// null-terminated strings (`char* const*`), typically in `KEY=VALUE` format.
///
/// Strings are converted to CStrings immediately when added via [`push`](Self::push),
/// and the null-terminated pointer array is maintained incrementally.
///
/// # Example
/// ```ignore
/// let mut c_strings = CStringArray::new();
/// c_strings.push("AUTHORITY=EPSG")?;
/// c_strings.push("ALLOW_BALLPARK=NO")?;
/// unsafe { proj_create_crs_to_crs_from_pj(ctx, src, tgt, area, c_strings.as_ptr()) };
/// ```
pub(crate) struct CStringArray {
    /// Owns the CString data.
    cstrings: Vec<CString>,
    /// Null-terminated pointer array, maintained incrementally as strings are added.
    ptrs: Vec<*const c_char>,
}

impl CStringArray {
    /// Creates a new empty `CStringArray`.
    pub fn new() -> Self {
        Self {
            cstrings: Vec::new(),
            ptrs: vec![ptr::null()],
        }
    }

    /// Adds a string to the array.
    ///
    /// Returns an error if the string contains an interior nul byte.
    pub fn push(&mut self, s: impl Into<String>) -> Result<(), NulError> {
        debug_assert_eq!(self.ptrs.last(), Some(&ptr::null()));
        debug_assert_eq!(self.ptrs.len(), self.cstrings.len() + 1);

        let cstring = CString::new(s.into())?;
        // Insert before the null terminator
        self.ptrs.insert(self.ptrs.len() - 1, cstring.as_ptr());
        self.cstrings.push(cstring);
        Ok(())
    }

    /// Returns a pointer to a null-terminated array of C string pointers,
    /// suitable for passing to PROJ C functions, or null if the list is empty.
    pub fn as_ptr(&self) -> *const *const c_char {
        debug_assert_eq!(self.ptrs.last(), Some(&ptr::null()));
        debug_assert_eq!(self.ptrs.len(), self.cstrings.len() + 1);
        if self.cstrings.is_empty() {
            // return null ptr, rather than a ptr to an empty list
            //
            // Historically we were not consistent about this. It's likely that proj handles an
            // empty list the same as NULL, but it seems better to consistently return NULL since
            // it's explicitly documented.
            ptr::null()
        } else {
            self.ptrs.as_ptr()
        }
    }
}

impl Default for CStringArray {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let arr = CStringArray::new();
        let ptr = arr.as_ptr();
        assert!(ptr.is_null());
    }

    #[test]
    fn test_single_string() {
        let mut arr = CStringArray::new();
        arr.push("KEY=VALUE").unwrap();
        let ptr = arr.as_ptr();
        unsafe {
            assert!(!(*ptr).is_null());
            assert!((*ptr.add(1)).is_null());
        }
    }

    #[test]
    fn test_nul_error() {
        let mut arr = CStringArray::new();
        assert!(arr.push("invalid\0string").is_err());
    }
}

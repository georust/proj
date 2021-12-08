/// A module for native grid network functionality, so we don't have to depend on libcurl
/// The crate-public functions are facades – they're designed for interaction with libproj –
/// delegating actual functionality to non-public versions, prefixed by an underscore.
///
/// **Note**: `error_string_max_size` is set to 128 by libproj.
// TODO: build some length checks for the errors that are stuffed into it
// This functionality based on https://github.com/OSGeo/PROJ/blob/master/src/networkfilemanager.cpp#L1675
use proj_sys::{proj_context_set_network_callbacks, PJ_CONTEXT, PROJ_NETWORK_HANDLE};

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::Method;
use std::ffi::CString;
use std::os::raw::c_ulonglong;
use std::ptr::{self, NonNull};

use crate::proj::{ProjError, _string};
use libc::c_char;
use libc::c_void;
use std::boxed::Box;
use std::{thread, time};

const CLIENT: &str = concat!("proj-rs/", env!("CARGO_PKG_VERSION"));
const MAX_RETRIES: u8 = 8;
// S3 sometimes sends these in place of actual client errors, so retry instead of erroring
const RETRY_CODES: [u16; 4] = [429, 500, 502, 504];

/// This struct is cast to `c_void`, then to `PROJ_NETWORK_HANDLE` so it can be passed around
struct HandleData {
    url: String,
    headers: reqwest::header::HeaderMap,
    // this raw pointer is handed out to libproj but never returned,
    // so a copy of the pointer (raw pointers are Copy) is stored here.
    // Note to future self: are you 100% sure that the pointer is never read again
    // after network_close returns?
    hptr: Option<NonNull<c_char>>,
}

impl HandleData {
    fn new(
        url: String,
        headers: reqwest::header::HeaderMap,
        hptr: Option<NonNull<c_char>>,
    ) -> Self {
        Self { url, headers, hptr }
    }
}

impl Drop for HandleData {
    // whenever HandleData is dropped we check whether it has a pointer,
    // dereferencing it if need be so the resource is freed
    fn drop(&mut self) {
        if let Some(header) = self.hptr {
            let _ = unsafe { CString::from_raw(header.as_ptr() as *mut i8) };
        }
    }
}

/// Return an exponential wait time based on the number of retries
///
/// Example: a value of 8 allows up to 6400 ms of retry delay, for a cumulative total of 25500 ms
fn get_wait_time_exp(retrycount: i32) -> u64 {
    if retrycount == 0 {
        return 0;
    }
    (retrycount as u64).pow(2) * 100u64
}

/// Process CDN response: handle retries in case of server error, or early return for client errors
fn error_handler<'a>(res: &'a mut Response, rb: RequestBuilder) -> Result<&'a Response, ProjError> {
    let mut status = res.status().as_u16();
    let mut retries = 0;
    // Check whether something went wrong on the server, or if it's an S3 retry code
    if res.status().is_server_error() || RETRY_CODES.contains(&status) {
        // Start retrying: up to MAX_RETRIES
        while (res.status().is_server_error() || RETRY_CODES.contains(&status))
            && retries <= MAX_RETRIES
        {
            retries += 1;
            let wait = time::Duration::from_millis(get_wait_time_exp(retries as i32));
            thread::sleep(wait);
            let retry = rb.try_clone().ok_or(ProjError::RequestCloneError)?;
            *res = retry.send()?;
            status = res.status().as_u16();
        }
    // Not a timeout or known S3 retry code: bail out
    } else if res.status().is_client_error() {
        return Err(ProjError::DownloadError(
            res.status().as_str().to_string(),
            res.url().to_string(),
            retries,
        ));
    }
    // Retries have been exhausted OR
    // The loop ended prematurely due to a different error
    if !res.status().is_success() {
        return Err(ProjError::DownloadError(
            res.status().as_str().to_string(),
            res.url().to_string(),
            retries,
        ));
    }
    Ok(res)
}

/// Network callback: open
///
/// Should try to read the `size_to_read` first bytes at the specified offset of the file given by
/// URL url, and write them to `buffer`. `out_size_read` should be updated with the actual amount
/// of bytes read (== `size_to_read` if the file is larger than `size_to_read`). During this read,
/// the implementation should make sure to store the HTTP headers from the server response to be
/// able to respond to `proj_network_get_header_value_cbk_type` callback.
/// `error_string_max_size` should be the maximum size that can be written into the `out_error_string`
/// buffer (including terminating nul character).
///
/// Note that this function is a facade for _network_open
pub(crate) unsafe extern "C" fn network_open(
    pc: *mut PJ_CONTEXT,
    url: *const c_char,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    out_size_read: *mut usize,
    error_string_max_size: usize,
    out_error_string: *mut c_char,
    ud: *mut c_void,
) -> *mut PROJ_NETWORK_HANDLE {
    match _network_open(
        pc,
        url,
        offset,
        size_to_read,
        buffer,
        out_size_read,
        error_string_max_size,
        out_error_string,
        ud,
    ) {
        Ok(res) => res,
        Err(e) => {
            let err_string = e.to_string();
            out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), err_string.len());
            out_error_string.add(err_string.len()).write(0);
            ptr::null_mut() as *mut PROJ_NETWORK_HANDLE
        }
    }
}

/// Where the ACTUAL work happens, taking advantage of Rust error-handling etc
unsafe fn _network_open(
    _: *mut PJ_CONTEXT,
    url: *const c_char,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    out_size_read: *mut usize,
    _: usize,
    out_error_string: *mut c_char,
    _: *mut c_void,
) -> Result<*mut PROJ_NETWORK_HANDLE, ProjError> {
    let url = _string(url)?;
    // - 1 is used because the HTTP convention is to use inclusive start and end offsets
    let end = offset as usize + size_to_read - 1;
    // RANGE header definition is "bytes=x-y"
    let hvalue = format!("bytes={}-{}", offset, end);
    // Create a new client that can be reused for subsequent queries
    let clt = Client::builder().build()?;
    let req = clt.request(Method::GET, &url);
    // this performs the initial byte read, presumably as an error check
    let initial = req.try_clone().ok_or(ProjError::RequestCloneError)?;
    let with_headers = initial.header("Range", &hvalue).header("Client", CLIENT);
    let mut res = with_headers.send()?;
    let in_case_of_error = req
        .try_clone()
        .ok_or(ProjError::RequestCloneError)?
        .header("Range", &hvalue);
    // hand the response off to the error-handler, continue on success
    error_handler(&mut res, in_case_of_error)?;
    // Write the initial read length value into the pointer
    let contentlength = res.content_length().ok_or(ProjError::ContentLength)? as usize;
    out_size_read.write(contentlength);
    let headers = res.headers().clone();
    // Copy the downloaded bytes into the buffer so it can be passed around
    res.bytes()?
        .as_ptr()
        .copy_to_nonoverlapping(buffer as *mut u8, contentlength.min(size_to_read));
    let hd = HandleData::new(url, headers, None);
    // heap-allocate the struct and cast it to a void pointer so it can be passed around to PROJ
    let hd_boxed = Box::new(hd);
    let void: *mut c_void = Box::into_raw(hd_boxed) as *mut c_void;
    let opaque: *mut PROJ_NETWORK_HANDLE = void as *mut PROJ_NETWORK_HANDLE;
    // If everything's OK, set the error string to empty
    let err_string = "";
    out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), err_string.len());
    out_error_string.add(err_string.len()).write(0);
    Ok(opaque)
}

/// Network callback: close connection and drop handle data (client and headers)
pub(crate) unsafe extern "C" fn network_close(
    _: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    _: *mut c_void,
) {
    // Because we created the raw pointer from a Box, we have to re-constitute the Box
    // This is the exact reverse order seen in _network_open
    let void = handle as *mut c_void as *mut HandleData;
    let _: Box<HandleData> = Box::from_raw(void);
}

/// Network callback: get header value
///
/// Note that this function is a facade for _network_get_header_value
pub(crate) unsafe extern "C" fn network_get_header_value(
    pc: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    header_name: *const c_char,
    ud: *mut c_void,
) -> *const c_char {
    let hd = &mut *(handle as *const c_void as *mut HandleData);
    match _network_get_header_value(pc, handle, header_name, ud) {
        Ok(res) => res,
        Err(_) => {
            // an empty value will cause an error upstream in libproj, which is the intention
            let hvalue = "";
            // unwrapping an empty str is fine
            let cstr = CString::new(hvalue).unwrap();
            let err = cstr.into_raw();
            hd.hptr = Some(NonNull::new(err).expect("Failed to create non-Null pointer"));
            err
        }
    }
}

/// Network callback: get header value
unsafe fn _network_get_header_value(
    _: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    header_name: *const c_char,
    _: *mut c_void,
) -> Result<*const c_char, ProjError> {
    let lookup = _string(header_name)?.to_lowercase();
    let hd = &mut *(handle as *mut c_void as *mut HandleData);
    let hvalue = hd
        .headers
        .get(&lookup)
        .ok_or_else(|| ProjError::HeaderError(lookup.to_string()))?
        .to_str()?;
    let cstr = CString::new(hvalue).unwrap();
    let header = cstr.into_raw();
    // Raw pointers are Copy: the pointer returned by this function is never returned by libproj so
    // in order to avoid a memory leak the pointer is copied and stored in the HandleData struct,
    // which is dropped when close_network returns. As part of that drop, the pointer in hptr is returned to Rust
    hd.hptr = Some(
        NonNull::new(header).expect("Failed to create non-Null pointer when building header value"),
    );
    Ok(header)
}

/// Network: read range
///
/// Read size_to_read bytes from handle, starting at `offset`, into `buffer`. During this read,
/// the implementation should make sure to store the HTTP headers from the server response to be
/// able to respond to `proj_network_get_header_value_cbk_type` callback.
/// `error_string_max_size` should be the maximum size that can be written into the
/// `out_error_string` buffer (including terminating nul character).
///
/// Return value should be the actual number of bytes read, 0 in case of error.
///
/// Note that this function is a facade for _network_read_range
pub(crate) unsafe extern "C" fn network_read_range(
    pc: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    error_string_max_size: usize,
    out_error_string: *mut c_char,
    ud: *mut c_void,
) -> usize {
    match _network_read_range(
        pc,
        handle,
        offset,
        size_to_read,
        buffer,
        error_string_max_size,
        out_error_string,
        ud,
    ) {
        Ok(res) => res,
        Err(e) => {
            // The assumption here is that if 0 is returned, whatever error is in out_error_string is displayed by libproj
            // since this isn't a conversion using CString, nul chars must be manually stripped
            let err_string = e.to_string().replace("0", "nought");
            out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), err_string.len());
            out_error_string.add(err_string.len()).write(0);
            0usize
        }
    }
}

/// Where the ACTUAL work happens
fn _network_read_range(
    _: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    _: usize,
    out_error_string: *mut c_char,
    _: *mut c_void,
) -> Result<usize, ProjError> {
    // - 1 is used because the HTTP convention is to use inclusive start and end offsets
    let end = offset as usize + size_to_read - 1;
    let hvalue = format!("bytes={}-{}", offset, end);
    let hd = unsafe { &mut *(handle as *const c_void as *mut HandleData) };
    let clt = Client::builder().build()?;
    let initial = clt.request(Method::GET, &hd.url);
    let in_case_of_error = initial
        .try_clone()
        .ok_or(ProjError::RequestCloneError)?
        .header("Range", &hvalue)
        .header("Client", CLIENT);
    let req = in_case_of_error
        .try_clone()
        .ok_or(ProjError::RequestCloneError)?;
    let mut res = req.send()?;
    // hand the response and retry instance off to the error-handler, continue on success
    error_handler(&mut res, in_case_of_error)?;
    let headers = res.headers().clone();
    let contentlength = res.content_length().ok_or(ProjError::ContentLength)? as usize;
    // Copy the downloaded bytes into the buffer so it can be passed around
    unsafe {
        res.bytes()?
            .as_ptr()
            .copy_to_nonoverlapping(buffer as *mut u8, contentlength.min(size_to_read));
    }
    let err_string = "";
    unsafe {
        out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), err_string.len());
        out_error_string.add(err_string.len()).write(0);
    }
    hd.headers = headers;
    Ok(contentlength)
}

/// Set up and initialise the grid download callback functions for all subsequent PROJ contexts
pub(crate) fn set_network_callbacks(ctx: *mut PJ_CONTEXT) -> i32 {
    let ud: *mut c_void = ptr::null_mut();
    unsafe {
        proj_context_set_network_callbacks(
            ctx,
            Some(network_open),
            Some(network_close),
            Some(network_get_header_value),
            Some(network_read_range),
            ud,
        )
    }
}

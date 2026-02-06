#![deny(
    clippy::cast_slice_from_raw_parts,
    clippy::cast_slice_different_sizes,
    invalid_null_arguments,
    clippy::ptr_as_ptr,
    clippy::transmute_ptr_to_ref
)]
/// A module for native grid network functionality, so we don't have to depend on libcurl.
/// The crate-public functions are callbacks designed for interaction with libproj,
/// delegating actual functionality to `NetworkClient`.
///
/// **Note**: `error_string_max_size` is set to 128 by libproj.
// TODO: build some length checks for the errors that are stuffed into it
// This functionality based on https://github.com/OSGeo/PROJ/blob/master/src/networkfilemanager.cpp#L1675
use proj_sys::{PJ_CONTEXT, PROJ_NETWORK_HANDLE, proj_context_set_network_callbacks};

use std::collections::HashMap;
use std::ffi::CString;
use std::io::Read;
use std::ops::Range;
use std::os::raw::c_ulonglong;
use std::ptr;
use ureq::Agent;

use crate::proj::{_string, ProjError};
use libc::c_char;
use libc::c_void;
use std::boxed::Box;
use std::{thread, time};

const CLIENT: &str = concat!("proj-rs/", env!("CARGO_PKG_VERSION"));
const MAX_RETRIES: u8 = 8;
// S3 sometimes sends these in place of actual client errors, so retry instead of erroring
const RETRY_CODES: [u16; 4] = [429, 500, 502, 504];
const SUCCESS_ERROR_CODES: Range<u16> = 200..300;
const CLIENT_ERROR_CODES: Range<u16> = 400..500;
const SERVER_ERROR_CODES: Range<u16> = 500..600;

/// HTTP client for a single resource URL, persisted across requests so that the underlying
/// connection can be reused. Cast to `PROJ_NETWORK_HANDLE` for passing through libproj callbacks.
struct NetworkClient {
    agent: Agent,
    url: String,
    /// Response headers from the most recent request, keyed by lowercase header name.
    /// Values are stored as `CString` for returning a stable pointer to libproj via
    /// `network_get_header_value`.
    most_recent_response_headers: HashMap<String, CString>,
}

impl NetworkClient {
    /// Create a new client for `url` and perform the initial read at `offset`.
    /// The number of bytes read is written to `out_size_read`.
    fn open(
        url: String,
        offset: u64,
        buffer: &mut [u8],
        out_size_read: &mut usize,
    ) -> Result<Self, ProjError> {
        let mut client = Self::new(url);
        *out_size_read = client.read(offset, buffer)?;
        Ok(client)
    }

    /// Create a new client for `url` with a fresh HTTP agent and empty headers.
    fn new(url: String) -> Self {
        let agent = Agent::new_with_defaults();
        Self {
            agent,
            url,
            most_recent_response_headers: HashMap::new(),
        }
    }

    /// Perform an HTTP range request starting at `offset` for `output_buffer.len()` bytes.
    /// The response body is written into `output_buffer` and the response headers are
    /// cached in `most_recent_response_headers`. Returns the number of bytes read.
    fn read(&mut self, offset: u64, output_buffer: &mut [u8]) -> Result<usize, ProjError> {
        // - 1 is used because the HTTP convention is to use inclusive start and end offsets
        let end = offset as usize + output_buffer.len() - 1;
        let agent = self.agent.clone();
        let url = self.url.clone();
        let mut response_ok = request_with_retries(&self.url, move || {
            let response = agent
                .get(url.clone())
                .header("Range", format!("bytes={offset}-{end}"))
                .header("Client", CLIENT)
                .call()?;
            Ok(response)
        })?;
        debug_assert!(response_ok.status().is_success());

        self.most_recent_response_headers = response_ok
            .headers()
            .iter()
            .filter_map(|(h, v)| {
                let header_name = h.to_string().to_lowercase();
                let header_value = v.to_str().ok()?;
                let header_value_cstring = CString::new(header_value).ok()?;
                Some((header_name, header_value_cstring))
            })
            .collect();

        let content_length = response_ok
            .headers()
            .get("Content-Length")
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or(ProjError::ContentLength)?;

        let len_to_write = content_length.min(output_buffer.len());
        let resized_output_buffer = &mut output_buffer[..len_to_write];
        // Note: this will (rightfully) error if the body reader is longer than the declared Content-Length
        response_ok
            .body_mut()
            .as_reader()
            .read_exact(resized_output_buffer)?;
        Ok(len_to_write)
    }
}

/// Return a quadratically-increasing wait time based on the number of retries
///
/// Example: a value of 8 allows up to 6400 ms of retry delay, for a cumulative total of 25500 ms
fn get_wait_time(retrycount: u8) -> time::Duration {
    let millis = if retrycount == 0 {
        0
    } else {
        (retrycount as u64).pow(2) * 100u64
    };
    time::Duration::from_millis(millis)
}

fn request_with_retries(
    url: &str,
    mut make_request: impl FnMut() -> Result<http::Response<ureq::Body>, ProjError>,
) -> Result<http::Response<ureq::Body>, ProjError> {
    let mut retries = 0;
    let mut res = make_request()?;
    // Check whether something went wrong on the server, or if it's an S3 retry code
    if SERVER_ERROR_CODES.contains(&res.status().as_u16())
        || RETRY_CODES.contains(&res.status().as_u16())
    {
        // Start retrying: up to MAX_RETRIES
        while (SERVER_ERROR_CODES.contains(&res.status().as_u16())
            || RETRY_CODES.contains(&res.status().as_u16()))
            && retries <= MAX_RETRIES
        {
            retries += 1;
            let wait = get_wait_time(retries);
            thread::sleep(wait);
            res = make_request()?;
        }
    // Not a timeout or known S3 retry code: bail out
    } else if CLIENT_ERROR_CODES.contains(&res.status().as_u16()) {
        return Err(ProjError::DownloadError(
            res.status().to_string(),
            url.to_string(),
            retries,
        ));
    }
    // Retries have been exhausted OR
    // The loop ended prematurely due to a different error
    if !SUCCESS_ERROR_CODES.contains(&res.status().as_u16()) {
        return Err(ProjError::DownloadError(
            res.status().to_string(),
            url.to_string(),
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
pub(crate) unsafe extern "C" fn network_open(
    _pc: *mut PJ_CONTEXT,
    url: *const c_char,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    out_size_read: *mut usize,
    error_string_max_size: usize,
    out_error_string: *mut c_char,
    _user_data: *mut c_void,
) -> *mut PROJ_NETWORK_HANDLE {
    // Given a start and a length, we can create a rust slice
    let output_buffer =
        unsafe { std::slice::from_raw_parts_mut(buffer.cast::<u8>(), size_to_read) };

    match unsafe { _string(url) }
        .map_err(ProjError::from)
        .and_then(|url| {
            NetworkClient::open(url, offset, output_buffer, unsafe { &mut *out_size_read })
        }) {
        Ok(network_client) => {
            // clear out any error message when successful
            unsafe {
                *out_error_string = 0;
            }
            Box::into_raw(Box::new(network_client)).cast::<PROJ_NETWORK_HANDLE>()
        }
        Err(e) => {
            let err_string = e.to_string();
            let len = err_string
                .len()
                .min(error_string_max_size.saturating_sub(1));
            unsafe {
                out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), len);
                out_error_string.add(len).write(0);
            }
            ptr::null_mut()
        }
    }
}

/// Network callback: close connection and drop handle data (client and headers)
pub(crate) unsafe extern "C" fn network_close(
    _: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    _: *mut c_void,
) {
    // Because we created the raw pointer from a Box, we have to re-constitute the Box
    // so that it can be dropped.
    // This is the exact reverse order seen in network_open
    let network_client = handle.cast::<NetworkClient>();
    let _ = unsafe { Box::from_raw(network_client) };
}

/// Network callback: get header value
pub(crate) unsafe extern "C" fn network_get_header_value(
    _pc: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    header_name: *const c_char,
    _user_data: *mut c_void,
) -> *const c_char {
    let network_client = unsafe { &mut *handle.cast::<NetworkClient>() };
    let Ok(header_name) = (unsafe { _string(header_name) }) else {
        debug_assert!(false, "bad header name passed to network_get_header_value");
        return ptr::null();
    };
    let header_name = header_name.to_lowercase();

    network_client
        .most_recent_response_headers
        .get(&header_name)
        .map(|cstring| cstring.as_ptr().cast())
        .unwrap_or(ptr::null())
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
pub(crate) unsafe extern "C" fn network_read_range(
    _pc: *mut PJ_CONTEXT,
    handle: *mut PROJ_NETWORK_HANDLE,
    offset: c_ulonglong,
    size_to_read: usize,
    buffer: *mut c_void,
    error_string_max_size: usize,
    out_error_string: *mut c_char,
    _user_data: *mut c_void,
) -> usize {
    let network_client = unsafe { &mut *handle.cast::<NetworkClient>() };
    // Given a start and a length, we can create a rust slice
    let output_buffer =
        unsafe { std::slice::from_raw_parts_mut(buffer.cast::<u8>(), size_to_read) };
    match network_client.read(offset, output_buffer) {
        Ok(read) => {
            unsafe {
                *out_error_string = 0;
            }
            read
        }
        Err(e) => {
            let err_string = e.to_string();
            let len = err_string
                .len()
                .min(error_string_max_size.saturating_sub(1));
            unsafe {
                out_error_string.copy_from_nonoverlapping(err_string.as_ptr().cast(), len);
                out_error_string.add(len).write(0);
            }
            0
        }
    }
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

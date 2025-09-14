use napi::bindgen_prelude::{Buffer, Either3};
use napi::{Either, Error, Result, Status};
use napi_derive::napi;
use std::cell::UnsafeCell;
use std::os::raw::{c_char, c_int, c_long, c_void};

use crate::api::curl_easy_error;
use crate::loader::CurlSlistNode;
use crate::log_info;
use crate::utils::get_ptr_address;
use crate::{
  constants::{CurlInfo, CurlOpt},
  loader::{napi_load_library, CurlFunctions, CurlHandle, CurlSlist},
};

// Simple memory write callback
extern "C" fn write_data(
  ptr: *mut c_char,
  size: usize,
  nmemb: usize,
  stream: *mut c_void,
) -> usize {
  let real_size = size * nmemb;
  if !ptr.is_null() && !stream.is_null() && real_size > 0 {
    let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, real_size) };
    let buffer = unsafe { &mut *(stream as *mut Vec<u8>) };
    buffer.extend_from_slice(data);
  }
  real_size
}

#[napi]
pub struct Curl {
  pub closed: bool,
  handle: CurlHandle,
  lib: &'static CurlFunctions,
  header_buffer: UnsafeCell<Vec<u8>>,
  content_buffer: UnsafeCell<Vec<u8>>,
  req_header: UnsafeCell<Option<CurlSlist>>,
  req_body: UnsafeCell<Vec<u8>>,
}

// UnsafeCell requires manual implementation of Send and Sync
unsafe impl Send for Curl {}
unsafe impl Sync for Curl {}

#[napi]
impl Curl {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    unsafe {
      let lib = napi_load_library()?;

      let handle = (lib.easy_init)();
      if handle.is_null() {
        return Err(Error::new(
          Status::GenericFailure,
          "Failed to initialize curl handle",
        ));
      }

      let curl = Curl {
        closed: false,
        lib,
        handle,
        header_buffer: UnsafeCell::new(Vec::new()),
        content_buffer: UnsafeCell::new(Vec::new()),
        req_header: UnsafeCell::new(None), // Initialize headers list
        req_body: UnsafeCell::new(Vec::new()),
      };

      Ok(curl)
    }
  }

  /// Initialize data callbacks
  #[napi]
  pub fn init(&self) {
    log_info!("Curl", "Initializing curl data callbacks");
    unsafe {
      (*self.header_buffer.get()).clear();
      (*self.content_buffer.get()).clear();
      // Set write function
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::WriteFunction as c_int,
        write_data as *const c_void,
      );

      // Set response body data storage
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::WriteData as c_int,
        self.content_buffer.get() as *mut c_void,
      );

      // Set header write function
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::HeaderFunction as c_int,
        write_data as *const c_void,
      );

      // Set response header data storage
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::HeaderData as c_int,
        self.header_buffer.get() as *mut c_void,
      );

      // Set response header data storage
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::HeaderData as c_int,
        self.header_buffer.get() as *mut c_void,
      );

      // *** Important: Enable cookie engine ***
      (self.lib.easy_setopt)(
        self.handle,
        CurlOpt::CookieJar as c_int,
        std::ptr::null::<c_void>(), // Use in-memory cookie jar
      );
    }
  }

  pub fn check_close(&self) -> Result<()> {
    // Check if the handle is valid
    if self.closed {
      return Err(Error::from_reason("Curl instance is closed"));
    }
    if self.handle.is_null() {
      return Err(Error::from_reason("Curl handle is null"));
    }
    Ok(())
  }

  #[napi]
  pub fn set_headers_raw(&self, headers: Vec<String>) -> Result<()> {
    self.check_close()?;
    // Release old header list
    unsafe {
      if let Some(list) = *self.req_header.get() {
        (self.lib.slist_free_all)(list);
        *self.req_header.get() = None;
      }
    }
    // Build new header list
    let mut current_list = std::ptr::null_mut();
    for header in headers {
      let header_cstr = std::ffi::CString::new(header)
        .map_err(|_| Error::new(Status::InvalidArg, "Invalid header string"))?;
      unsafe {
        current_list = (self.lib.slist_append)(current_list, header_cstr.as_ptr());
      }
    }
    // Save to struct
    unsafe {
      *self.req_header.get() = if current_list.is_null() {
        None
      } else {
        Some(current_list)
      };
    }
    self.set_opt(CurlOpt::HttpHeader, current_list as *const c_void)
  }

  pub fn set_opt(&self, option: CurlOpt, value: *const c_void) -> Result<()> {
    self.check_close()?;
    log_info!(
      "Curl",
      "Setting option: {:?} with value: {:?}",
      option,
      value
    );
    self.result(unsafe { (self.lib.easy_setopt)(self.handle, option as c_int, value) })
  }

  #[napi]
  pub fn set_option(&self, option: CurlOpt, value: Either3<String, i64, bool>) -> Result<()> {
    match value {
      Either3::A(string_value) => {
        let c_str = std::ffi::CString::new(string_value).unwrap();
        self.set_opt(option, c_str.as_ptr() as *const c_void)
      }
      Either3::B(long_value) => self.set_opt(option, long_value as *const c_void),
      Either3::C(bool_value) => {
        self.set_opt(option, if bool_value { 1 } else { 0 } as *const c_void)
      }
    }
  }

  #[napi]
  pub fn set_body(&self, value: Either<String, Buffer>) -> Result<()> {
    self.check_close()?;

    let bytes = match value {
      Either::A(string_value) => {
        // If it's a String, convert to bytes
        string_value.into_bytes()
      }
      Either::B(buffer_value) => {
        // If it's a Buffer, convert to Vec<u8>
        buffer_value.to_vec()
      }
    };

    unsafe {
      (*self.req_body.get()) = bytes;
    }
    let buf = unsafe { &(*self.req_body.get()) };
    self.set_opt(CurlOpt::PostFields, buf.as_ptr() as *const c_void)?;
    self.set_opt(CurlOpt::PostFieldSize, buf.len() as *const c_void)
  }

  #[napi]
  pub fn set_opt_str_list(&self, option: CurlOpt, arrays: Vec<String>) -> Result<()> {
    self.check_close()?;
    self.set_opt(option, arrays.as_ptr() as *const c_void)
  }

  fn result(&self, code: i32) -> Result<()> {
    if code != 0 {
      Err(Error::new(
        Status::GenericFailure,
        format!("failed with code: {} message:{}", code, self.error(code)),
      ))
    } else {
      Ok(())
    }
  }

  pub fn get_info(&self, info: CurlInfo, value: *mut c_void) -> Result<()> {
    self.check_close()?;
    log_info!("Curl", "{:?}Get info: {:?} ", self.id(), info);
    self.result(unsafe { (self.lib.easy_getinfo)(self.handle, info as c_int, value) })
  }

  /// Get response code
  #[napi]
  pub fn get_info_number(&self, option: CurlInfo) -> Result<i64> {
    let mut response_code: c_long = 0;
    self.get_info(option, &mut response_code as *mut _ as *mut c_void)?;
    Ok(response_code as i64)
  }

  /// Get string information
  #[napi]
  pub fn get_info_string(&self, option: CurlInfo) -> Result<String> {
    let mut url_ptr: *mut c_char = std::ptr::null_mut();
    self.get_info(option, &mut url_ptr as *mut _ as *mut c_void)?;
    let cstr = unsafe { std::ffi::CStr::from_ptr(url_ptr) };
    Ok(cstr.to_string_lossy().to_string())
  }

  /// Impersonate browser
  #[napi]
  pub fn impersonate(&self, target: String, default_headers: Option<bool>) -> Result<()> {
    self.check_close()?;
    let target_cstr = std::ffi::CString::new(target.clone()).unwrap();
    let use_default_headers = default_headers.unwrap_or(true);
    log_info!(
      "Curl",
      "Impersonating as: {} with default headers: {}",
      target,
      use_default_headers
    );

    self.result(unsafe {
      (self.lib.easy_impersonate)(
        self.handle,
        target_cstr.as_ptr(),
        if use_default_headers { 1 } else { 0 },
      )
    })
  }

  /// Get error message string
  #[napi]
  pub fn error(&self, code: i32) -> String {
    log_info!("Curl", "error {}", code);
    curl_easy_error(code)
  }

  /// Get curl ID
  #[napi]
  pub fn id(&self) -> String {
    get_ptr_address(self.handle)
  }

  /// Get curl handle (internal use) - Added safety check
  pub fn get_handle(&self) -> CurlHandle {
    if self.handle.is_null() {
      println!("Warning: curl handle is null!");
    }
    self.handle
  }

  /// Clean up curl handle
  #[napi]
  pub fn close(&mut self) {
    if self.closed {
      return;
    }
    self.closed = true;
    if self.handle.is_null() {
      return;
    }

    log_info!("Curl", "easy_cleanup {:?}", self.id());
    unsafe {
      // Release header list
      if let Some(list) = *self.req_header.get() {
        (self.lib.slist_free_all)(list);
        *self.req_header.get() = None;
      }
      // Clear body data
      (*self.req_body.get()).clear();

      (self.lib.easy_cleanup)(self.handle);
    }
  }

  /// Reset curl
  #[napi]
  pub fn reset(&self) -> Result<()> {
    self.check_close()?;
    log_info!("Curl", "easy_reset");
    unsafe {
      (*self.header_buffer.get()).clear();
      (*self.content_buffer.get()).clear();

      (self.lib.easy_reset)(self.handle);
    }
    Ok(())
  }

  /// Execute curl request
  #[napi]
  pub fn perform_sync(&self) -> Result<()> {
    // Ensure data callback is initialized
    self.init();
    log_info!("Curl", "perform");
    self.result(unsafe { (self.lib.easy_perform)(self.handle) })
  }
  #[napi]
  pub async fn perform(&self) -> Result<()> {
    // Ensure data callback is initialized
    self.init();
    log_info!("Curl", "perform");
    // self.result(unsafe { (self.lib.easy_perform)(self.handle) })

    let handle = self.handle as usize;
    tokio::task::spawn_blocking(move || {
      unsafe {
        // Restore lib reference
        let lib = napi_load_library()?;
        let code = (lib.easy_perform)(handle as CurlHandle);
        if code != 0 {
          let error = curl_easy_error(code);
          return Err(Error::from_reason(format!(
            "failed with code: {} message:{}",
            code, error
          )));
        }
        Ok(())
      }
    })
    .await
    .map_err(|e| Error::from_reason(format!("Tokio join error: {e}")))?
  }

  /// Get response header data
  #[napi]
  pub fn get_resp_headers(&self) -> Buffer {
    unsafe { Buffer::from((*self.header_buffer.get()).clone()) }
  }

  /// Get response body data
  #[napi]
  pub fn get_resp_body(&self) -> Buffer {
    unsafe { Buffer::from((*self.content_buffer.get()).clone()) }
  }

  /// Get info list
  #[napi]
  pub fn get_info_list(&self, option: CurlInfo) -> Result<Vec<String>> {
    self.check_close()?;
    log_info!("Curl", "get_info_list {:?}", option);
    let mut cookie_list: CurlSlist = std::ptr::null_mut();
    self.get_info(option, &mut cookie_list as *mut _ as *mut c_void)?;
    let mut cookies = Vec::new();
    if !cookie_list.is_null() {
      unsafe {
        // Convert pointer to correct struct type
        let mut current = cookie_list as *mut CurlSlistNode;

        // Traverse the linked list - like the while(each) loop in C code
        while !current.is_null() {
          let node = &*current;

          // Check if data pointer is valid
          if !node.data.is_null() {
            // Convert C string to Rust string
            let cstr = std::ffi::CStr::from_ptr(node.data);
            if let Ok(cookie_str) = cstr.to_str() {
              cookies.push(cookie_str.to_string());
            }
          }

          // Move to next node - equivalent to C code: each = each->next
          current = node.next;
        }

        // Free cookie list - equivalent to C code: curl_slist_free_all(cookies)
        (self.lib.slist_free_all)(cookie_list);
      }
    }

    Ok(cookies)
  }
  /// Set linked list
  #[napi]
  pub fn set_opt_list(&self, option: CurlOpt, arrays: Vec<String>) -> Result<()> {
    self.check_close()?;
    log_info!("Curl", "set_opt_list {:?}", option);
    let mut list_ptr: CurlSlist = std::ptr::null_mut();

    for item in arrays {
      let item_cstr = std::ffi::CString::new(item)
        .map_err(|_| Error::new(Status::InvalidArg, "Invalid cookie string"))?;
      unsafe {
        list_ptr = (self.lib.slist_append)(list_ptr, item_cstr.as_ptr());
      }
    }

    if list_ptr.is_null() {
      return Err(Error::new(
        Status::GenericFailure,
        "Failed to create cookie list",
      ));
    }
    self.set_opt(option, list_ptr as *const c_void)?;
    // Free linked list
    unsafe {
      (self.lib.slist_free_all)(list_ptr);
    }

    Ok(())
  }

  /// Get cookie list
  #[napi]
  pub fn get_cookies(&self) -> Result<Vec<String>> {
    self.get_info_list(CurlInfo::CookieList)
  }

  /// Set cookie
  #[napi]
  pub fn set_cookies(&self, cookie: String) -> Result<()> {
    self.check_close()?;
    let cookie_cstr = std::ffi::CString::new(cookie)
      .map_err(|_| Error::new(Status::InvalidArg, "Invalid cookie string"))?;
    self.set_opt(CurlOpt::Cookie, cookie_cstr.as_ptr() as *const c_void)
  }

  #[napi]
  pub fn status(&self) -> Result<i32> {
    let result = self.get_info_number(CurlInfo::ResponseCode)?;
    Ok(result as i32)
  }
}

// For safety, implement Drop trait to ensure proper resource cleanup
impl Drop for Curl {
  fn drop(&mut self) {
    self.close();
  }
}

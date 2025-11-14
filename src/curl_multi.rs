use std::{
  ffi::{c_int, c_long, c_void},
  sync::{Arc, Mutex},
};

use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;

use crate::{
  api::curl_multi_error,
  constants::CurlMOpt,
  curl::Curl,
  loader::{napi_load_library, CurlFunctions, CurlHandle, CurlMultiHandle},
  utils::get_ptr_address,
};
use crate::{loader::CurlWaitFd, log_info};

#[napi(object)]
pub struct CurlMsgResult {
  pub msg: i64,
  pub easy_handle: i64,
  pub easy_id: String,
  pub data: CurlMsgDataResult,
}

#[napi(object)]
pub struct CurlMsgDataResult {
  pub whatever: i64,
  pub result: i32,
}

#[napi(object)]
pub struct SocketData {
  pub curl_id: String,
  pub sockfd: i32,
  pub what: i32,
}

#[napi(object)]
pub struct TimerData {
  pub multi_id: String,
  pub timeout_ms: i64,
}

#[derive(Debug)]
struct RawMulti {
  handle: CurlMultiHandle,
  lib: &'static CurlFunctions,
}

struct MultiData {
  socket: Box<dyn FnMut(SocketData) + Send>,
  timer: Box<dyn FnMut(TimerData) -> bool + Send>,
}

pub struct MultiPollTask {
  handle: usize,
  timeout_ms: i32,
}

impl napi::Task for MultiPollTask {
  type Output = i32;
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    let mut remaining = 0;
    unsafe {
      let extra_fds: *mut CurlWaitFd = std::ptr::null_mut();
      let lib = napi_load_library()?;
      let code = (lib.multi_poll)(
        self.handle as CurlMultiHandle,
        extra_fds,
        0,
        self.timeout_ms,
        &mut remaining,
      );
      if code != 0 {
        return Err(Error::from_reason(format!(
          "failed with code: {} message: {}",
          code,
          curl_multi_error(code)
        )));
      }
    }
    Ok(remaining)
  }

  fn resolve(&mut self, _env: napi::Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }

  fn reject(&mut self, _env: napi::Env, err: Error) -> Result<Self::JsValue> {
    Err(err)
  }
}

pub struct MultiWaitTask {
  handle: usize,
  timeout_ms: i32,
}

impl napi::Task for MultiWaitTask {
  type Output = i32;
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    let mut remaining = 0;
    unsafe {
      let extra_fds: *mut CurlWaitFd = std::ptr::null_mut();
      let lib = napi_load_library()?;
      let code = (lib.multi_wait)(
        self.handle as CurlMultiHandle,
        extra_fds,
        0,
        self.timeout_ms,
        &mut remaining,
      );
      if code != 0 {
        return Err(Error::from_reason(format!(
          "failed with code: {} message: {}",
          code,
          curl_multi_error(code)
        )));
      }
    }
    Ok(remaining)
  }

  fn resolve(&mut self, _env: napi::Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }

  fn reject(&mut self, _env: napi::Env, err: Error) -> Result<Self::JsValue> {
    Err(err)
  }
}

#[napi(js_name = "CurlMulti")]
pub struct CurlMulti {
  pub closed: bool,
  raw: Arc<RawMulti>,
  data: Arc<Mutex<MultiData>>,
  socket_data_ptr: Option<*const Mutex<MultiData>>,
  timer_data_ptr: Option<*const Mutex<MultiData>>,
}

unsafe impl Send for CurlMulti {}
unsafe impl Sync for CurlMulti {}

#[napi]
impl CurlMulti {
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let lib = napi_load_library()?;
    let handle = unsafe { (lib.multi_init)() };

    if handle.is_null() {
      return Err(Error::from_reason("Failed to initialize curl multi handle"));
    }

    let multi = Self {
      raw: Arc::new(RawMulti { handle, lib }),
      data: Arc::new(Mutex::new(MultiData {
        socket: Box::new(|_| {}),
        timer: Box::new(|_| true),
      })),
      closed: false,
      socket_data_ptr: None,
      timer_data_ptr: None,
    };

    // multi.setup_default_callbacks()?;
    Ok(multi)
  }

  fn result(&self, code: i32) -> Result<()> {
    if code != 0 {
      Err(Error::new(
        Status::GenericFailure,
        format!(
          "failed with code: {} message:{}",
          code,
          self.error(code.into())
        ),
      ))
    } else {
      Ok(())
    }
  }

  pub fn set_opt(&self, option: CurlMOpt, value: *const c_void) -> Result<()> {
    self.check_close()?;
    log_info!(
      "Curl",
      "Setting option: {:?} with value: {:?}",
      option,
      value
    );
    self.result(unsafe { (self.raw.lib.multi_setopt)(self.raw.handle, option as c_int, value) })
  }

  #[napi]
  pub fn set_option(&self, option: CurlMOpt, value: Either3<String, i64, bool>) -> Result<()> {
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
  pub fn set_opt_buffer(&self, option: CurlMOpt, body: Buffer) -> Result<()> {
    self.set_opt(option, body.as_ptr() as *const c_void)
  }

  pub fn check_close(&self) -> Result<()> {
    if self.closed {
      return Err(Error::from_reason("CurlMulti has been closed"));
    }
    // Check if the handle is valid
    if self.raw.handle.is_null() {
      return Err(Error::from_reason("Curl multi handle is null"));
    }
    Ok(())
  }

  #[napi]
  pub fn set_socket_callback(&mut self, callback: ThreadsafeFunction<SocketData>) -> Result<()> {
    self.check_close()?;
    let tsfn = Arc::new(callback);
    log_info!("CurlMulti", "Setting socket callback");
    if let Ok(mut data) = self.data.lock() {
      data.socket = Box::new(move |sdata| {
        log_info!(
          "CurlMulti",
          "Socket callback: curl_id={}, sockfd={}, what={}",
          sdata.curl_id,
          sdata.sockfd,
          sdata.what
        );
        let _ = tsfn.call(
          Ok(sdata),
          napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
      });
    }

    // If a callback was set before, clean up the old pointer first
    if let Some(old_ptr) = self.socket_data_ptr {
      unsafe { Arc::from_raw(old_ptr) };
    }

    // Convert Arc to raw pointer and pass to libcurl
    let data_ptr = Arc::into_raw(Arc::clone(&self.data));
    self.socket_data_ptr = Some(data_ptr);

    unsafe {
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::SocketFunction as c_int,
        socket_callback as *const c_void,
      );
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::SocketData as c_int,
        data_ptr as *const c_void,
      );
    }
    Ok(())
  }

  #[napi]
  pub fn set_timer_callback(&mut self, callback: ThreadsafeFunction<TimerData>) -> Result<()> {
    self.check_close()?;
    let tsfn = Arc::new(callback);
    log_info!("CurlMulti", "Setting timer callback");
    if let Ok(mut data) = self.data.lock() {
      data.timer = Box::new(move |tdata| {
        log_info!(
          "CurlMulti",
          "Timer callback: multi_id={}, timeout_ms={}",
          tdata.multi_id,
          tdata.timeout_ms
        );
        let _ = tsfn.call(
          Ok(tdata),
          napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
        );
        true
      });
    }

    // If a callback was set before, clean up the old pointer first
    if let Some(old_ptr) = self.timer_data_ptr {
      unsafe { Arc::from_raw(old_ptr) };
    }

    // Convert Arc to raw pointer and pass to libcurl
    let data_ptr = Arc::into_raw(Arc::clone(&self.data));
    self.timer_data_ptr = Some(data_ptr);

    unsafe {
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::TimerFunction as c_int,
        timer_callback as *const c_void,
      );
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::TimerData as c_int,
        data_ptr as *const c_void,
      );
    }
    Ok(())
  }

  #[napi]
  pub fn add_handle(&self, curl: &Curl) -> Result<i32> {
    self.check_close()?;
    let handle = curl.get_handle();
    if handle.is_null() {
      return Err(Error::from_reason("Invalid curl handle"));
    }
    curl.init();
    log_info!("CurlMulti", "Adding handle: {}", get_ptr_address(handle));
    unsafe { Ok((self.raw.lib.multi_add_handle)(self.raw.handle, handle)) }
  }

  #[napi]
  pub fn remove_handle(&self, curl: &Curl) -> Result<i32> {
    self.check_close()?;
    let handle = curl.get_handle();
    if handle.is_null() {
      return Err(Error::from_reason("Invalid curl handle"));
    }
    log_info!("CurlMulti", "Removing handle: {}", get_ptr_address(handle));
    unsafe { Ok((self.raw.lib.multi_remove_handle)(self.raw.handle, handle)) }
  }

  #[napi]
  pub fn error(&self, err: i64) -> String {
    log_info!("CurlMulti", "Getting error for code: {}", err);
    curl_multi_error(err as i32)
  }

  #[napi]
  pub fn perform(&self) -> Result<i32> {
    self.check_close()?;
    let mut remaining = 0;
    log_info!("CurlMulti", "Performing multi operation");
    unsafe {
      let result = (self.raw.lib.multi_perform)(self.raw.handle, &mut remaining);
      if result != 0 && result != 1 {
        return Err(Error::from_reason(format!("Perform failed: {}", result)));
      }
    }
    Ok(remaining)
  }

  #[napi]
  pub fn poll(&self, timeout_ms: i32) -> Result<AsyncTask<MultiPollTask>> {
    self.check_close()?;
    let handle = self.raw.handle as usize;
    Ok(AsyncTask::new(MultiPollTask { handle, timeout_ms }))
  }

  #[napi]
  pub fn wait(&self, timeout_ms: i32) -> Result<AsyncTask<MultiWaitTask>> {
    self.check_close()?;
    let handle = self.raw.handle as usize;
    Ok(AsyncTask::new(MultiWaitTask { handle, timeout_ms }))
  }

  #[napi]
  pub fn get_running_handles(&self) -> Result<i32> {
    self.check_close()?;
    let mut remaining = 0;
    log_info!("CurlMulti", "Getting running handles");
    unsafe {
      let result = (self.raw.lib.multi_perform)(self.raw.handle, &mut remaining);
      if result != 0 && result != 1 {
        return Err(Error::from_reason(format!(
          "Get running handles failed: {}",
          result
        )));
      }
    }
    Ok(remaining)
  }

  #[napi]
  pub fn socket_action(&self, socket: i64, what: i64) -> Result<i32> {
    self.check_close()?;
    let mut remaining = 0;
    log_info!(
      "CurlMulti",
      "Performing socket action: socket={}, what={}",
      socket,
      what
    );
    unsafe {
      let result = (self.raw.lib.multi_socket_action)(
        self.raw.handle,
        socket as c_int,
        what as c_int,
        &mut remaining,
      );
      if result != 0 {
        return Err(Error::from_reason(format!("Action failed: {}", result)));
      }
    }
    Ok(remaining)
  }

  #[napi]
  pub fn info_read(&self) -> Result<Option<CurlMsgResult>> {
    self.check_close()?;
    if self.raw.handle.is_null() {
      return Err(Error::from_reason("Curl multi handle is null"));
    }
    log_info!("CurlMulti", "Reading info from multi handle");

    let mut msgs_left = 0;
    let msg_ptr = unsafe { (self.raw.lib.multi_info_read)(self.raw.handle, &mut msgs_left) };

    // println!("info_read: msg_ptr={:p}, msgs_left={}", msg_ptr, msgs_left);

    if msg_ptr.is_null() {
      // Get current running transfers count for debugging
      let mut running = 0;
      unsafe {
        let result = (self.raw.lib.multi_perform)(self.raw.handle, &mut running);
        if result != 0 {
          return Err(Error::from_reason(format!("Info read  failed: {}", result)));
        }
      }

      // Also check the multi handle status
      // println!("info_read: multi_handle={:p}", self.raw.handle);

      return Ok(None);
    }

    // Properly dereference pointer to access struct fields
    let curl_msg = unsafe { &*msg_ptr };

    // println!(
    //   "info_read: found message - msg={}, easy_handle={:p}, result={}",
    //   curl_msg.msg,
    //   curl_msg.easy_handle,
    //   unsafe { curl_msg.data.result }
    // );

    Ok(Some(CurlMsgResult {
      msg: curl_msg.msg as i64,
      easy_handle: curl_msg.easy_handle as i64,
      easy_id: get_ptr_address(curl_msg.easy_handle),
      data: CurlMsgDataResult {
        whatever: unsafe { curl_msg.data.whatever as i64 },
        result: unsafe { curl_msg.data.result },
      },
    }))
  }

  #[napi]
  pub fn close(&mut self) {
    if self.closed || self.raw.handle.is_null() {
      return;
    }
    log_info!(
      "CurlMulti",
      "Closing CurlMulti handle: {}",
      get_ptr_address(self.raw.handle)
    );
    self.closed = true;

    unsafe {
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::SocketFunction as c_int,
        std::ptr::null(),
      );
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::SocketData as c_int,
        std::ptr::null(),
      );

      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::TimerFunction as c_int,
        std::ptr::null(),
      );
      (self.raw.lib.multi_setopt)(
        self.raw.handle,
        CurlMOpt::TimerData as c_int,
        std::ptr::null(),
      );

      (self.raw.lib.multi_cleanup)(self.raw.handle);
    }

    // Clean up allocated pointers
    if let Some(ptr) = self.socket_data_ptr.take() {
      unsafe { Arc::from_raw(ptr) };
    }
    if let Some(ptr) = self.timer_data_ptr.take() {
      unsafe { Arc::from_raw(ptr) };
    }

    if let Ok(mut data) = self.data.lock() {
      data.socket = Box::new(|_| {});
      data.timer = Box::new(|_| true);
    }
  }
}

impl Drop for CurlMulti {
  fn drop(&mut self) {
    self.close();
  }
}

extern "C" fn socket_callback(
  _easy: CurlHandle,
  sockfd: c_int,
  what: c_int,
  userptr: *mut c_void,
  _socketp: *mut c_void,
) -> c_int {
  if userptr.is_null() {
    return 0;
  }

  // Reconstruct Arc from raw pointer but don't release ownership
  let data_arc = unsafe { Arc::from_raw(userptr as *const Mutex<MultiData>) };
  let result = if let Ok(mut data) = data_arc.try_lock() {
    (data.socket)(SocketData {
      curl_id: get_ptr_address(_easy),
      sockfd,
      what,
    });
    0
  } else {
    0
  };

  // Leak Arc again to avoid freeing memory
  std::mem::forget(data_arc);
  result
}

extern "C" fn timer_callback(
  _multi: CurlMultiHandle,
  timeout_ms: c_long,
  userptr: *mut c_void,
) -> c_int {
  if userptr.is_null() {
    return 0;
  }

  // Reconstruct Arc from raw pointer but don't release ownership
  let data_arc = unsafe { Arc::from_raw(userptr as *const Mutex<MultiData>) };
  let result = if let Ok(mut data) = data_arc.try_lock() {
    let keep_going = (data.timer)(TimerData {
      multi_id: get_ptr_address(_multi),
      timeout_ms: timeout_ms as i64,
    });
    if keep_going {
      0
    } else {
      -1
    }
  } else {
    0
  };

  // Leak Arc again to avoid freeing memory
  std::mem::forget(data_arc);
  result
}

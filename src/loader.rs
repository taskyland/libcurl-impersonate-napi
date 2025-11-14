use libloading::{Library, Symbol};
use napi::{Error, Status};
use once_cell::sync::OnceCell;
use std::{
  ffi::c_short,
  os::raw::{c_char, c_int, c_long, c_uint, c_void},
};

use crate::libpath::get_lib_path;

// Define type aliases for curl library
pub type CurlHandle = *mut std::ffi::c_void;
pub type CurlMultiHandle = *mut std::ffi::c_void;
pub type CurlSlist = *mut std::ffi::c_void;
pub type CurlMime = *mut std::ffi::c_void;
pub type CurlMimepart = *mut std::ffi::c_void;
pub type CurlShare = *mut std::ffi::c_void;
pub type CurlUrl = *mut std::ffi::c_void;
pub type CurlForm = *mut std::ffi::c_void;
pub type CurlHttpPost = *mut c_void;

// Store loaded function instances
static CURL_FUNCTIONS: OnceCell<CurlFunctions> = OnceCell::new();

// Callback function types
pub type WriteCallback = unsafe extern "C" fn(
  contents: *mut c_char,
  size: usize,
  nmemb: usize,
  userp: *mut c_void,
) -> usize;

pub type ReadCallback = unsafe extern "C" fn(
  buffer: *mut c_char,
  size: usize,
  nitems: usize,
  userp: *mut c_void,
) -> usize;

pub type ProgressCallback = unsafe extern "C" fn(
  clientp: *mut c_void,
  dltotal: f64,
  dlnow: f64,
  ultotal: f64,
  ulnow: f64,
) -> c_int;

pub type HeaderCallback = unsafe extern "C" fn(
  buffer: *mut c_char,
  size: usize,
  nitems: usize,
  userdata: *mut c_void,
) -> usize;

// Multi socket and timer callback types
pub type CurlSocketCallback = unsafe extern "C" fn(
  easy: CurlHandle,
  s: c_int,
  what: c_int,
  userp: *mut c_void,
  socketp: *mut c_void,
) -> c_int;

pub type CurlTimerCallback =
  unsafe extern "C" fn(multi: CurlMultiHandle, timeout_ms: c_long, userp: *mut c_void) -> c_int;

// pollfd struct type
#[repr(C)]
pub struct CurlWaitFd {
  pub fd: c_int,
  pub events: c_short,
  pub revents: c_short,
}

#[repr(C)]
pub struct CurlSlistNode {
  pub data: *mut c_char,
  pub next: *mut CurlSlistNode,
}

pub type CurlEasyInit = unsafe extern "C" fn() -> CurlHandle;
pub type CurlEasyCleanup = unsafe extern "C" fn(handle: CurlHandle);
pub type CurlEasySetopt =
  unsafe extern "C" fn(handle: CurlHandle, option: c_int, value: *const c_void) -> c_int;
pub type CurlEasyPerform = unsafe extern "C" fn(handle: CurlHandle) -> c_int;
pub type CurlEasyGetinfo =
  unsafe extern "C" fn(handle: CurlHandle, info: c_int, value: *mut c_void) -> c_int;
pub type CurlEasyDuphandle = unsafe extern "C" fn(handle: CurlHandle) -> CurlHandle;
pub type CurlEasyReset = unsafe extern "C" fn(handle: CurlHandle);
pub type CurlEasyStrerror = unsafe extern "C" fn(code: c_int) -> *const c_char;
pub type CurlEasyImpersonate =
  unsafe extern "C" fn(handle: CurlHandle, target: *const c_char, default_headers: c_int) -> c_int;
pub type CurlEasyEscape =
  unsafe extern "C" fn(handle: CurlHandle, string: *const c_char, length: c_int) -> *mut c_char;
pub type CurlEasyUnescape = unsafe extern "C" fn(
  handle: CurlHandle,
  string: *const c_char,
  length: c_int,
  outlength: *mut c_int,
) -> *mut c_char;
pub type CurlEasyHeader = unsafe extern "C" fn(
  handle: CurlHandle,
  name: *const c_char,
  size: usize,
  amount: usize,
  index: usize,
  origin: c_uint,
  request: *mut c_void,
) -> *mut c_void;
pub type CurlEasyNextheader = unsafe extern "C" fn(
  handle: CurlHandle,
  origin: c_uint,
  request: c_int,
  prev: *mut c_void,
) -> *mut c_void;
pub type CurlEasyOptionById = unsafe extern "C" fn(id: c_int) -> *const c_void;
pub type CurlEasyOptionByName = unsafe extern "C" fn(name: *const c_char) -> *const c_void;
pub type CurlEasyOptionNext = unsafe extern "C" fn(prev: *const c_void) -> *const c_void;
pub type CurlEasyPause = unsafe extern "C" fn(handle: CurlHandle, bitmask: c_int) -> c_int;
pub type CurlEasyRecv = unsafe extern "C" fn(
  handle: CurlHandle,
  buffer: *mut c_void,
  buflen: usize,
  n: *mut usize,
) -> c_int;
pub type CurlEasySend = unsafe extern "C" fn(
  handle: CurlHandle,
  buffer: *const c_void,
  buflen: usize,
  n: *mut usize,
) -> c_int;
pub type CurlEasySslsExport =
  unsafe extern "C" fn(handle: CurlHandle, ssl_ctx: *mut *mut c_void) -> c_int;
pub type CurlEasyUpkeep = unsafe extern "C" fn(handle: CurlHandle) -> c_int;

// Multi interface function types
pub type CurlMultiInit = unsafe extern "C" fn() -> CurlMultiHandle;
pub type CurlMultiCleanup = unsafe extern "C" fn(handle: CurlMultiHandle) -> c_int;
pub type CurlMultiPerform =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, running_handles: *mut c_int) -> c_int;
pub type CurlMultiWait = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  extra_fds: *mut CurlWaitFd,
  extra_nfds: c_uint,
  timeout_ms: c_int,
  ret: *mut c_int,
) -> c_int;
pub type CurlMultiPoll = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  extra_fds: *mut CurlWaitFd,
  extra_nfds: c_uint,
  timeout_ms: c_int,
  ret: *mut c_int,
) -> c_int;
pub type CurlMultiWakeup = unsafe extern "C" fn(multi_handle: CurlMultiHandle) -> c_int;
pub type CurlMultiAddHandle =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, easy_handle: CurlHandle) -> c_int;
pub type CurlMultiRemoveHandle =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, easy_handle: CurlHandle) -> c_int;
pub type CurlMultiInfoRead =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, msgs_in_queue: *mut c_int) -> *mut CurlMsg;
pub type CurlMultiSetopt =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, option: c_int, value: *const c_void) -> c_int;
pub type CurlMultiStrerror = unsafe extern "C" fn(code: c_int) -> *const c_char;
pub type CurlMultiSocket = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  sockfd: c_int,
  running_handles: *mut c_int,
) -> c_int;
pub type CurlMultiSocketAction = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  sockfd: c_int,
  ev_bitmask: c_int,
  running_handles: *mut c_int,
) -> c_int;
pub type CurlMultiSocketAll =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, running_handles: *mut c_int) -> c_int;
pub type CurlMultiTimeout =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, timeout: *mut c_long) -> c_int;
pub type CurlMultiFdset = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  read_fd_set: *mut c_void,
  write_fd_set: *mut c_void,
  exc_fd_set: *mut c_void,
  max_fd: *mut c_int,
) -> c_int;
pub type CurlMultiAssign =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle, sockfd: c_int, sockp: *mut c_void) -> c_int;
pub type CurlMultiGetHandles =
  unsafe extern "C" fn(multi_handle: CurlMultiHandle) -> *mut CurlHandle;
pub type CurlMultiWaitfds = unsafe extern "C" fn(
  multi_handle: CurlMultiHandle,
  ufds: *mut CurlWaitFd,
  size: c_uint,
  fd_count: *mut c_uint,
) -> c_int;

// Slist functions
pub type CurlSlistAppend =
  unsafe extern "C" fn(list: CurlSlist, string: *const c_char) -> CurlSlist;
pub type CurlSlistFreeAll = unsafe extern "C" fn(list: CurlSlist);

// MIME API function types
pub type CurlMimeInit = unsafe extern "C" fn(easy: CurlHandle) -> CurlMime;
pub type CurlMimeFree = unsafe extern "C" fn(mime: CurlMime);
pub type CurlMimeAddpart = unsafe extern "C" fn(mime: CurlMime) -> CurlMimepart;
pub type CurlMimeName = unsafe extern "C" fn(part: CurlMimepart, name: *const c_char) -> c_int;
pub type CurlMimeData =
  unsafe extern "C" fn(part: CurlMimepart, data: *const c_char, datasize: usize) -> c_int;
pub type CurlMimeDataCb = unsafe extern "C" fn(
  part: CurlMimepart,
  readfunc: ReadCallback,
  seekfunc: *mut c_void,
  freefunc: *mut c_void,
  arg: *mut c_void,
) -> c_int;
pub type CurlMimeEncoder =
  unsafe extern "C" fn(part: CurlMimepart, encoding: *const c_char) -> c_int;
pub type CurlMimeFiledata =
  unsafe extern "C" fn(part: CurlMimepart, filename: *const c_char) -> c_int;
pub type CurlMimeFilename =
  unsafe extern "C" fn(part: CurlMimepart, filename: *const c_char) -> c_int;
pub type CurlMimeHeaders = unsafe extern "C" fn(part: CurlMimepart, headers: CurlSlist) -> c_int;
pub type CurlMimeSubparts = unsafe extern "C" fn(part: CurlMimepart, subparts: CurlMime) -> c_int;
pub type CurlMimeType = unsafe extern "C" fn(part: CurlMimepart, mimetype: *const c_char) -> c_int;

// Form API function types
pub type CurlFormadd =
  unsafe extern "C" fn(httppost: *mut CurlHttpPost, last_post: *mut CurlHttpPost, ...) -> c_int;
pub type CurlFormfree = unsafe extern "C" fn(form: CurlHttpPost);
pub type CurlFormget = unsafe extern "C" fn(
  form: CurlHttpPost,
  arg: *mut c_void,
  append: unsafe extern "C" fn(*mut c_void, *const c_char, usize) -> usize,
) -> c_int;

// Share API function types
pub type CurlShareInit = unsafe extern "C" fn() -> CurlShare;
pub type CurlShareCleanup = unsafe extern "C" fn(share: CurlShare) -> c_int;
pub type CurlShareSetopt =
  unsafe extern "C" fn(share: CurlShare, option: c_int, value: *const c_void) -> c_int;
pub type CurlShareStrerror = unsafe extern "C" fn(code: c_int) -> *const c_char;

// URL API function types
pub type CurlUrlInit = unsafe extern "C" fn() -> CurlUrl;
pub type CurlUrlCleanup = unsafe extern "C" fn(handle: CurlUrl);
pub type CurlUrlDup = unsafe extern "C" fn(in_url: CurlUrl) -> CurlUrl;
pub type CurlUrlSet = unsafe extern "C" fn(
  handle: CurlUrl,
  part: c_int,
  content: *const c_char,
  flags: c_uint,
) -> c_int;
pub type CurlUrlGet = unsafe extern "C" fn(
  handle: CurlUrl,
  part: c_int,
  content: *mut *mut c_char,
  flags: c_uint,
) -> c_int;
pub type CurlUrlStrerror = unsafe extern "C" fn(code: c_int) -> *const c_char;

// Global function types
pub type CurlGlobalInit = unsafe extern "C" fn(flags: c_long) -> c_int;
pub type CurlGlobalInitMem = unsafe extern "C" fn(
  flags: c_long,
  m: unsafe extern "C" fn(usize) -> *mut c_void,
  f: unsafe extern "C" fn(*mut c_void),
  r: unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void,
  s: unsafe extern "C" fn(*const c_void) -> *mut c_char,
  c: unsafe extern "C" fn(*mut c_void, usize),
) -> c_int;
pub type CurlGlobalCleanup = unsafe extern "C" fn();
pub type CurlGlobalSslset =
  unsafe extern "C" fn(id: c_int, name: *const c_char, avail: *mut *const c_void) -> c_int;
pub type CurlGlobalTrace = unsafe extern "C" fn(config: *const c_char) -> c_int;
pub type CurlVersion = unsafe extern "C" fn() -> *const c_char;
pub type CurlVersionInfo = unsafe extern "C" fn(age: c_int) -> *mut c_void;
pub type CurlEscape = unsafe extern "C" fn(string: *const c_char, length: c_int) -> *mut c_char;
pub type CurlUnescape = unsafe extern "C" fn(string: *const c_char, length: c_int) -> *mut c_char;
pub type CurlFree = unsafe extern "C" fn(p: *mut c_void);

// String processing function types
pub type CurlStrequal = unsafe extern "C" fn(s1: *const c_char, s2: *const c_char) -> c_int;
pub type CurlStrnequal =
  unsafe extern "C" fn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

// Time function types
pub type CurlGetdate =
  unsafe extern "C" fn(datestring: *const c_char, now: *const c_long) -> c_long;

// Environment function types
pub type CurlGetenv = unsafe extern "C" fn(variable: *const c_char) -> *mut c_char;

// printf series function types
pub type CurlMaprintf = unsafe extern "C" fn(format: *const c_char, ...) -> *mut c_char;
pub type CurlMfprintf = unsafe extern "C" fn(fd: *mut c_void, format: *const c_char, ...) -> c_int;
pub type CurlMprintf = unsafe extern "C" fn(format: *const c_char, ...) -> c_int;
pub type CurlMsnprintf =
  unsafe extern "C" fn(buffer: *mut c_char, maxlength: usize, format: *const c_char, ...) -> c_int;
pub type CurlMsprintf =
  unsafe extern "C" fn(buffer: *mut c_char, format: *const c_char, ...) -> c_int;
pub type CurlMvaprintf =
  unsafe extern "C" fn(format: *const c_char, args: *mut c_void) -> *mut c_char;
pub type CurlMvfprintf =
  unsafe extern "C" fn(fd: *mut c_void, format: *const c_char, args: *mut c_void) -> c_int;
pub type CurlMvprintf = unsafe extern "C" fn(format: *const c_char, args: *mut c_void) -> c_int;
pub type CurlMvsnprintf = unsafe extern "C" fn(
  buffer: *mut c_char,
  maxlength: usize,
  format: *const c_char,
  args: *mut c_void,
) -> c_int;
pub type CurlMvsprintf =
  unsafe extern "C" fn(buffer: *mut c_char, format: *const c_char, args: *mut c_void) -> c_int;

// WebSocket API function types
pub type CurlWsMeta = unsafe extern "C" fn(handle: CurlHandle) -> *const c_void;
pub type CurlWsRecv = unsafe extern "C" fn(
  handle: CurlHandle,
  buffer: *mut c_void,
  buflen: usize,
  recv: *mut usize,
  meta: *mut c_void,
) -> c_int;
pub type CurlWsSend = unsafe extern "C" fn(
  handle: CurlHandle,
  buffer: *const c_void,
  buflen: usize,
  sent: *mut usize,
  fragsize: c_long,
  flags: c_uint,
) -> c_int;

// Push header function types
pub type CurlPushheaderByname =
  unsafe extern "C" fn(h: *mut c_void, name: *const c_char) -> *mut c_char;
pub type CurlPushheaderBynum = unsafe extern "C" fn(h: *mut c_void, num: usize) -> *mut c_char;

// Store all loaded functions
#[derive(Debug, Clone)]
pub struct CurlFunctions {
  // Easy interface
  pub easy_init: Symbol<'static, CurlEasyInit>,
  pub easy_cleanup: Symbol<'static, CurlEasyCleanup>,
  pub easy_setopt: Symbol<'static, CurlEasySetopt>,
  pub easy_perform: Symbol<'static, CurlEasyPerform>,
  pub easy_getinfo: Symbol<'static, CurlEasyGetinfo>,
  pub easy_duphandle: Symbol<'static, CurlEasyDuphandle>,
  pub easy_reset: Symbol<'static, CurlEasyReset>,
  pub easy_strerror: Symbol<'static, CurlEasyStrerror>,
  pub easy_impersonate: Symbol<'static, CurlEasyImpersonate>,
  pub easy_escape: Symbol<'static, CurlEasyEscape>,
  pub easy_unescape: Symbol<'static, CurlEasyUnescape>,
  pub easy_header: Symbol<'static, CurlEasyHeader>,
  pub easy_nextheader: Symbol<'static, CurlEasyNextheader>,
  pub easy_option_by_id: Symbol<'static, CurlEasyOptionById>,
  pub easy_option_by_name: Symbol<'static, CurlEasyOptionByName>,
  pub easy_option_next: Symbol<'static, CurlEasyOptionNext>,
  pub easy_pause: Symbol<'static, CurlEasyPause>,
  pub easy_recv: Symbol<'static, CurlEasyRecv>,
  pub easy_send: Symbol<'static, CurlEasySend>,
  pub easy_ssls_export: Symbol<'static, CurlEasySslsExport>,
  pub easy_upkeep: Symbol<'static, CurlEasyUpkeep>,

  // Multi interface
  pub multi_init: Symbol<'static, CurlMultiInit>,
  pub multi_cleanup: Symbol<'static, CurlMultiCleanup>,
  pub multi_perform: Symbol<'static, CurlMultiPerform>,
  pub multi_wait: Symbol<'static, CurlMultiWait>,
  pub multi_poll: Symbol<'static, CurlMultiPoll>,
  pub multi_wakeup: Symbol<'static, CurlMultiWakeup>,
  pub multi_add_handle: Symbol<'static, CurlMultiAddHandle>,
  pub multi_remove_handle: Symbol<'static, CurlMultiRemoveHandle>,
  pub multi_info_read: Symbol<'static, CurlMultiInfoRead>,
  pub multi_setopt: Symbol<'static, CurlMultiSetopt>,
  pub multi_strerror: Symbol<'static, CurlMultiStrerror>,
  pub multi_socket: Symbol<'static, CurlMultiSocket>,
  pub multi_socket_action: Symbol<'static, CurlMultiSocketAction>,
  pub multi_socket_all: Symbol<'static, CurlMultiSocketAll>,
  pub multi_timeout: Symbol<'static, CurlMultiTimeout>,
  pub multi_fdset: Symbol<'static, CurlMultiFdset>,
  pub multi_assign: Symbol<'static, CurlMultiAssign>,
  pub multi_get_handles: Symbol<'static, CurlMultiGetHandles>,
  pub multi_waitfds: Symbol<'static, CurlMultiWaitfds>,

  // Slist
  pub slist_append: Symbol<'static, CurlSlistAppend>,
  pub slist_free_all: Symbol<'static, CurlSlistFreeAll>,

  // MIME
  pub mime_init: Symbol<'static, CurlMimeInit>,
  pub mime_free: Symbol<'static, CurlMimeFree>,
  pub mime_addpart: Symbol<'static, CurlMimeAddpart>,
  pub mime_name: Symbol<'static, CurlMimeName>,
  pub mime_data: Symbol<'static, CurlMimeData>,
  pub mime_data_cb: Symbol<'static, CurlMimeDataCb>,
  pub mime_encoder: Symbol<'static, CurlMimeEncoder>,
  pub mime_filedata: Symbol<'static, CurlMimeFiledata>,
  pub mime_filename: Symbol<'static, CurlMimeFilename>,
  pub mime_headers: Symbol<'static, CurlMimeHeaders>,
  pub mime_subparts: Symbol<'static, CurlMimeSubparts>,
  pub mime_type: Symbol<'static, CurlMimeType>,

  // Form API
  pub formadd: Symbol<'static, CurlFormadd>,
  pub formfree: Symbol<'static, CurlFormfree>,
  pub formget: Symbol<'static, CurlFormget>,

  // Share API
  pub share_init: Symbol<'static, CurlShareInit>,
  pub share_cleanup: Symbol<'static, CurlShareCleanup>,
  pub share_setopt: Symbol<'static, CurlShareSetopt>,
  pub share_strerror: Symbol<'static, CurlShareStrerror>,

  // URL API
  pub url: Symbol<'static, CurlUrlInit>,
  pub url_cleanup: Symbol<'static, CurlUrlCleanup>,
  pub url_dup: Symbol<'static, CurlUrlDup>,
  pub url_set: Symbol<'static, CurlUrlSet>,
  pub url_get: Symbol<'static, CurlUrlGet>,
  pub url_strerror: Symbol<'static, CurlUrlStrerror>,

  // Global functions
  pub global_init: Symbol<'static, CurlGlobalInit>,
  pub global_init_mem: Symbol<'static, CurlGlobalInitMem>,
  pub global_cleanup: Symbol<'static, CurlGlobalCleanup>,
  pub global_sslset: Symbol<'static, CurlGlobalSslset>,
  pub global_trace: Symbol<'static, CurlGlobalTrace>,
  pub version: Symbol<'static, CurlVersion>,
  pub version_info: Symbol<'static, CurlVersionInfo>,
  pub escape: Symbol<'static, CurlEscape>,
  pub unescape: Symbol<'static, CurlUnescape>,
  pub free: Symbol<'static, CurlFree>,

  // String processing functions
  pub strequal: Symbol<'static, CurlStrequal>,
  pub strnequal: Symbol<'static, CurlStrnequal>,

  // Time functions
  pub getdate: Symbol<'static, CurlGetdate>,

  // Environment functions
  pub getenv: Symbol<'static, CurlGetenv>,

  // printf series functions
  pub maprintf: Symbol<'static, CurlMaprintf>,
  pub mfprintf: Symbol<'static, CurlMfprintf>,
  pub mprintf: Symbol<'static, CurlMprintf>,
  pub msnprintf: Symbol<'static, CurlMsnprintf>,
  pub msprintf: Symbol<'static, CurlMsprintf>,
  pub mvaprintf: Symbol<'static, CurlMvaprintf>,
  pub mvfprintf: Symbol<'static, CurlMvfprintf>,
  pub mvprintf: Symbol<'static, CurlMvprintf>,
  pub mvsnprintf: Symbol<'static, CurlMvsnprintf>,
  pub mvsprintf: Symbol<'static, CurlMvsprintf>,

  // WebSocket API
  pub ws_meta: Symbol<'static, CurlWsMeta>,
  pub ws_recv: Symbol<'static, CurlWsRecv>,
  pub ws_send: Symbol<'static, CurlWsSend>,

  // Push header functions
  pub pushheader_byname: Symbol<'static, CurlPushheaderByname>,
  pub pushheader_bynum: Symbol<'static, CurlPushheaderBynum>,
}

// Implement Send and Sync traits
unsafe impl Send for CurlFunctions {}
unsafe impl Sync for CurlFunctions {}

#[repr(C)]
pub struct CurlMsg {
  pub msg: c_int,
  pub easy_handle: *mut c_void, // Fixed: This should be void* instead of CurlHandle
  pub data: CurlMsgData,
}

#[repr(C)]
pub union CurlMsgData {
  pub whatever: *mut c_void,
  pub result: c_int,
}

// Load library method
pub fn load_curl_library() -> Result<&'static CurlFunctions, Box<dyn std::error::Error>> {
  let lib_path = get_lib_path().ok_or("lib path is not set")?;

  CURL_FUNCTIONS.get_or_try_init(|| {
    let lib = unsafe { Library::new(&lib_path)? };
    let lib_static: &'static Library = Box::leak(Box::new(lib));

    let functions = CurlFunctions {
      // Easy interface
      easy_init: unsafe { lib_static.get(b"curl_easy_init\0")? },
      easy_cleanup: unsafe { lib_static.get(b"curl_easy_cleanup\0")? },
      easy_setopt: unsafe { lib_static.get(b"curl_easy_setopt\0")? },
      easy_perform: unsafe { lib_static.get(b"curl_easy_perform\0")? },
      easy_getinfo: unsafe { lib_static.get(b"curl_easy_getinfo\0")? },
      easy_duphandle: unsafe { lib_static.get(b"curl_easy_duphandle\0")? },
      easy_reset: unsafe { lib_static.get(b"curl_easy_reset\0")? },
      easy_strerror: unsafe { lib_static.get(b"curl_easy_strerror\0")? },
      easy_impersonate: unsafe { lib_static.get(b"curl_easy_impersonate\0")? },
      easy_escape: unsafe { lib_static.get(b"curl_easy_escape\0")? },
      easy_unescape: unsafe { lib_static.get(b"curl_easy_unescape\0")? },
      easy_header: unsafe { lib_static.get(b"curl_easy_header\0")? },
      easy_nextheader: unsafe { lib_static.get(b"curl_easy_nextheader\0")? },
      easy_option_by_id: unsafe { lib_static.get(b"curl_easy_option_by_id\0")? },
      easy_option_by_name: unsafe { lib_static.get(b"curl_easy_option_by_name\0")? },
      easy_option_next: unsafe { lib_static.get(b"curl_easy_option_next\0")? },
      easy_pause: unsafe { lib_static.get(b"curl_easy_pause\0")? },
      easy_recv: unsafe { lib_static.get(b"curl_easy_recv\0")? },
      easy_send: unsafe { lib_static.get(b"curl_easy_send\0")? },
      easy_ssls_export: unsafe { lib_static.get(b"curl_easy_ssls_export\0")? },
      easy_upkeep: unsafe { lib_static.get(b"curl_easy_upkeep\0")? },

      // Multi interface
      multi_init: unsafe { lib_static.get(b"curl_multi_init\0")? },
      multi_cleanup: unsafe { lib_static.get(b"curl_multi_cleanup\0")? },
      multi_perform: unsafe { lib_static.get(b"curl_multi_perform\0")? },
      multi_wait: unsafe { lib_static.get(b"curl_multi_wait\0")? },
      multi_poll: unsafe { lib_static.get(b"curl_multi_poll\0")? },
      multi_wakeup: unsafe { lib_static.get(b"curl_multi_wakeup\0")? },
      multi_add_handle: unsafe { lib_static.get(b"curl_multi_add_handle\0")? },
      multi_remove_handle: unsafe { lib_static.get(b"curl_multi_remove_handle\0")? },
      multi_info_read: unsafe { lib_static.get(b"curl_multi_info_read\0")? },
      multi_setopt: unsafe { lib_static.get(b"curl_multi_setopt\0")? },
      multi_strerror: unsafe { lib_static.get(b"curl_multi_strerror\0")? },
      multi_socket: unsafe { lib_static.get(b"curl_multi_socket\0")? },
      multi_socket_action: unsafe { lib_static.get(b"curl_multi_socket_action\0")? },
      multi_socket_all: unsafe { lib_static.get(b"curl_multi_socket_all\0")? },
      multi_timeout: unsafe { lib_static.get(b"curl_multi_timeout\0")? },
      multi_fdset: unsafe { lib_static.get(b"curl_multi_fdset\0")? },
      multi_assign: unsafe { lib_static.get(b"curl_multi_assign\0")? },
      multi_get_handles: unsafe { lib_static.get(b"curl_multi_get_handles\0")? },
      multi_waitfds: unsafe { lib_static.get(b"curl_multi_waitfds\0")? },

      // Slist
      slist_append: unsafe { lib_static.get(b"curl_slist_append\0")? },
      slist_free_all: unsafe { lib_static.get(b"curl_slist_free_all\0")? },

      // MIME
      mime_init: unsafe { lib_static.get(b"curl_mime_init\0")? },
      mime_free: unsafe { lib_static.get(b"curl_mime_free\0")? },
      mime_addpart: unsafe { lib_static.get(b"curl_mime_addpart\0")? },
      mime_name: unsafe { lib_static.get(b"curl_mime_name\0")? },
      mime_data: unsafe { lib_static.get(b"curl_mime_data\0")? },
      mime_data_cb: unsafe { lib_static.get(b"curl_mime_data_cb\0")? },
      mime_encoder: unsafe { lib_static.get(b"curl_mime_encoder\0")? },
      mime_filedata: unsafe { lib_static.get(b"curl_mime_filedata\0")? },
      mime_filename: unsafe { lib_static.get(b"curl_mime_filename\0")? },
      mime_headers: unsafe { lib_static.get(b"curl_mime_headers\0")? },
      mime_subparts: unsafe { lib_static.get(b"curl_mime_subparts\0")? },
      mime_type: unsafe { lib_static.get(b"curl_mime_type\0")? },

      // Form API
      formadd: unsafe { lib_static.get(b"curl_formadd\0")? },
      formfree: unsafe { lib_static.get(b"curl_formfree\0")? },
      formget: unsafe { lib_static.get(b"curl_formget\0")? },

      // Share API
      share_init: unsafe { lib_static.get(b"curl_share_init\0")? },
      share_cleanup: unsafe { lib_static.get(b"curl_share_cleanup\0")? },
      share_setopt: unsafe { lib_static.get(b"curl_share_setopt\0")? },
      share_strerror: unsafe { lib_static.get(b"curl_share_strerror\0")? },

      // URL API
      url: unsafe { lib_static.get(b"curl_url\0")? },
      url_cleanup: unsafe { lib_static.get(b"curl_url_cleanup\0")? },
      url_dup: unsafe { lib_static.get(b"curl_url_dup\0")? },
      url_set: unsafe { lib_static.get(b"curl_url_set\0")? },
      url_get: unsafe { lib_static.get(b"curl_url_get\0")? },
      url_strerror: unsafe { lib_static.get(b"curl_url_strerror\0")? },

      // Global functions
      global_init: unsafe { lib_static.get(b"curl_global_init\0")? },
      global_init_mem: unsafe { lib_static.get(b"curl_global_init_mem\0")? },
      global_cleanup: unsafe { lib_static.get(b"curl_global_cleanup\0")? },
      global_sslset: unsafe { lib_static.get(b"curl_global_sslset\0")? },
      global_trace: unsafe { lib_static.get(b"curl_global_trace\0")? },
      version: unsafe { lib_static.get(b"curl_version\0")? },
      version_info: unsafe { lib_static.get(b"curl_version_info\0")? },
      escape: unsafe { lib_static.get(b"curl_escape\0")? },
      unescape: unsafe { lib_static.get(b"curl_unescape\0")? },
      free: unsafe { lib_static.get(b"curl_free\0")? },

      // String processing functions
      strequal: unsafe { lib_static.get(b"curl_strequal\0")? },
      strnequal: unsafe { lib_static.get(b"curl_strnequal\0")? },

      // Time functions
      getdate: unsafe { lib_static.get(b"curl_getdate\0")? },

      // Environment functions
      getenv: unsafe { lib_static.get(b"curl_getenv\0")? },

      // printf series functions
      maprintf: unsafe { lib_static.get(b"curl_maprintf\0")? },
      mfprintf: unsafe { lib_static.get(b"curl_mfprintf\0")? },
      mprintf: unsafe { lib_static.get(b"curl_mprintf\0")? },
      msnprintf: unsafe { lib_static.get(b"curl_msnprintf\0")? },
      msprintf: unsafe { lib_static.get(b"curl_msprintf\0")? },
      mvaprintf: unsafe { lib_static.get(b"curl_mvaprintf\0")? },
      mvfprintf: unsafe { lib_static.get(b"curl_mvfprintf\0")? },
      mvprintf: unsafe { lib_static.get(b"curl_mvprintf\0")? },
      mvsnprintf: unsafe { lib_static.get(b"curl_mvsnprintf\0")? },
      mvsprintf: unsafe { lib_static.get(b"curl_mvsprintf\0")? },

      // WebSocket API
      ws_meta: unsafe { lib_static.get(b"curl_ws_meta\0")? },
      ws_recv: unsafe { lib_static.get(b"curl_ws_recv\0")? },
      ws_send: unsafe { lib_static.get(b"curl_ws_send\0")? },

      // Push header functions
      pushheader_byname: unsafe { lib_static.get(b"curl_pushheader_byname\0")? },
      pushheader_bynum: unsafe { lib_static.get(b"curl_pushheader_bynum\0")? },
    };

    Ok(functions)
  })
}

// Get loaded function instances (if already loaded)
pub fn get_curl_functions() -> Option<&'static CurlFunctions> {
  CURL_FUNCTIONS.get()
}

// Check if library is already loaded
pub fn is_library_loaded() -> bool {
  CURL_FUNCTIONS.get().is_some()
}

pub fn napi_load_library() -> napi::Result<&'static CurlFunctions> {
  load_curl_library().map_err(|e| {
    Error::new(
      Status::GenericFailure,
      format!(
        "Failed to load @taskylizard/libcurl library. Path: {}, Error: {}",
        get_lib_path().unwrap_or("unknown path".to_string()),
        e
      ),
    )
  })
}

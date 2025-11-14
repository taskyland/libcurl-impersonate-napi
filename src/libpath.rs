use napi_derive::napi;
use once_cell::sync::Lazy;
use std::sync::RwLock;

static LIB_PATH: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

#[napi]
pub fn set_lib_path(path: String) {
  if let Ok(mut lib_path) = LIB_PATH.write() {
    *lib_path = Some(path);
  }
}

#[napi]
pub fn get_lib_path() -> Option<String> {
  LIB_PATH.read().ok()?.clone()
}

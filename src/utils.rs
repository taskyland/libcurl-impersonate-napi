use napi_derive::napi;
use std::path::Path;

pub fn get_ptr_address<T>(ptr: *const T) -> String {
  format!("0x{:x}", ptr as usize)
}

#[napi]
pub fn get_default_dir_name() -> String {
  let arch = match std::env::consts::ARCH {
    "x86_64" => "x86_64",
    "arm64" => "arm64",
    "arm" => "arm-linux-gnueabihf",
    "riscv64" => "riscv64",
    "i386" => "i386",
    "ia32" => "i686",
    other => other,
  };

  let platform = match std::env::consts::OS {
    "linux" => "linux-gnu",
    "macos" => "macos",
    "windows" => "win32",
    other => other,
  };

  format!("{}-{}", arch, platform)
}

#[napi]
pub fn get_default_lib_path(base: Option<String>) -> String {
  let base_dir = Path::new(&base.unwrap_or("".to_string())).join("libs");

  let dir_name = get_default_dir_name();

  let lib_name = match std::env::consts::OS {
    "windows" => Path::new("bin").join("libcurl.dll"),
    "macos" => Path::new("libcurl-impersonate.dylib").to_path_buf(),
    "linux" => Path::new("libcurl-impersonate.so").to_path_buf(),
    _ => Path::new("libcurl-impersonate.so").to_path_buf(),
  };

  base_dir
    .join(dir_name)
    .join(lib_name)
    .to_string_lossy()
    .to_string()
}

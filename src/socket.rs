use napi_derive::napi;

#[cfg(unix)]
use std::os::unix::io::RawFd;

#[napi]
pub fn socket_is_readable(sockfd: i32) -> bool {
  check_readable(sockfd)
}

#[napi]
pub fn socket_is_writable(sockfd: i32) -> bool {
  check_writable(sockfd)
}

#[cfg(unix)]
fn check_readable(sockfd: i32) -> bool {
  use std::mem;
  use std::ptr;

  unsafe {
    let mut read_fds: libc::fd_set = mem::zeroed();
    libc::FD_ZERO(&mut read_fds);
    libc::FD_SET(sockfd, &mut read_fds);

    let mut timeout = libc::timeval {
      tv_sec: 0,
      tv_usec: 0,
    };

    let result = libc::select(
      sockfd + 1,
      &mut read_fds,
      ptr::null_mut(),
      ptr::null_mut(),
      &mut timeout,
    );

    result > 0 && libc::FD_ISSET(sockfd, &read_fds)
  }
}

#[cfg(unix)]
fn check_writable(sockfd: i32) -> bool {
  use std::mem;
  use std::ptr;
  unsafe {
    let mut write_fds: libc::fd_set = mem::zeroed();
    libc::FD_ZERO(&mut write_fds);
    libc::FD_SET(sockfd, &mut write_fds);

    let mut timeout = libc::timeval {
      tv_sec: 0,
      tv_usec: 0,
    };

    let result = libc::select(
      sockfd + 1,
      ptr::null_mut(),
      &mut write_fds,
      ptr::null_mut(),
      &mut timeout,
    );

    result > 0 && libc::FD_ISSET(sockfd, &write_fds)
  }
}

#[cfg(windows)]
fn check_readable(sockfd: i32) -> bool {
  use std::mem;
  use std::ptr;
  use winapi::um::winsock2::{fd_set, select, timeval, SOCKET};

  unsafe {
    let mut read_fds: fd_set = mem::zeroed();
    read_fds.fd_count = 1;
    read_fds.fd_array[0] = sockfd as SOCKET;

    let mut timeout = timeval {
      tv_sec: 0,
      tv_usec: 0,
    };

    let result = select(
      0, // ignored on Windows
      &mut read_fds,
      ptr::null_mut(),
      ptr::null_mut(),
      &mut timeout,
    );

    result > 0
  }
}

#[cfg(windows)]
fn check_writable(sockfd: i32) -> bool {
  use std::mem;
  use std::ptr;
  use winapi::um::winsock2::{fd_set, select, timeval, SOCKET};

  unsafe {
    let mut write_fds: fd_set = mem::zeroed();
    write_fds.fd_count = 1;
    write_fds.fd_array[0] = sockfd as SOCKET;

    let mut timeout = timeval {
      tv_sec: 0,
      tv_usec: 0,
    };

    let result = select(
      0, // ignored on Windows
      ptr::null_mut(),
      &mut write_fds,
      ptr::null_mut(),
      &mut timeout,
    );

    result > 0
  }
}

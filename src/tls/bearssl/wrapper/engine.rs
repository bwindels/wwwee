use super::ffi::*;
use std;
use std::os::raw::{c_int, c_void};
use super::Result;

pub type Context = br_ssl_engine_context;

impl Context {
  pub fn recvrec_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_recvrec_buf(
        self as *const Context,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn recvrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvrec_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn sendrec_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_sendrec_buf(
        self as *const Context,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn sendrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_sendrec_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn recvapp_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_recvapp_buf(
        self as *const Context,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn recvapp_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvapp_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn sendapp_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_sendapp_buf(
        self as *const Context,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn sendapp_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_sendapp_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn flush(&mut self, force: bool) {
    let force : c_int = if force {1} else {0};
    unsafe {
      br_ssl_engine_flush(self as *mut Context, force)
    }
  }

  pub fn close(&mut self) {
    unsafe {
      br_ssl_engine_close(self as *mut Context)
    }
  }

  pub fn set_buffer<'a,'b:'a>(&'a mut self, buffer: &'b mut [u8], bidi: bool) {
    let bidi = if bidi {1} else {0};
    unsafe {
      br_ssl_engine_set_buffer(
        self as *mut Context,
        buffer.as_ptr() as *mut c_void,
        buffer.len(),
        bidi)
    }
  }

  pub fn last_error(&self) -> Result<()> {
    if self.err == BR_ERR_OK as i32 {
      Ok(())
    }
    else {
      let err = unsafe { std::mem::transmute(self.err as i16) };
      Err(err)
    }
  }
}

fn ptr_to_slice<'a>(ptr: *mut std::os::raw::c_uchar, len: usize) -> Option<&'a mut [u8]> {
  not_null_mut(ptr).map(|ptr| {
    unsafe {
      std::slice::from_raw_parts_mut(ptr as *mut u8, len)
    }
  })
}

fn not_null_mut<T>(ptr: *mut T) -> Option<*mut T> {
  if ptr.is_null() {
    None
  }
  else {
    Some(ptr)
  }
}

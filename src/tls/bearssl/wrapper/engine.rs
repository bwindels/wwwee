use super::ffi::*;
use std;
use std::slice;
use std::os::raw::{c_int, c_void};
use super::Result;

pub type Context = br_ssl_engine_context;

impl Context {
  pub fn recvrec_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    unsafe {
      let ptr = br_ssl_engine_recvrec_buf(
          self as *const Context,
          &mut size as *mut usize);
      ptr_to_slice(ptr, size)
    }
  }

  pub fn recvrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvrec_ack(self as *mut Context, len)
    }
    self.last_error()
  }
  //TODO: self should be mut here, returning a mutable ref
  pub fn sendrec_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    unsafe {
      let ptr = br_ssl_engine_sendrec_buf(
        self as *const Context,
        &mut size as *mut usize);
      ptr_to_slice(ptr, size)
    }
  }

  pub fn sendrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_sendrec_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn recvapp_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    unsafe {
      let ptr = br_ssl_engine_recvapp_buf(
        self as *const Context,
        &mut size as *mut usize);
      ptr_to_slice(ptr, size)
    }
  }

  pub fn recvapp_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvapp_ack(self as *mut Context, len)
    }
    self.last_error()
  }

  pub fn sendapp_buf<'a>(&'a self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    unsafe {
      let ptr = br_ssl_engine_sendapp_buf(
        self as *const Context,
        &mut size as *mut usize);
      ptr_to_slice(ptr, size)
    }
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

  pub fn state(&self) -> State {
    let state = unsafe {
      br_ssl_engine_current_state(self as *const Context)
    };
    State(state)
  }

  pub fn close(&mut self) {
    unsafe {
      br_ssl_engine_close(self as *mut Context)
    }
  }

  pub fn is_closed(&self) -> bool {
    unsafe {
      br_ssl_engine_current_state(self as *const Context) == BR_SSL_CLOSED
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
      Err(super::Error::from_primitive(self.err as u32))
    }
  }

  pub fn get_server_name<'a>(&'a self) -> Option<&'a [u8]> {
    let ptr = &self.server_name as *const i8;
    let ptr = if ptr.is_null() { None } else { Some(ptr) };
    unsafe {
      ptr.map(|ptr| {
        let len = libc::strnlen(ptr, 256);
        slice::from_raw_parts(ptr as *const u8, len)
      })
    }
  }
}

#[derive(Clone, Copy)]
pub struct State(u32);

impl State {
  pub fn includes(self, flag: StateFlag) -> bool {
    (self.0 & flag as u32) != 0
  }
}

impl IntoIterator for State {
  type Item = StateFlag;
  type IntoIter = StateIterator;

  fn into_iter(self) -> Self::IntoIter {
    StateIterator(0u8, self.0)
  }
}

pub struct StateIterator(u8, u32);
impl Iterator for StateIterator {
  type Item = StateFlag;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let idx = self.0;
      let state = self.1;
      self.0 += 1;
      match idx {
        0 => if (state & BR_SSL_CLOSED) != 0 {
          return Some(StateFlag::Closed);
        },
        1 => if (state & BR_SSL_SENDREC) != 0 {
          return Some(StateFlag::SendRec);
        },
        2 => if (state & BR_SSL_RECVREC) != 0 {
          return Some(StateFlag::RecvRec);
        },
        3 => if (state & BR_SSL_SENDAPP) != 0 {
          return Some(StateFlag::SendApp);
        },
        4 => if (state & BR_SSL_RECVAPP) != 0 {
          return Some(StateFlag::RecvApp);
        },
        _ => return None
      }
      
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum StateFlag {
  Closed = BR_SSL_CLOSED as isize,
  SendRec = BR_SSL_SENDREC as isize,
  RecvRec = BR_SSL_RECVREC as isize,
  SendApp = BR_SSL_SENDAPP as isize,
  RecvApp = BR_SSL_RECVAPP as isize
}

unsafe fn ptr_to_slice<'a>(ptr: *mut std::os::raw::c_uchar, len: usize) -> Option<&'a mut [u8]> {
  ptr.as_mut().map(|ptr| slice::from_raw_parts_mut(ptr as *mut u8, len))
}


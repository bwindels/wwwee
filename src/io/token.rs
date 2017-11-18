pub type ConnectionId = u32;
#[cfg(target_pointer_width = "64")]
pub type AsyncToken = u32;
#[cfg(target_pointer_width = "32")]
pub type AsyncToken = u16;

#[cfg(target_pointer_width = "64")]
pub fn split_token(token: usize) -> (ConnectionId, AsyncToken) {
  let conn_id = (token >> 32) as u32;
  let async_token = (token & 0xFFFFFFFF) as u32;
  (conn_id, async_token)
}
#[cfg(target_pointer_width = "32")]
pub fn split_token(token: usize) -> (ConnectionId, AsyncToken) {
  let conn_id = token >> 10;
  let async_token = token & 0b11_1111_1111;
  (conn_id, async_token)
}

#[cfg(target_pointer_width = "64")]
pub fn create_token(conn_id: ConnectionId, async_token: AsyncToken) -> usize {
  (async_token as usize) | ((conn_id as usize) << 32)
}
#[cfg(target_pointer_width = "32")]
pub fn create_token(conn_id: ConnectionId, async_token: AsyncToken) -> usize {
  (async_token as usize) | ((conn_id as usize) << 22)
}

#[cfg(test)]
mod tests {
  use super::{create_token, split_token};

  #[test]
  fn test_split() {
    let token : usize = create_token(5, 2);
    let (conn_id, async_token) = split_token(token);
    assert_eq!(conn_id, 5);
    assert_eq!(async_token, 2);
  }

  #[test]
  fn test_max_async_token() {
    let token : usize = create_token(5, 1024);
    let (conn_id, async_token) = split_token(token);
    assert_eq!(conn_id, 5);
    assert_eq!(async_token, 1024);
  }
}

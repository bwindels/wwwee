type ConnectionId = u32;
#[cfg(target_pointer_width = "64")]
type AsyncToken = u32;
#[cfg(target_pointer_width = "32")]
type AsyncToken = u8;

#[cfg(target_pointer_width = "64")]
fn split_token(token: usize) -> (ConnectionId, AsyncToken) {
  let conn_id = (token >> 32) as u32;
  let async_token = (token & 0xFFFFFFFF) as u32;
  (conn_id, async_token)
}
#[cfg(target_pointer_width = "32")]
fn split_token(token: usize) -> (ConnectionId, AsyncToken) {
  let conn_id = token >> 8;
  let async_token = token & 0xFF;
  (conn_id, async_token)
}

#[cfg(target_pointer_width = "64")]
fn create_token(conn_id: ConnectionId, async_token: AsyncToken) -> usize {
  (async_token as usize) | ((conn_id as usize) << 32)
}
#[cfg(target_pointer_width = "32")]
fn create_token(conn_id: ConnectionId, async_token: AsyncToken) -> usize {
  (async_token as usize) | ((conn_id as usize) << 24)
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
}

type ConnectionToken = u32;
#[cfg(target_pointer_width = "64")]
type AsyncToken = u32;
#[cfg(target_pointer_width = "32")]
type AsyncToken = u8;

#[cfg(target_pointer_width = "64")]
fn split_token(token: usize) -> (ConnectionToken, AsyncToken) {
  let connection_idx = token >> 32;
  let async_handle_idx = token & 0xFFFFFFFF;
  (connection_idx, async_handle_idx)
}
#[cfg(target_pointer_width = "32")]
fn split_token(token: usize) -> (ConnectionToken, AsyncToken) {
  let connection_idx = token >> 8;
  let async_handle_idx = token & 0xFF;
  (connection_idx, async_handle_idx)
}

#[cfg(target_pointer_width = "64")]
fn create_token(conn_token: ConnectionToken, async_token: AsyncToken) -> usize {
  async_token as usize & ((conn_token as usize) << 32)
}
#[cfg(target_pointer_width = "32")]
fn create_token(conn_token: ConnectionToken, async_token: AsyncToken) -> usize {
  async_token as usize & ((conn_token as usize) << 24)
}

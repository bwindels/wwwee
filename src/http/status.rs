pub type Status = (u16, &'static str);
pub const OK:                     Status = (200, "OK");
pub const BAD_REQUEST:            Status = (400, "Bad request");
pub const INTERNAL_SERVER_ERROR:  Status = (500, "Internal server error");

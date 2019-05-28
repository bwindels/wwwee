pub type Status = (u16, &'static str);
pub const OK:                     Status = (200, "OK");
pub const NOT_MODIFIED:           Status = (304, "Not Modified");
pub const BAD_REQUEST:            Status = (400, "Bad Request");
pub const UNAUTHORIZED:           Status = (401, "Unauthorized");
pub const NOT_FOUND:              Status = (404, "Not Found");
pub const INTERNAL_SERVER_ERROR:  Status = (500, "Internal Server Error");

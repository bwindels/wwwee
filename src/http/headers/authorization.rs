use http::RequestResult;

pub struct BasicAuth<'a> {
	user: &'a str,
	password: &'a str
}

/* pub enum Authorization<'a> {
  Basic(BasicAuth<'a>),
  Digest(&'a str),
  Bearer(&'a str)
} */

pub struct Authorization<'a> {
	authorization: &'a str
}

impl<'a> Authorization<'a> {
	pub fn parse(header_value: &'a str) -> RequestResult<Authorization<'a>> {
		Ok(Authorization {authorization: header_value})
	}
}

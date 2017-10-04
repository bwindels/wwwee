use http::RequestResult;

pub struct ContentRange<'a> {
	range: &'a str
}

impl<'a> ContentRange<'a> {
	pub fn parse(header_value: &'a str) -> RequestResult<ContentRange<'a>> {
		Ok(ContentRange {range: header_value})
	}
}

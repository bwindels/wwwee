/// NEED TO CLEAN THIS UP, POINT IS TO EXPLAIN DESIGN DECISIONS IN THIS MODULE
/// This problem is turning out to be more complicated than expected
/// given the limitation of only stack allocated memory.
///
/// This class would implement an iterator. Parameters names and values
/// can be url encoded because they can contain the delimiters (= and &)
/// Once the whole query string is decoded, we can't destinguish = from %3D
/// and & from %26 anymore. So we could decode on the fly, but then we have
/// the same problem the second time we iterator over the params.
/// We could have an internal buffer to the array, to which 
/// we copy name and value, and decode then but then the iterator value 
/// is only valid for one iteration, too short to be ergononic.
///
/// Consider this query string:
///
/// | f | o | o | = | % | 2 | 6 | b | a | r | & | b | = | 2 |   
/// ^           ^   ^                       ^   ^   ^   ^   ^
///
/// We decode it to this, and returning an error if the decoded message contains \0:
///
/// | f | o | o | = | 0 | & | b | a | r | 0 | & | b | = | 2 |   
/// ^           ^       ^               ^       ^   ^   ^   ^
/// if the name or value starts with \0, the delimiter is only valid if preceeded by \0
/// this has the disadvantage of not knowing where we need to start writing in url_decode,
/// only when we encounter a url encoded value should we start writing at offset 1, of not at offset 0
/// so once we encounter it, we'd have to move all the text preceeding it from offset 0 to offset 1
///
/// this is still the most elegant solution I think.
/// The only limitation to the API is that you can't use \0 in the query string.
/// We'd have to send Bad Request or something.
///
/// we could first loop through the whole byte array per component to see if it contains any %.
/// if it does we set the offset to 1, if not to 0 and we don't need to copy anything.
///
/// #Irregular query strings
///
///  - foo&bar=foo
///  - foo=
///  - foo=&bar
///  - foo=bar=&bar&&
///   ^___^___^^___^
///  - 
///  - foo&=bar


mod decode;
mod parse_decoded;
mod iterator;

pub use self::iterator::{UrlEncodedParams, UrlEncodedParamsIterator};
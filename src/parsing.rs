
use std::str::Chars;
use renounce::*;

pat!(pub(crate) parse_any<'a>: char => char = x => x);
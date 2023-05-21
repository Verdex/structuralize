
use std::str::Chars;
use renounce::*;

pat!(pub(crate) parse_any<'a>: char => char = x => x);

pub (crate) fn parse_whitespace<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
    fn space<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!( input => {
            x <= parse_any;
            where x.is_whitespace();
            select ()
        })
    }

    parser!( input => {
        _x <= * space;
        select ()
    })
}


use std::str::Chars;
use renounce::*;

macro_rules! parse_list {
    ($input:ident => $l_bracket:ident, $target:ident : $target_type:ty, $r_bracket:ident) => {
        {
            pat!(parse_comma: char => () = ',' => ());

            fn parse_target_comma<'a>(input : &mut Chars<'a>) -> Result<$target_type, ParseError> {
                parser!(input => {
                    _clear_0 <= parse_whitespace;
                    target <= $target;
                    _clear_1 <= parse_whitespace;
                    _comma <= parse_comma;
                    _clear_2 <= parse_whitespace;
                    select target 
                })
            }

            parser!($input => {
                _clear_0 <= parse_whitespace;
                _left_bracket <= $l_bracket;
                targets <= * parse_target_comma;
                last_target <= ? $target;
                _clear_1 <= parse_whitespace;
                _right_bracket <= ! $r_bracket;
                _clear_2 <= parse_whitespace;
                select {
                    let mut targets = targets;
                    match last_target {
                        Some(target) => { targets.push(target); },
                        None => (),
                    }
                    targets
                }
            })
        }
    };
}

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

pub (crate) use parse_list;
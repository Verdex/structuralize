
use std::str::Chars;
use renounce::*;

macro_rules! parse_list {
    ($input:ident => $l_bracket:ident, $target:ident : $target_type:ty, $r_bracket:ident) => {
        {
            pat!(parse_comma: char => () = ',' => ());

            fn parse_target_comma(input : &mut Chars) -> Result<$target_type, ParseError> {
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
                _clear_1 <= parse_whitespace;
                last_target <= ? $target;
                _clear_2 <= parse_whitespace;
                _right_bracket <= ! $r_bracket;
                _clear_3 <= parse_whitespace;
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

pub (crate) fn parse_whitespace(input : &mut Chars) -> Result<(), ParseError> {
    fn space(input : &mut Chars) -> Result<(), ParseError> {
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

pub (crate) fn parse_digit(input : &mut Chars) -> Result<char, ParseError> {
    parser!(input => {
        num <= parse_any;
        where num.is_digit(10);
        select num
    })
}

pub (crate) fn parse_word(input : &mut Chars) -> Result<Box<str>, ParseError> {
    pat!(underscore: char => char = '_' => '_');

    fn parse_alpha(input : &mut Chars) -> Result<char, ParseError> {
        parser!(input => {
            init_symbol <= parse_any;
            where init_symbol.is_alphabetic();
            select init_symbol
        })
    }

    fn parse_init(input : &mut Chars) -> Result<char, ParseError> {
        alt!(input => parse_alpha; underscore)
    }

    fn parse_symbol_char(input : &mut Chars) -> Result<char, ParseError> {
        alt!(input => parse_alpha; parse_digit; underscore)
    }

    parser!(input => {
        init <= parse_init;
        rest <= * parse_symbol_char;
        select {
            let mut rest = rest;
            rest.insert(0, init);
            rest.into_iter().collect::<String>().into()
        } 
    })
}

pub (crate) fn parse_string(input : &mut Chars) -> Result<Box<str>, ParseError> {
    pat!(parse_n: char => char = 'n' => '\n');
    pat!(parse_r: char => char = 'r' => '\r');
    pat!(parse_t: char => char = 't' => '\t');
    pat!(parse_slash: char => char = '\\' => '\\');
    pat!(parse_zero: char => char = '0' => '\0');
    pat!(parse_quote: char => char = '"' => '"');

    fn parse_code(input : &mut Chars) -> Result<char, ParseError> {
        alt!(input => parse_n; parse_r; parse_t; parse_slash; parse_zero; parse_quote)
    }

    fn parse_escape(input : &mut Chars) -> Result<char, ParseError> {
        parser!(input => {
            _slash <= parse_slash;
            code <= ! parse_code;
            select code
        })
    }

    fn parse_any_but_quote(input : &mut Chars) -> Result<char, ParseError> {
        parser!(input => {
            any <= parse_any;
            where any != '"';
            select any
        })
    }

    fn parse_str_char(input : &mut Chars) -> Result<char, ParseError> {
        alt!(input => parse_escape; parse_any_but_quote)
    }

    parser!(input => {
        _start_quote <= parse_quote;
        str_chars <= * parse_str_char;
        _end_quote <= parse_quote;
        select str_chars.into_iter().collect::<String>().into()
    })
}

pub (crate) use parse_list;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_empty_string() {
        let mut input = "\"\"".chars();
        let output = parse_string(&mut input).unwrap();
        assert_eq!(*output, *"");
    }

    #[test]
    fn should_parse_string() {
        let input = "\"blah \\t \\n \\r \\\\ \\0 \\\"  \"";
        let mut cs = input.chars();
        let output = parse_string(&mut cs).unwrap();
        assert_eq!(*output, *"blah \t \n \r \\ \0 \"  ");
    }
}
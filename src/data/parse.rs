
use std::str::Chars;

use renounce::*;

use super::data::*;

macro_rules! parse_list {
    ($input:ident => $l_bracket:ident, $target:ident : $target_type:ty, $r_bracket:ident) => {
        {
            pat!(parse_comma: char => () = ',' => ());

            fn parse_target_comma<'a>(input : &mut Chars<'a>) -> Result<$target_type, ParseError> {
                parser!(input => {
                    target <= $target;
                    _clear <= junk;
                    _comma <= parse_comma;
                    select target 
                })
            }

            parser!($input => {
                _clear_0 <= junk;
                _left_bracket <= $l_bracket;
                _clear_1 <= junk;
                targets <= * parse_target_comma;
                _clear_2 <= junk;
                last_target <= ? $target;
                _clear_3 <= junk;
                _right_bracket <= ! $r_bracket;
                _clear_4 <= junk;
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

    ($input:ident => $l_bracket:ident, $target:ident, $r_bracket:ident) => {
        parse_list!($input => $l_bracket, $target : Data, $r_bracket)
    };
}

impl std::str::FromStr for Data {
    type Err = Box<dyn std::error::Error>;  

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        // TODO:  the Chars struct will be returned at the point of failure
        // in the event of a Fatal result.  Might be a good idea to somehow
        // reflect that in a different concrete Error so that it can be shown
        // to a consumer.
        Ok(parse_data(&mut s.chars())?)
    }
}

fn parse_data<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    fn options<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
        alt!(input => parse_float64; 
                      parse_cons; 
                      parse_struct;
                      parse_list; 
                      parse_symbol)
    }

    parser!(input => {
        _before_clear <= junk;
        data <= options;
        _after_clear <= junk;
        select data
    })
}

pat!(any<'a>: char => char = x => x);

fn number<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
    parser!(input => {
        num <= any;
        where num.is_digit(10);
        select num
    })
}

fn junk<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
    pat!(any: char => char = x => x);
    fn space<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!( input => {
            x <= any;
            where x.is_whitespace();
            select ()
        })
    }

    parser!( input => {
        _x <= * space;
        select ()
    })
}

fn parse_word<'a>(input : &mut Chars<'a>) -> Result<String, ParseError> {
    pat!(underscore: char => char = '_' => '_');

    fn parse_alpha<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        parser!(input => {
            init_symbol <= any;
            where init_symbol.is_alphabetic();
            select init_symbol
        })
    }

    fn parse_init<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        alt!(input => parse_alpha; underscore)
    }

    fn parse_symbol_char<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        alt!(input => parse_alpha; number; underscore)
    }

    parser!(input => {
        init <= parse_init;
        rest <= * parse_symbol_char;
        select {
            let mut rest = rest;
            rest.insert(0, init);
            rest.into_iter().collect::<String>()
        } 
    })
}

fn parse_struct<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_paren: char => () = '{' => ());
    pat!(parse_r_paren: char => () = '}' => ());
    pat!(parse_colon: char => () = ':' => ());

    fn parse_field<'a>(input : &mut Chars<'a>) -> Result<(String, Data), ParseError> {
        parser!(input => {
            field_name <= parse_word;
            _clear_1 <= junk;
            _colon <= ! parse_colon;
            // Note: parse_data clears before and after itself
            data <= parse_data; 
            select (field_name, data)
        })
    }

    fn parse_fields<'a>(input : &mut Chars<'a>) -> Result<Vec<(String, Data)>, ParseError> {
        parse_list!(input => parse_l_paren, parse_field : (String, Data), parse_r_paren)
    }

    parser!(input => {
        struct_name <= parse_word;
        _clear <= junk;
        fields <= parse_fields;
        select Data::Struct { name: struct_name, fields }
    })
}

fn parse_cons<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn param_list<'a>(input : &mut Chars<'a>) -> Result<Vec<Data>, ParseError> {
        parse_list!(input => parse_l_paren, parse_data, parse_r_paren)
    }

    parser!(input => {
        cons_name <= parse_word;
        _clear <= junk;
        params <= param_list;
        select Data::Cons { name: cons_name, params }
    })
}

fn parse_symbol<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    parser!(input => {
        word <= parse_word;
        select Data::Symbol(word)
    })
}

fn parse_float64<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(lower_e: char => char = 'e' => 'e');
    pat!(upper_e: char => char = 'E' => 'E');
    pat!(minus: char => char = '-' => '-');
    pat!(plus: char => char = '+' => '+');
    pat!(dot: char => char = '.' => '.');

    fn parse_num_char<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        alt!(input => number; dot; minus; plus; lower_e; upper_e)
    }

    fn parse_init_num_char<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        alt!(input => number; minus)
    }

    fn combine(c : char, mut v : Vec<char>) -> Vec<char> { v.insert(0, c); v }

    parser!(input => {
        initial <= parse_init_num_char;
        num_chars <= * parse_num_char;
        let result = combine(initial, num_chars).into_iter().collect::<String>().parse::<f64>();
        ! where result.is_ok();
        select Data::Number(Number::Float64(result.unwrap()))
    })
}

fn parse_list<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());

    Ok(Data::List(parse_list!(input => parse_l_square, parse_data, parse_r_square)?))
}


#[cfg(test)]
mod test {
    use intra::*;
    use super::*;

    fn slice<'a, T>(input : &'a Vec<T>) -> &'a [T] { &input[..] }
    fn unbox<'a, T>(input : &'a Box<T> ) -> &'a T { &**input }

    #[test]
    fn should_parse_struct() {
        let input = " name  { first : 1.0 , second: inner  }";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Struct { name, fields: ref fields}] fields; 
                       slice $ [ [(first, Data::Number(Number::Float64(f))), (second, Data::Symbol(sym))] ] => { 
            assert_eq!(*f, 1f64);
            assert_eq!(sym, "inner");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_cons() {
        let input = " name  ( 1.0, inner, 5.5 )";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Cons { name, params: ref params }] params; 
                       slice $ [ [Data::Number(Number::Float64(a)), Data::Symbol(_), Data::Number(Number::Float64(b))] ] => { 
            assert_eq!(*a, 1f64);
            assert_eq!(*b, 5.5f64);
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_symbol() {
        let input = " symbol_123 ";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Symbol(sym)] => { 
            assert_eq!(sym, "symbol_123");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_list() {
        let input = " [ [], [1, 2], [1 , 2, 3], 4] ";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::List(ref params)] params; 
              slice $ [ [Data::List(first), Data::List(second), Data::List(third), Data::Number(Number::Float64(f))] ] => { 
            assert_eq!(*f, 4f64);
            assert_eq!(first.len(), 0);
            assert_eq!(second.len(), 2);
            assert_eq!(second[0], Data::Number(Number::Float64(1f64)));
            assert_eq!(second[1], Data::Number(Number::Float64(2f64)));
            assert_eq!(third.len(), 3);
            assert_eq!(third[0], Data::Number(Number::Float64(1f64)));
            assert_eq!(third[1], Data::Number(Number::Float64(2f64)));
            assert_eq!(third[2], Data::Number(Number::Float64(3f64)));
            matched = true;
        } );

        assert!(matched);
    }

    #[test]
    fn should_parse_float64() {
        let input = "-123.456E-2";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Number(Number::Float64(x))] => { 
            assert_eq!(x, -123.456E-2);
            matched = true;
        } );

        assert!(matched);
    }
}
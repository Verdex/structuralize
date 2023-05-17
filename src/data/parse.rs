
use std::str::Chars;

use renounce::*;

use super::data::*;

impl std::str::FromStr for Data {
    type Err = Box<dyn std::error::Error>;  

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        Ok(parse_data(&mut s.chars())?)
    }
}

fn parse_data<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    fn options<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
        alt!(input => parse_float64)
    }

    parser!(input => {
        _before_clear <= junk;
        data <= options;
        _after_clear <= junk;
        select data
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

fn parse_symbol<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {

    parser!(input => {
        ! where false;
        select Data::Symbol("blarg".to_string())
    })
}

fn parse_float64<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(any: char => char = x => x);
    pat!(lower_e: char => char = 'e' => 'e');
    pat!(upper_e: char => char = 'E' => 'E');
    pat!(minus: char => char = '-' => '-');
    pat!(plus: char => char = '+' => '+');
    pat!(dot: char => char = '.' => '.');

    fn number<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        parser!(input => {
            num <= any;
            where num.is_digit(10);
            select num
        })
    }

    fn parse_num_char<'a>(input : &mut Chars<'a>) -> Result<char, ParseError> {
        alt!(input => number; dot; minus; plus; lower_e; upper_e)
    }

    parser!(input => {
        num_chars <= * parse_num_char;
        let result = num_chars.into_iter().collect::<String>().parse::<f64>();
        ! where result.is_ok();
        select Data::Number(Number::Float64(result.unwrap()))
    })
}

fn parse_list<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());
    pat!(parse_comma: char => () = ',' => ());

    fn parse_data_comma<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
        parser!(input => {
            data <= parse_data;
            _comma <= parse_comma;
            select data
        })
    }

    parser!(input => {
        _left_square_bracket <= parse_l_square;
        datas <= * parse_data_comma;
        last_data <= ? parse_data;
        _right_square_bracket <= ! parse_r_square;
        select {
            let mut datas = datas;
            match last_data {
                Some(data) => { datas.push(data); },
                None => (),
            }
            Data::List(datas)
        }
    })
}

#[cfg(test)]
mod test {
    use intra::*;
    use super::*;

    fn slice<'a, T>(input : &'a Vec<T>) -> &'a [T] { &input[..] }
    fn unbox<'a, T>(input : &'a Box<T> ) -> &'a T { &**input }

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
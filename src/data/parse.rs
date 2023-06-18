
use std::str::Chars;
use renounce::*;

use crate::parsing::*;
use super::data::*;

impl std::str::FromStr for Data {
    type Err = Box<dyn std::error::Error>;  

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        // TODO:  the Chars struct will be returned at the point of failure
        // in the event of a Fatal result.  Might be a good idea to somehow
        // reflect that in a different concrete Error so that it can be shown
        // to a consumer.
        let mut x = s.chars();
        let y = parse_data(&mut x);
        println!("{}", x.collect::<String>());
        Ok(y?)
        //Ok(parse_data(&mut s.chars())?)
    }
}

fn parse_data<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    fn options<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
        alt!(input => parse_data_float64; 
                      parse_cons; 
                      parse_struct;
                      parse_list; 
                      parse_symbol)
    }

    parser!(input => {
        _before_clear <= parse_whitespace;
        data <= options;
        _after_clear <= parse_whitespace;
        select data
    })
}

fn parse_struct<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_paren: char => () = '{' => ());
    pat!(parse_r_paren: char => () = '}' => ());
    pat!(parse_colon: char => () = ':' => ());

    fn parse_field<'a>(input : &mut Chars<'a>) -> Result<(Box<str>, Data), ParseError> {
        parser!(input => {
            field_name <= parse_word;
            _clear_1 <= parse_whitespace;
            _colon <= ! parse_colon;
            // Note: parse_data clears before and after itself
            data <= parse_data; 
            select (field_name, data)
        })
    }

    fn parse_fields<'a>(input : &mut Chars<'a>) -> Result<Vec<(Box<str>, Data)>, ParseError> {
        parse_list!(input => parse_l_paren, parse_field : (Box<str>, Data), parse_r_paren)
    }

    parser!(input => {
        struct_name <= parse_word;
        _clear <= parse_whitespace;
        fields <= parse_fields;
        select Data::Struct { name: struct_name, fields }
    })
}

fn parse_cons<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn param_list<'a>(input : &mut Chars<'a>) -> Result<Vec<Data>, ParseError> {
        parse_list!(input => parse_l_paren, parse_data : Data, parse_r_paren)
    }

    parser!(input => {
        cons_name <= parse_word;
        _clear <= parse_whitespace;
        params <= param_list;
        select Data::Cons { name: cons_name, params }
    })
}

fn parse_symbol<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_colon: char => () = ':' => ());
    parser!(input => {
        _colon <= parse_colon;
        word <= parse_word;
        select Data::Symbol(word)
    })
}

fn parse_data_float64<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    Ok(Data::Number(Number::Float64(parse_float64(input)?)))
}

fn parse_list<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());

    Ok(Data::List(parse_list!(input => parse_l_square, parse_data : Data, parse_r_square)?))
}


#[cfg(test)]
mod test {
    use intra::*;
    use super::*;

    fn slice<'a, T>(input : &'a Vec<T>) -> &'a [T] { &input[..] }
    fn unbox<'a, T>(input : &'a Box<T> ) -> &'a T { &**input }

    #[test]
    fn should_parse_complex_data() {
        let input = " name  { first : other { first : one( 1, 2, num([3, 2, 3, [:blarg]]) ) } , second: :inner ,  }";
        let data = input.parse::<Data>().unwrap();
        let mut matched = false;
        atom!(data => [Data::Struct { .. }] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_struct() {
        let input = " name  { first : 1.0 , second: :inner ,  }";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Struct { name, fields: ref fields}] fields; 
                       slice $ [ [(first, Data::Number(Number::Float64(f))), (second, Data::Symbol(sym))] ] => { 
            assert_eq!(*name, *"name");
            assert_eq!(*f, 1f64);
            assert_eq!(**sym, *"inner");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_cons() {
        let input = " name  ( 1.0, :inner, 5.5 )";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Cons { name, params: ref params }] params; 
                       slice $ [ [Data::Number(Number::Float64(a)), Data::Symbol(_), Data::Number(Number::Float64(b))] ] => { 
            assert_eq!(*name, *"name");
            assert_eq!(*a, 1f64);
            assert_eq!(*b, 5.5f64);
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_symbol() {
        let input = " :symbol_123 ";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Symbol(sym)] => { 
            assert_eq!(*sym, *"symbol_123");
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
    fn should_parse_data_float64() {
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
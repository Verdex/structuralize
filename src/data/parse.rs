
use std::str::Chars;
use renounce::*;

use crate::parsing::*;
use super::data::*;

#[derive(Debug)]
pub struct E(Box<str>);

impl std::fmt::Display for E {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            E(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for E { }

impl std::str::FromStr for Data {
    type Err = Box<dyn std::error::Error>;  

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let mut cs = s.chars();
        let result = parse(&mut cs);
        match result {
            Ok(v) => Ok(v),
            Err(ParseError::Fatal(x)) => Err(Box::new(E(format!("Error Trace: {:?}\nAt: {}", x, cs.collect::<String>()).into()))),
            Err(ParseError::Error) => Err(Box::new(E("Not Data".into()))),
        }
    }
}

fn parse<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    parser!(input => {
        data <= ! parse_data;
        ! end;
        select data
    })
}

fn parse_data<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    fn options<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
        alt!(input => parse_cons; 
                      parse_list; 
                      parse_symbol;
                      parse_string_data)
    }

    parser!(input => {
        _before_clear <= parse_whitespace;
        data <= options;
        _after_clear <= parse_whitespace;
        select data
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

fn parse_string_data<'a>(input : &mut Chars<'a>) -> Result<Data, ParseError> {
    parser!(input => {
        string <= parse_string;
        select Data::String(string)
    })
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

    #[test]
    fn should_parse_complex_data() {
        let input = " name  (  other ( one( :a, :b, num([:c, :d, :e, [:blarg]]) ) ) , :inner  )";
        let data = input.parse::<Data>().unwrap();
        let mut matched = false;
        atom!(data => [Data::Cons { .. }] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_cons() {
        let input = " name  ( :first, :inner, :last )";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::Cons { name, params: ref params }] params; 
                       slice $ [ [Data::Symbol(a), Data::Symbol(b), Data::Symbol(c)] ] => { 
            assert_eq!(*name, *"name");
            assert_eq!(**a, *"first");
            assert_eq!(**b, *"inner");
            assert_eq!(**c, *"last");
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
        let input = " [ [], [:a, :b], [:c , :d, :e], :f] ";
        let data = input.parse::<Data>().unwrap();

        let mut matched = false;
        atom!(data => [Data::List(ref params)] params; 
              slice $ [ [Data::List(first), Data::List(second), Data::List(third), Data::Symbol(last)] ] => { 
            assert_eq!(first.len(), 0);
            assert_eq!(second.len(), 2);
            assert_eq!(second[0], Data::Symbol("a".into()));
            assert_eq!(second[1], Data::Symbol("b".into()));
            assert_eq!(third.len(), 3);
            assert_eq!(third[0], Data::Symbol("c".into()));
            assert_eq!(third[1], Data::Symbol("d".into()));
            assert_eq!(third[2], Data::Symbol("e".into()));
            assert_eq!(**last, *"f");
            matched = true;
        } );

        assert!(matched);
    }
}
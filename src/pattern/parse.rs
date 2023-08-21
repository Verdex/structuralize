
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

impl std::str::FromStr for Pattern {
    type Err = Box<dyn std::error::Error>;  

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let mut cs = s.chars();
        let result = parse(&mut cs);
        match result {
            Ok(v) => Ok(v),
            Err(ParseError::Fatal(x)) => Err(Box::new(E(format!("Error Trace: {:?}\nAt: {}", x, cs.collect::<String>()).into()))),
            Err(ParseError::Error) => Err(Box::new(E("Not a Pattern".into()))),
        }
    }
}

fn parse<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    parser!(input => {
        pattern <= ! parse_pattern;
        ! end;
        select pattern 
    })
}

fn parse_pattern<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    fn options<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
        alt!(input => parse_data_float64; 
                      parse_cons; 
                      parse_struct;
                      parse_list_path;
                      parse_list; 
                      parse_wild;
                      // Note:  parse capture variable needs to happen after parse wild
                      parse_capture_var;
                      parse_path;
                      parse_path_next;
                      parse_symbol;
                      parse_string_pattern)
    }

    fn end_options<'a>(input : &mut Chars<'a>) -> Result<EndCombinator, ParseError> {
        alt!(input => parse_and; parse_or)
    }

    parser!(input => {
        _before_clear <= parse_whitespace;
        pattern <= options;
        _after_clear <= parse_whitespace;
        ends <= * end_options;
        _after_end_clear <= parse_whitespace;
        select ends.into_iter().fold(pattern, 
            |p, ec| match ec { 
                EndCombinator::And(e) => Pattern::And(Box::new(p), Box::new(e)),
                EndCombinator::Or(e) => Pattern::Or(Box::new(p), Box::new(e)),
            })
    })
}

enum EndCombinator {
    And(Pattern),
    Or(Pattern),
}

fn parse_or<'a>(input : &mut Chars<'a>) -> Result<EndCombinator, ParseError> {
    pat!(parse_orbar: char => () = '|' => ());
    pat!(parse_gt: char => () = '>' => ());
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn parse_pipe<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _orbar <= parse_orbar;
            _gt <= parse_gt;
            select ()
        })
    }

    fn parse_and<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            and <= parse_word;
            where *and == *"or";
            select ()
        })
    }

    parser!(input => {
        _ws0 <= parse_whitespace;
        _pipe <= parse_pipe;
        _ws1 <= parse_whitespace;
        _and <= parse_and;
        _ws2 <= parse_whitespace;
        _paren_l <= ! parse_l_paren;
        pattern <= ! parse_pattern;
        _paren_r <= ! parse_r_paren; 
        _ws3 <= parse_whitespace;
        select EndCombinator::Or(pattern)
    })
}

fn parse_and<'a>(input : &mut Chars<'a>) -> Result<EndCombinator, ParseError> {
    pat!(parse_orbar: char => () = '|' => ());
    pat!(parse_gt: char => () = '>' => ());
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn parse_pipe<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _orbar <= parse_orbar;
            _gt <= parse_gt;
            select ()
        })
    }

    fn parse_and<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            and <= parse_word;
            where *and == *"and";
            select ()
        })
    }

    parser!(input => {
        _ws0 <= parse_whitespace;
        _pipe <= parse_pipe;
        _ws1 <= parse_whitespace;
        _and <= parse_and;
        _ws2 <= parse_whitespace;
        _paren_l <= ! parse_l_paren;
        pattern <= ! parse_pattern;
        _paren_r <= ! parse_r_paren; 
        _ws3 <= parse_whitespace;
        select EndCombinator::And(pattern)
    })
}

fn parse_list_path<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());
    pat!(parse_bar: char => () = '|' => ());

    fn parse_l_bracket<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _square <= parse_l_square;
            _bar <= parse_bar;
            select ()
        })
    }

    fn parse_r_bracket<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _bar <= parse_bar;
            _square <= parse_r_square;
            select ()
        })
    }

    fn parse_points<'a>(input : &mut Chars<'a>) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_bracket, parse_pattern : Pattern, parse_r_bracket)
    }

    parser!(input => {
        points <= parse_points;
        select Pattern::ListPath(points)
    })
}

fn parse_path<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_l_curl: char => () = '{' => ());
    pat!(parse_r_curl: char => () = '}' => ());
    pat!(parse_bar: char => () = '|' => ());

    fn parse_l_bracket<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _curl <= parse_l_curl;
            _bar <= parse_bar;
            select ()
        })
    }

    fn parse_r_bracket<'a>(input : &mut Chars<'a>) -> Result<(), ParseError> {
        parser!(input => {
            _bar <= parse_bar;
            _curl <= parse_r_curl;
            select ()
        })
    }

    fn parse_points<'a>(input : &mut Chars<'a>) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_bracket, parse_pattern : Pattern, parse_r_bracket)
    }

    parser!(input => {
        points <= parse_points;
        select Pattern::Path(points)
    })
}

pat!(parse_path_next<'a>: char => Pattern = '^' => Pattern::PathNext);

fn parse_struct<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_l_curl: char => () = '{' => ());
    pat!(parse_r_curl: char => () = '}' => ());
    pat!(parse_colon: char => () = ':' => ());

    fn parse_field<'a>(input : &mut Chars<'a>) -> Result<(Box<str>, Pattern), ParseError> {
        parser!(input => {
            field_name <= parse_word;
            _clear_1 <= parse_whitespace;
            _colon <= ! parse_colon;
            // Note: parse_pattern clears before and after itself
            pattern <= parse_pattern; 
            select (field_name, pattern)
        })
    }

    fn parse_fields<'a>(input : &mut Chars<'a>) -> Result<Vec<(Box<str>, Pattern)>, ParseError> {
        parse_list!(input => parse_l_curl, parse_field : (Box<str>, Pattern), parse_r_curl)
    }

    parser!(input => {
        struct_name <= parse_word;
        _clear <= parse_whitespace;
        fields <= parse_fields;
        select Pattern::Struct { name: struct_name, fields }
    })
}

fn parse_cons<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn param_list<'a>(input : &mut Chars<'a>) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_paren, parse_pattern : Pattern, parse_r_paren)
    }

    parser!(input => {
        cons_name <= parse_word;
        _clear <= parse_whitespace;
        params <= param_list;
        select Pattern::Cons { name: cons_name, params }
    })
}

fn parse_capture_var<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    parser!(input => {
        word <= parse_word;
        select Pattern::CaptureVar(word)
    })
}

fn parse_wild<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    parser!(input => {
        word <= parse_word;
        where *word == *"_";
        select Pattern::Wild
    })
}

fn parse_symbol<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_colon: char => () = ':' => ());
    parser!(input => {
        _colon <= parse_colon;
        word <= parse_word;
        select Pattern::Symbol(word)
    })
}

fn parse_string_pattern<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    parser!(input => {
        string <= parse_string;
        select Pattern::String(string)
    })
}

fn parse_data_float64<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    use crate::data::Number;
    Ok(Pattern::Number(Number::Float64(parse_float64(input)?)))
}

fn parse_list<'a>(input : &mut Chars<'a>) -> Result<Pattern, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());

    Ok(Pattern::ExactList(parse_list!(input => parse_l_square, parse_pattern : Pattern, parse_r_square)?))
}


#[cfg(test)]
mod test {
    use intra::*;
    use super::*;
    use crate::data::Number;

    fn slice<'a, T>(input : &'a Vec<T>) -> &'a [T] { &input[..] }
    fn unbox<T>(input : Box<T> ) -> T { *input }

    fn get_float(p : &Pattern) -> Option<f64> {
        if let Pattern::Number(Number::Float64(x)) = p {
            Some(*x)
        }
        else {
            None
        }
    }

    #[test]
    fn should_parse_nested_ends() {
        let input = "[] |> and( :c |> or ( \"e\" |> or( 1.0 ) ) )";
        let data = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(data => [Pattern::And(a, b)] b; unbox $ [Pattern::Or(c, d)] d; unbox $ [Pattern::Or(e, f)] =>  {
            assert!( matches!( *a, Pattern::ExactList(_) ) );
            assert!( matches!( *c, Pattern::Symbol(_) ) );
            assert!( matches!( *e, Pattern::String(_) ) );
            assert!( matches!( *f, Pattern::Number(_) ) );
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_multiple_alternating_ends() {
        let input = "[] |> or ( 1.0 ) |> and ( :b ) |> or ( :c )";
        let data = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(data => [Pattern::Or(a, b)] a; unbox $ [Pattern::And(c, d)] c; unbox $ [Pattern::Or(e, f)] =>  {
            assert!( matches!( *b, Pattern::Symbol(_) ) );
            assert!( matches!( *d, Pattern::Symbol(_) ) );
            assert!( matches!( *e, Pattern::ExactList(_) ) );
            assert!( matches!( *f, Pattern::Number(_) ) );
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_or() {
        let input = ":a |> or ( 1.0 )";
        let data = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(data => [Pattern::Or(a, b)] =>  {
            assert!( matches!( *a, Pattern::Symbol(_) ) );
            assert!( matches!( *b, Pattern::Number(_) ) );
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_and() {
        let input = ":a |> and ( 1.0 )";
        let data = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(data => [Pattern::And(a, b)] =>  {
            assert!( matches!( *a, Pattern::Symbol(_) ) );
            assert!( matches!( *b, Pattern::Number(_) ) );
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_path() {
        let input = "{| a, b, c |}";
        let data = input.parse::<Pattern>().unwrap();
        assert!(matches!(data, Pattern::Path(_)));
    }

    #[test]
    fn should_parse_list_path() {
        let input = "[| a, b, c |]";
        let data = input.parse::<Pattern>().unwrap();
        assert!(matches!(data, Pattern::ListPath(_)));
    }

    #[test]
    fn should_parse_complex_data() {
        let input = " name  { first : other { first : one( 1, 2, num([3, 2, 3, [:blarg]]) ) } , second: :inner ,  }";
        let data = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(data => [Pattern::Struct { .. }] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_struct() {
        let input = " name  { first : 1.0 , second: :inner ,  }";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::Struct { name, fields: ref fields}] fields; 
                       slice $ [ [(first, Pattern::Number(Number::Float64(f))), (second, Pattern::Symbol(sym))] ] => { 
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
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::Cons { name, params: ref params }] params; 
                       slice $ [ [Pattern::Number(Number::Float64(a)), Pattern::Symbol(_), Pattern::Number(Number::Float64(b))] ] => { 
            assert_eq!(*name, *"name");
            assert_eq!(*a, 1f64);
            assert_eq!(*b, 5.5f64);
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_capture_var() {
        let input = " symbol_123 ";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::CaptureVar(sym)] => { 
            assert_eq!(*sym, *"symbol_123");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_symbol() {
        let input = " :symbol_123 ";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::Symbol(sym)] => { 
            assert_eq!(*sym, *"symbol_123");
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_wild() {
        let input = " _ ";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::Wild] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_exact_list() {
        let input = " [ [], [1, 2], [1 , 2, 3], 4] ";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::ExactList(ref params)] params; 
              slice $ [ [Pattern::ExactList(first), Pattern::ExactList(second), Pattern::ExactList(third), Pattern::Number(Number::Float64(f))] ] => { 
            assert_eq!(*f, 4f64);
            assert_eq!(first.len(), 0);
            assert_eq!(second.len(), 2);
            assert_eq!(get_float(&second[0]).unwrap(), 1f64);
            assert_eq!(get_float(&second[1]).unwrap(), 2f64);
            assert_eq!(third.len(), 3);
            assert_eq!(get_float(&third[0]).unwrap(), 1f64);
            assert_eq!(get_float(&third[1]).unwrap(), 2f64);
            assert_eq!(get_float(&third[2]).unwrap(), 3f64);
            matched = true;
        } );

        assert!(matched);
    }

    #[test]
    fn should_parse_data_float64() {
        let input = "-123.456E-2";
        let data = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(data => [Pattern::Number(Number::Float64(x))] => { 
            assert_eq!(x, -123.456E-2);
            matched = true;
        } );

        assert!(matched);
    }
}
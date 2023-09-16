
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

fn parse(input : &mut Chars) -> Result<Pattern, ParseError> {
    parser!(input => {
        pattern <= ! parse_pattern;
        ! end;
        select pattern 
    })
}

fn parse_pattern(input : &mut Chars) -> Result<Pattern, ParseError> {
    fn options(input : &mut Chars) -> Result<Pattern, ParseError> {
        alt!(input => parse_cons; 
                      parse_list_path;
                      parse_list; 
                      parse_wild;
                      // Note:  parse capture variable needs to happen after parse wild
                      parse_capture_var;
                      parse_path;
                      parse_path_next;
                      parse_symbol;
                      parse_string_pattern;
                      parse_template_variable)
    }

    fn end_options(input : &mut Chars) -> Result<EndCombinator, ParseError> {
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

fn parse_or(input : &mut Chars) -> Result<EndCombinator, ParseError> {
    pat!(parse_dot: char => () = '.' => ());
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn parse_and(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            and <= parse_word;
            where *and == *"or";
            select ()
        })
    }

    parser!(input => {
        _ws0 <= parse_whitespace;
        _dot <= parse_dot;
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

fn parse_and(input : &mut Chars) -> Result<EndCombinator, ParseError> {
    pat!(parse_dot: char => () = '.' => ());
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn parse_and(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            and <= parse_word;
            where *and == *"and";
            select ()
        })
    }

    parser!(input => {
        _ws0 <= parse_whitespace;
        _dot <= parse_dot;
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

fn parse_list_path(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());
    pat!(parse_bar: char => () = '|' => ());

    fn parse_l_bracket(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            _square <= parse_l_square;
            _bar <= parse_bar;
            select ()
        })
    }

    fn parse_r_bracket(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            _bar <= parse_bar;
            _square <= parse_r_square;
            select ()
        })
    }

    fn parse_points(input : &mut Chars) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_bracket, parse_pattern : Pattern, parse_r_bracket)
    }

    parser!(input => {
        points <= parse_points;
        select Pattern::ListPath(points)
    })
}

fn parse_path(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_l_curl: char => () = '{' => ());
    pat!(parse_r_curl: char => () = '}' => ());
    pat!(parse_bar: char => () = '|' => ());

    fn parse_l_bracket(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            _curl <= parse_l_curl;
            _bar <= parse_bar;
            select ()
        })
    }

    fn parse_r_bracket(input : &mut Chars) -> Result<(), ParseError> {
        parser!(input => {
            _bar <= parse_bar;
            _curl <= parse_r_curl;
            select ()
        })
    }

    fn parse_points(input : &mut Chars) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_bracket, parse_pattern : Pattern, parse_r_bracket)
    }

    parser!(input => {
        points <= parse_points;
        select Pattern::Path(points)
    })
}

pat!(parse_path_next<'a>: char => Pattern = '^' => Pattern::PathNext);

fn parse_cons(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_l_paren: char => () = '(' => ());
    pat!(parse_r_paren: char => () = ')' => ());

    fn param_list(input : &mut Chars) -> Result<Vec<Pattern>, ParseError> {
        parse_list!(input => parse_l_paren, parse_pattern : Pattern, parse_r_paren)
    }

    parser!(input => {
        cons_name <= parse_word;
        _clear <= parse_whitespace;
        params <= param_list;
        select Pattern::Cons { name: cons_name, params }
    })
}

fn parse_capture_var(input : &mut Chars) -> Result<Pattern, ParseError> {
    parser!(input => {
        word <= parse_word;
        select Pattern::CaptureVar(word)
    })
}

fn parse_wild(input : &mut Chars) -> Result<Pattern, ParseError> {
    parser!(input => {
        word <= parse_word;
        where *word == *"_";
        select Pattern::Wild
    })
}

fn parse_symbol(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_colon: char => () = ':' => ());
    parser!(input => {
        _colon <= parse_colon;
        word <= parse_word;
        select Pattern::Symbol(word)
    })
}

fn parse_string_pattern(input : &mut Chars) -> Result<Pattern, ParseError> {
    parser!(input => {
        string <= parse_string;
        select Pattern::String(string)
    })
}

fn parse_list(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_l_square: char => () = '[' => ());
    pat!(parse_r_square: char => () = ']' => ());

    Ok(Pattern::ExactList(parse_list!(input => parse_l_square, parse_pattern : Pattern, parse_r_square)?))
}

fn parse_template_variable(input : &mut Chars) -> Result<Pattern, ParseError> {
    pat!(parse_percent: char => () = '%' => ());

    parser!(input => {
        _dollar <= parse_percent;
        word <= ! parse_word;
        select Pattern::TemplateVar(word)
    })
}

#[cfg(test)]
mod test {
    use intra::*;
    use super::*;

    fn slice<'a, T>(input : &'a Vec<T>) -> &'a [T] { &input[..] }
    fn unbox<T>(input : Box<T> ) -> T { *input }

    #[test]
    fn should_parse_template() {
        let input = "cons(%a, [%b, %c, d, :e] )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [ Pattern::Cons { .. } ] =>  {
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_nested_ends() {
        let input = "[] . and( :c . or ( \"e\" . or( \"1.0\" ) ) )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [Pattern::And(a, b)] b; unbox $ [Pattern::Or(c, d)] d; unbox $ [Pattern::Or(e, f)] =>  {
            assert!( matches!( *a, Pattern::ExactList(_) ) );
            assert!( matches!( *c, Pattern::Symbol(_) ) );
            assert!( matches!( *e, Pattern::String(_) ) );
            assert!( matches!( *f, Pattern::String(_) ) );
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_multiple_alternating_ends() {
        let input = "[] . or ( \"1.0\" ) . and ( :b ) . or ( \"c\" )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [Pattern::Or(a, b)] a; unbox $ [Pattern::And(c, d)] c; unbox $ [Pattern::Or(e, f)] =>  {
            assert!( matches!( *b, Pattern::String(_) ) );
            assert!( matches!( *d, Pattern::Symbol(_) ) );
            assert!( matches!( *e, Pattern::ExactList(_) ) );
            assert!( matches!( *f, Pattern::String(_) ) );
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_or() {
        let input = ":a . or ( \"1.0\" )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [Pattern::Or(a, b)] =>  {
            assert!( matches!( *a, Pattern::Symbol(_) ) );
            assert!( matches!( *b, Pattern::String(_) ) );
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_and() {
        let input = ":a . and ( \"1.0\" )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [Pattern::And(a, b)] =>  {
            assert!( matches!( *a, Pattern::Symbol(_) ) );
            assert!( matches!( *b, Pattern::String(_) ) );
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_path() {
        let input = "{| a, b, c |}";
        let pattern = input.parse::<Pattern>().unwrap();
        assert!(matches!(pattern, Pattern::Path(_)));
    }

    #[test]
    fn should_parse_list_path() {
        let input = "[| a, b, c |]";
        let pattern = input.parse::<Pattern>().unwrap();
        assert!(matches!(pattern, Pattern::ListPath(_)));
    }

    #[test]
    fn should_parse_complex_pattern() {
        let input = " name ( other ( one( :a, :b, num([:c, :d, :e, [:blarg]]) ) ) , :inner  )";
        let pattern = input.parse::<Pattern>().unwrap();
        let mut matched = false;
        atom!(pattern => [Pattern::Cons { .. }] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_cons() {
        let input = " name  ( :first, :inner, \"5.5\" )";
        let pattern = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(pattern => [Pattern::Cons { name, params: ref params }] params; 
                       slice $ [ [Pattern::Symbol(a), Pattern::Symbol(b), Pattern::String(c)] ] => { 
            assert_eq!(*name, *"name");
            assert_eq!(**a, *"first");
            assert_eq!(**b, *"inner");
            assert_eq!(**c, *"5.5");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_capture_var() {
        let input = " symbol_123 ";
        let pattern = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(pattern => [Pattern::CaptureVar(sym)] => { 
            assert_eq!(*sym, *"symbol_123");
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_symbol() {
        let input = " :symbol_123 ";
        let pattern = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(pattern => [Pattern::Symbol(sym)] => { 
            assert_eq!(*sym, *"symbol_123");
            matched = true;
        } );
        assert!(matched);
    }
    
    #[test]
    fn should_parse_wild() {
        let input = " _ ";
        let pattern = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(pattern => [Pattern::Wild] => { 
            matched = true;
        } );
        assert!(matched);
    }

    #[test]
    fn should_parse_exact_list() {
        fn extract<'a>(p : &'a Pattern) -> &'a str {
            match p {
                Pattern::Symbol(x) => x,
                _ => panic!("extraction failure"),
            }
        }
        
        let input = " [ [], [:a, :b], [:c , :d, :e], :f] ";
        let pattern = input.parse::<Pattern>().unwrap();

        let mut matched = false;
        atom!(pattern => [Pattern::ExactList(ref params)] params; 
              slice $ [ [Pattern::ExactList(first), Pattern::ExactList(second), Pattern::ExactList(third), Pattern::Symbol(f)] ] => { 
            assert_eq!(**f, *"f");
            assert_eq!(first.len(), 0);
            assert_eq!(second.len(), 2);
            assert_eq!(extract(&second[0]), "a");
            assert_eq!(extract(&second[1]), "b");
            assert_eq!(third.len(), 3);
            assert_eq!(extract(&third[0]), "c");
            assert_eq!(extract(&third[1]), "d");
            assert_eq!(extract(&third[2]), "e");
            matched = true;
        } );

        assert!(matched);
    }
}
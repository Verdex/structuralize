
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use denest::*;


#[derive(Debug, Clone)]
pub enum Pattern {
    String(Box<str>), 
    Symbol(Box<str>),
    Wild,
    CaptureVar(Box<str>),
    Cons { name: Box<str>, params: Vec<Pattern> },
    ExactList(Vec<Pattern>),
    ListPath(Vec<Pattern>),
    PathNext,
    Path(Vec<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
    TemplateVar(Box<str>), // TODO : need to typecheck that these only occur in a ret pattern 
    Func { params : Vec<Box<str>>, ret : Box<Pattern> },
    // <| [a, b, c] => cons( a , b , c) |> // a => pat <==> (a : a) => pat
    // (a : [| h |] ) b c => cons( $a , $b , $c, d)
    // TODO pattern function (really want to see if this can work)
}

// a b c => cons($a, $b, $c, d) 
// <| [a, b, c] : pat1 => pat2, ... |>
//    the exact list here is just syntax, the pat1 will still match against a list, but the user doesn't get an opportunity to put in something exotic
//    should the captures in pat1 be available for producing pat2 ?

impl<'a> Linearizable<'a> for Pattern {
    fn l_next(&'a self) -> Vec<&'a Self> {
        use Pattern::*;
        match self {
            String(_) => vec![], 
            Symbol(_) => vec![],
            Wild => vec![],
            CaptureVar(_) => vec![],
            Cons { params, name: _ } => params.iter().collect(),
            ExactList(ps) => ps.iter().collect(),
            ListPath(ps) => ps.iter().collect(),
            PathNext => vec![],
            Path(ps) => ps.iter().collect(),
            And(a, b) => vec![&**a, &**b],
            Or(a, b) => vec![&**a, &**b],
            TemplateVar(_) => vec![],
            Func { params: _, ret } => vec![&**ret],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Slot {
    Next,
    Symbol(Box<str>),
    Path(Vec<Box<str>>),
}

impl Display for Slot {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        match self {
            Slot::Next => write!(f, "Next"),
            Slot::Symbol(s) => write!(f, "{}", s),
            Slot::Path(s) => write!(f, "{}", s.join(".")),
        }
    }
}

impl From<&Box<str>> for Slot {
    fn from(item : &Box<str>) -> Self {
        Slot::Symbol(item.clone())
    }
}

impl From<Box<str>> for Slot {
    fn from(item : Box<str>) -> Self {
        Slot::Symbol(item)
    }
}

impl From<&str> for Slot {
    fn from(item : &str) -> Self {
        Slot::Symbol(item.into())
    }
}

impl From<&String> for Slot {
    fn from(item : &String) -> Self {
        Slot::Symbol(item.clone().into())
    }
}

impl From<String> for Slot {
    fn from(item : String) -> Self {
        Slot::Symbol(item.into())
    }
}

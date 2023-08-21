
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use denest::*;

use crate::data::*;


#[derive(Debug, Clone)]
pub enum Pattern {
    Number(Number),
    String(Box<str>), 
    Symbol(Box<str>),
    Wild,
    CaptureVar(Box<str>),
    Cons { name: Box<str>, params: Vec<Pattern> },
    Struct { name: Box<str>, fields: Vec<(Box<str>, Pattern)> },
    ExactList(Vec<Pattern>),
    ListPath(Vec<Pattern>),
    PathNext,
    Path(Vec<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
    // TODO reference other pattern ? 
    // TODO pattern function (really want to see if this can work)
    // TODO lessthan pattern ? 
    // TODO greater than pattern ? 
    // TODO greater than or equal pattern ? 
    // TODO less than or equal pattern ? 
}

impl<'a> Linearizable<'a> for Pattern {
    fn l_next(&'a self) -> Vec<&'a Self> {
        use Pattern::*;
        match self {
            Number(_) => vec![],
            String(_) => vec![], 
            Symbol(_) => vec![],
            Wild => vec![],
            CaptureVar(_) => vec![],
            Cons { params, .. } => params.iter().collect(),
            Struct { fields: fs, .. } => fs.iter().map(|(_, p)| p).collect(),
            ExactList(ps) => ps.iter().collect(),
            ListPath(ps) => ps.iter().collect(),
            PathNext => vec![],
            Path(ps) => ps.iter().collect(),
            And(a, b) => vec![&**a, &**b],
            Or(a, b) => vec![&**a, &**b],
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

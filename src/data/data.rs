
use std::fmt::{Display, Formatter};

use denest::*;

#[derive(Debug, PartialEq)]
pub enum Data {
    String(Box<str>), 
    Symbol(Box<str>),
    Cons { name: Box<str>, params: Vec<Data> },
    List(Vec<Data>),
}

impl<'a> Linearizable<'a> for Data {
    fn l_next(&'a self) -> Vec<&'a Data> {
        match self {
            Data::String(_) => vec![],
            Data::Symbol(_) => vec![],
            Data::Cons { params, .. } => params.iter().collect(),
            Data::List(ds) => ds.iter().collect(),
        }
    }
}

impl Display for Data {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        match self {
            Data::String(s) => write!(f, "\"{}\"", s), 
            Data::Symbol(s) => write!(f, ":{}", s),
            Data::Cons { name, params } => write!(f, "{}({})", name, params.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(", ")),
            Data::List(ds) => write!(f, "[{}]", ds.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(", ")),
        }
    }
}

impl From<Box<str>> for Data {
    fn from(item : Box<str>) -> Self {
        Data::String(item.into())
    }
}

impl From<&str> for Data {
    fn from(item : &str) -> Self {
        Data::String(item.into())
    }
}

impl From<String> for Data {
    fn from(item : String) -> Self {
        Data::String(item.into())
    }
}

impl<T> From<Vec<T>> for Data where Data : From<T> {
    fn from(item : Vec<T>) -> Self {
        Data::List(item.into_iter().map(|x| x.into()).collect::<Vec<Data>>())
    }
}
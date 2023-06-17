
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::collections::HashMap;

use crate::data::*;


#[derive(Debug, Clone)]
pub enum Pattern {
    Number(Number),
    CaptureVar(Box<str>),
    Cons { name: Box<str>, params: Vec<Pattern> },
}

#[derive(Debug, Clone)]
pub struct MatchResult<'a> {
    map : HashMap<Slot, &'a Data>
}

impl<'a> MatchResult<'a> {
    pub fn new() -> Self {
        MatchResult { map: HashMap::new() }
    }
    
    pub fn merge(&mut self, other : MatchResult<'a>) { 
        // TODO: Is it faster to collect both self and other?
        // TODO:  This can fail on duplicate slots unless pattern is type checked
        for (key, value) in other.map.into_iter() {
            self.map.insert(key, value);
        }
    }

    pub fn get(&self, key : &Slot) -> Option<&'a Data> {
        Some(*self.map.get(key)?)
    }

    pub fn add(&mut self, key : Slot, value : &'a Data) {
        self.map.insert(key, value);
    }
}

impl<'a, const N : usize> From<[(Slot, &'a Data); N]> for MatchResult<'a> {
    fn from(item : [(Slot, &'a Data); N]) -> Self {
        let map = item.into_iter().map(|(k,v)| (k, v)).collect::<HashMap<Slot, &'a Data>>();
        MatchResult { map }
    }
}

impl<'a> From<Vec<(Slot, &'a Data)>> for MatchResult<'a> {
    fn from(item : Vec<(Slot, &'a Data)>) -> Self {
        let map = item.into_iter().map(|(k,v)| (k, v)).collect::<HashMap<Slot, &'a Data>>();
        MatchResult { map }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Slot {
    Symbol(String),
    Path(Vec<String>),
}

impl Display for Slot {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        match self {
            Slot::Symbol(s) => write!(f, "{}", s),
            Slot::Path(s) => write!(f, "{}", s.join(".")),
        }
    }
}

impl From<&Box<str>> for Slot {
    fn from(item : &Box<str>) -> Self {
        Slot::Symbol(item.to_string())
    }
}

impl From<Box<str>> for Slot {
    fn from(item : Box<str>) -> Self {
        Slot::Symbol(item.to_string())
    }
}

impl From<&str> for Slot {
    fn from(item : &str) -> Self {
        Slot::Symbol(item.to_string())
    }
}

impl From<&String> for Slot {
    fn from(item : &String) -> Self {
        Slot::Symbol(item.to_string())
    }
}

impl From<String> for Slot {
    fn from(item : String) -> Self {
        Slot::Symbol(item.to_string())
    }
}

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work

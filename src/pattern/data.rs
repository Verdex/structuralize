
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::collections::HashMap;

use crate::data::*;


#[derive(Debug, Clone)]
pub enum Pattern {
    CaptureVar(String),
    As(String, Box<Pattern>),
    Cons { name: String, params: Vec<Pattern> },
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    map : HashMap<Slot, Data>
}

impl MatchResult {
    pub fn new() -> Self {
        MatchResult { map: HashMap::new() }
    }
    
    pub fn merge(&mut self, other : MatchResult) { 
        // TODO: Is it faster to collect both self and other?
        // TODO:  This can fail on duplicate slots unless pattern is type checked
        for (key, value) in other.map.into_iter() {
            self.map.insert(key, value);
        }
    }

    pub fn get(&self, key : &Slot) -> Option<&Data> {
        self.map.get(key)
    }
}

impl<const N : usize> From<[(Slot, &Data); N]> for MatchResult {
    fn from(item : [(Slot, &Data); N]) -> Self {
        let map = item.into_iter().map(|(k,v)| (k, v.clone())).collect::<HashMap<Slot, Data>>();
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

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work

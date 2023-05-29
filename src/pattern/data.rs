
use std::hash::Hash;
use std::collections::HashMap;

use crate::data::*;


#[derive(Debug, Clone)]
pub enum Pattern {
    CaptureVar(String),
    As(String, Box<Pattern>),
    Cons { name: String, params: Vec<Pattern> },
}

#[derive(Debug)]
pub struct MatchResult {
    map : HashMap<Slot, Data>
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

impl From<&String> for Slot {
    fn from(item : &String) -> Self {
        Slot::Symbol(item.to_string())
    }
}

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work

#[derive(Debug)]
pub enum MatchError {

}

impl std::fmt::Display for MatchError {
   fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TODO")
    }
}

impl std::error::Error for MatchError {}
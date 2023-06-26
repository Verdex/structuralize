
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::collections::HashMap;

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
    RestList(Vec<Pattern>, Box<Pattern>), // TODO needs parser
    // TODO allow parsing path next whereever, but make sure the type check
    // prevents path next anywhere except inside of a path pattern
    PathNext,
    Path(Vec<Pattern>),
    // TODO and pattern
    // TODO or pattern
    // TODO not pattern? (with brakets or just traditional prefix not?)
    // TODO "array" pattern
    // TODO reference other pattern
    // TODO pattern function
    // TODO lessthan pattern
    // TODO greater than pattern
    // TODO greater than or equal pattern
    // TODO less than or equal pattern
}

// TODO a pattern that captures something probably shouldn't be allowed to convert that data into a pattern literal
// and then use that as a reference other pattern in a later part of the pattern
// specifically because that could depend on the evaluation order of the pattern as a whole 

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

    pub fn extract(self) -> Vec<(Slot, &'a Data)> {
        self.map.into_iter().collect()
    }

    pub fn extract_nexts(&mut self) -> Vec<&'a Data> {
        let nexts = self.map.keys().filter(|k| matches!(k, Slot::Next(_))).map(|k| k.clone()).collect::<Vec<_>>();
        let mut ret = vec![];
        for next in nexts {
            let data = self.map.remove(&next).unwrap();
            ret.push(data);
        }
        ret
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
    Next(usize),
    Symbol(Box<str>),
    Path(Vec<Box<str>>),
}

impl Display for Slot {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        match self {
            Slot::Next(x) => write!(f, "~Next({})~", x),
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

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work


#[cfg(test)]
mod test {
    use super::*;

    fn get_float(d : &Data) -> f64 {
        if let Data::Number(Number::Float64(f)) = d {
            *f
        }
        else {
            panic!("get_float::not a number");
        }
    }

    #[test]
    fn result_should_extract_nexts() {
        let mut result = MatchResult::new();

        let a : Data = "1".parse().unwrap();
        let b : Data = "2".parse().unwrap();
        let c : Data = "3".parse().unwrap();

        result.add(Slot::Next(1), &a);
        result.add(Slot::Next(2), &b);
        result.add("x".into(), &c);

        let mut nexts = result.extract_nexts();

        nexts.sort_by(|a, b| get_float(a).partial_cmp(&get_float(b)).unwrap() );

        assert_eq!( result.map.iter().count(), 1 );
        assert_eq!( result.get(&"x".into()).unwrap(), &"3".parse::<Data>().unwrap() );

        assert_eq!( nexts.len(), 2 );
        assert_eq!( nexts[0], &a );
        assert_eq!( nexts[1], &b );
    }
}
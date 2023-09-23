
use crate::data::*;
use super::data::*;
use super::check::*;

pub type MatchMap<'a> = Vec<(Slot, &'a Data)>;

pub fn pattern_match<'data>(pattern : &TypeChecked, data : &'data Data) -> Vec<MatchMap<'data>> {
    //inner_match(pattern.pattern(), data, &vec![])
    vec![]
}

#[derive(Debug, Clone)]
pub struct Matches<'a> {
    matches : MatchMap<'a>,
    current_work : Vec<(Pattern, &'a Data)>,
    future_work : Vec<(MatchMap<'a>, Vec<(Pattern, &'a Data)>)>,
}

impl<'a> Iterator for Matches<'a> {
    type Item = MatchMap<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_work.len() == 0 {
            return None;
        }

        while self.current_work.len() > 0 {
            match self.current_work.pop().unwrap() {
                (Pattern::CaptureVar(name), data) => { self.matches.push((name.into(), data)); },
                _ => todo!(),
            }
        }

        if self.future_work.len() == 0 {
            let ret = std::mem::replace(&mut self.matches, vec![]);
            Some(ret)
        }
        else {
            let (new_matches, new_work) = self.future_work.pop().unwrap();
            self.current_work = new_work;
            let ret = std::mem::replace(&mut self.matches, new_matches);
            Some(ret)
        }
    }
}

fn data_to_pattern(data : &Data) -> Pattern {
    match data {
        Data::String(s) => Pattern::String(s.clone()), 
        Data::Symbol(s) => Pattern::Symbol(s.clone()),
        Data::Cons { name, params } => Pattern::Cons { name: name.clone(), params: params.iter().map(data_to_pattern).collect() },
        Data::List(ds) => Pattern::ExactList(ds.iter().map(data_to_pattern).collect()),
    }
}
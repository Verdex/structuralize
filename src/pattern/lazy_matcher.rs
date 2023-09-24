
use crate::data::*;
use super::data::*;
use super::check::*;

pub type MatchMap<'a> = Vec<(Slot, &'a Data)>;

pub fn pattern_match<'data>(pattern : &TypeChecked, data : &'data Data) -> Matches<'data> {
    let p = pattern.pattern().clone();
    Matches { matches: vec![], current_work: vec![(p, &data)], future_work: vec![] }
}

#[derive(Debug, Clone)]
pub struct Matches<'a> {
    matches : MatchMap<'a>,
    current_work : Vec<(Pattern, &'a Data)>,
    future_work : Vec<(MatchMap<'a>, Vec<(Pattern, &'a Data)>)>,
}

// QueueWork
macro_rules! qw {
    ($s : expr, $pattern : expr, $data : expr) => {
        for (p, d) in $pattern.into_iter().zip($data.iter()).rev() {
            $s.push((p, d));
        }
    };
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
                (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == 0 && ds.len() == 0 => { /* pass */ },
                (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
                    qw!(self.current_work, ps, ds);
                },

                (Pattern::ListPath(ps), Data::List(_)) if ps.len() == 0 => { /* pass */ },
                (Pattern::ListPath(ps), Data::List(ds)) if ps.len() <= ds.len() => {
                    let p_len = ps.len();

                    for i in 1..=(ds.len() - p_len) {
                        let target = &ds[i..(i + p_len)];
                        let mut work = self.current_work.clone();
                        qw!(work, ps.clone(), target);
                        self.future_work.push((self.matches.clone(), work));
                    }

                    let target = &ds[0..p_len];
                    qw!(self.current_work, ps, target);
                },

                _ => { 
                    // This match failed
                    if let Some((new_matches, new_work)) = self.future_work.pop() {
                        self.current_work = new_work;
                        self.matches = new_matches;
                    }
                    else {
                        self.current_work = vec![];
                        self.matches = vec![];
                        return None;
                    }
                },
            }
        }

        if let Some((new_matches, new_work)) = self.future_work.pop() {
            self.current_work = new_work;
            let ret = std::mem::replace(&mut self.matches, new_matches);
            Some(ret)
        }
        else {
            let ret = std::mem::replace(&mut self.matches, vec![]);
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
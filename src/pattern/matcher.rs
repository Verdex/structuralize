
use crate::data::*;
use super::data::*;
use super::check::*;

// TODO : Consider .. pattern so that cons and exact list can ignore fields

// TODO : allow list of functions that take the matches and return a pattern of known type
//        probably needs to specify the type
// TODO : allow list of functions that "match"
// TODO : pattern match function that takes Matchable impl instead of Data (have data implement matchable)
// TODO : pattern match function that takes Patternable impl instead of Pattern 

// TODO : Atom (eqable), object(not eqable), cons, list


// TODO move this to pattern::data here in a bit
pub type MatchMap<'a, T> = Vec<(Box<str>, &'a T)>; 

pub fn pattern_match<'a, D : Matchable>(pattern : &TypeChecked, data : &'a D) -> Matches<'a, D> {
    let p = pattern.pattern().clone();
    let mut current_work = Work::new();
    current_work.push((p, data));

    Matches { matches: vec![], current_work, future_work: vec![] }
}

#[derive(Debug)]
struct WorkPath<'a, D : Matchable> {
    work : Vec<(Pattern, &'a D)>,
    path : Vec<Pattern>,
    nexts : Vec<&'a D>,
}

impl<'a, D : Matchable> Clone for WorkPath<'a, D> {
    fn clone(&self) -> Self {
        WorkPath { work: self.work.clone()
                 , path: self.path.clone()
                 , nexts: self.nexts.clone()
                 }
    }
}

impl<'a, D : Matchable> WorkPath<'a, D> {
    pub fn empty() -> Self {
        WorkPath { work: vec![], path: vec![], nexts: vec![] }
    }

    pub fn new(path : Vec<Pattern>) -> Self {
        WorkPath { work: vec![], path, nexts: vec![] }
    }

    pub fn push(&mut self, item : (Pattern, &'a D)) {
        self.work.push(item);
    }

    pub fn pop(&mut self) -> Option<(Pattern, &'a D)> {
        self.work.pop()
    }

    pub fn work_finished(&self) -> bool {
        self.work.len() == 0
    }
}

#[derive(Debug)]
struct Work<'a, D : Matchable> {
    work : Vec<WorkPath<'a, D>>,
}

impl<'a, D : Matchable> Clone for Work<'a, D> {
    fn clone(&self) -> Self {
        Work { work : self.work.clone() }
    }
}

impl<'a, D : Matchable> Work<'a, D> {
    pub fn new() -> Self {
        Work { work : vec![WorkPath::empty()] }
    }

    pub fn push(&mut self, item : (Pattern, &'a D)) {
        self.work.last_mut().unwrap().push(item);
    }

    pub fn work_finished(&self) -> bool {
        self.work.len() == 1 && self.work.last().unwrap().work_finished()
    }

    pub fn path(&mut self, mut patterns : Vec<Pattern>, data : &'a D) {
        patterns.reverse();
        let first_pattern = patterns.pop().unwrap();
        self.work.push(WorkPath::new(patterns));
        self.push((first_pattern, data));
    }

    pub fn next(&mut self, data : &'a D) {
        self.work.last_mut().unwrap().nexts.push(data);
    }
}

#[derive(Debug, Clone)]
pub struct Matches<'a, D : Matchable> {
    matches : MatchMap<'a, D>,
    current_work : Work<'a, D>,
    future_work : Vec<(MatchMap<'a, D>, Work<'a, D>)>,
}

// QueueWork
macro_rules! qw {
    ($s : expr, $pattern : expr, $data : expr) => {
        for (p, d) in $pattern.into_iter().zip($data.iter()).rev() {
            $s.push((p, d));
        }
    };
}

impl<'a, D : Matchable> Matches<'a, D> {
    fn pop_current_work(&mut self) -> Option<(Pattern, &'a D)> { 
        if let Some(ret) = self.current_work.work.last_mut().unwrap().pop() {
            Some(ret)
        }
        else if self.current_work.work.last().unwrap().nexts.len() != 0 { 
            let mut nexts = std::mem::replace(&mut self.current_work.work.last_mut().unwrap().nexts, vec![]);
            nexts.reverse();

            let pattern = self.current_work.work.last_mut().unwrap().path.pop().unwrap();

            let first = nexts.pop().unwrap();

            for next in nexts.into_iter() {
                let mut work = self.current_work.clone();
                work.push((pattern.clone(), next));
                self.future_work.push((self.matches.clone(), work));
            }

            Some((pattern, first))
        }
        else if self.current_work.work.len() > 1 {
            self.current_work.work.pop();
            self.pop_current_work()
        }
        else {
            None
        }
    }
}

impl<'a, D : Matchable> Iterator for Matches<'a, D> {
    type Item = MatchMap<'a, D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_work.work_finished() {
            return None;
        }

        while let Some((pattern, matchable)) = self.pop_current_work() {
            match (pattern, matchable.kind()) {
                (Pattern::CaptureVar(name), _) => { self.matches.push((name.into(), matchable)); },
                (Pattern::ExactList(ps), MatchKind::List(ds)) if ps.len() == 0 && ds.len() == 0 => { /* pass */ },
                (Pattern::ExactList(ps), MatchKind::List(ds)) if ps.len() == ds.len() => {
                    qw!(self.current_work, ps, ds);
                },

                (Pattern::Cons {name: pname, params: pparams}, MatchKind::Cons(dname, dparams)) 
                // TODO !!!! pname is hopefully becoming &str soon ::: REMOVE .into() from dname
                    if pname == dname.into() && pparams.len() == dparams.len() => {

                    qw!(self.current_work, pparams, dparams);
                },

                (Pattern::ListPath(ps), MatchKind::List(_)) if ps.len() == 0 => { /* pass */ },
                (Pattern::ListPath(ps), MatchKind::List(ds)) if ps.len() <= ds.len() => {
                    let p_len = ps.len();

                    for i in (1..=(ds.len() - p_len)).rev() { // TODO I would be really great if this were lazy
                        let target = &ds[i..(i + p_len)];
                        let mut work = self.current_work.clone();
                        qw!(work, ps.clone(), target);
                        self.future_work.push((self.matches.clone(), work));
                    }

                    let target = &ds[0..p_len];
                    qw!(self.current_work, ps, target);
                },

                (Pattern::Wild, _) => { /* pass */ },
                //(Pattern::Symbol(p), Data::Symbol(d)) if p == *d => { /* pass */ }, 
                //(Pattern::String(p), Data::String(d)) if p == *d => { /* pass */ },

                (Pattern::TemplateVar(var), _) => {
                    let (_, d) = self.matches.iter().find(|(k, _)| k == &var ).unwrap();
                    let p = data_to_pattern(*d);
                    self.current_work.push((p, matchable));
                },

                (Pattern::And(a, b), _) => {
                    self.current_work.push((*b, matchable));
                    self.current_work.push((*a, matchable));
                },

                (Pattern::PathNext, _) => { 
                    self.current_work.next(matchable);
                },
                
                (Pattern::Path(ps), _) if ps.len() == 0 => { /* pass */ },
                (Pattern::Path(ps), _) => {
                    self.current_work.path(ps.clone(), matchable);   
                },

                (Pattern::Or(a, b), _) => {
                    let mut work = self.current_work.clone();
                    work.push((*b, matchable));
                    self.future_work.push((self.matches.clone(), work));
                    self.current_work.push((*a, matchable));
                },

                _ => { 
                    // This match failed
                    if let Some((new_matches, new_work)) = self.future_work.pop() {
                        self.current_work = new_work;
                        self.matches = new_matches;
                    }
                    else {
                        self.current_work = Work::new();
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

// TODO: rename to something like matchable_to_pattern
fn data_to_pattern<D : Matchable>(data : &D) -> Pattern {
    match data.kind() {
        //Data::String(s) => Pattern::String(s.clone()), 
        //Data::Symbol(s) => Pattern::Symbol(s.clone()),
        // TODO name.clone().into() needs fixed !!!
        MatchKind::Cons(name, params) => Pattern::Cons { name: name.clone().into(), params: params.iter().map(data_to_pattern).collect() },
        MatchKind::List(ds) => Pattern::ExactList(ds.iter().map(data_to_pattern).collect()),
        _ => todo!(),
    }
}
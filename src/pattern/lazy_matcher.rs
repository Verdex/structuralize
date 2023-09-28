
use crate::data::*;
use super::data::*;
use super::check::*;

pub type MatchMap<'a> = Vec<(Slot, &'a Data)>; // TODO this should be common

pub fn pattern_match<'data>(pattern : &TypeChecked, data : &'data Data) -> Matches<'data> {
    let p = pattern.pattern().clone();
    let mut current_work = Work::new();
    current_work.push((p, data));

    Matches { matches: vec![], current_work, future_work: vec![] }
}

#[derive(Debug, Clone)]
struct WorkQueue<'a> {
    queue : Vec<(Pattern, &'a Data)>,
}

impl<'a> WorkQueue<'a> {
    pub fn new() -> Self {
        WorkQueue { queue : vec![] }
    }

    pub fn push(&mut self, item : (Pattern, &'a Data)) {
        self.queue.push(item);
    }

    pub fn pop(&mut self) -> Option<(Pattern, &'a Data)> {
        self.queue.pop()
    }

    pub fn work_finished(&self) -> bool {
        self.queue.len() == 0
    }
}

#[derive(Debug, Clone)]
struct Work<'a> {
    work : Vec<WorkQueue<'a>>,
}

impl<'a> Work<'a> {
    pub fn new() -> Self {
        Work { work : vec![WorkQueue::new()] }
    }

    pub fn push(&mut self, item : (Pattern, &'a Data)) {
        self.work.last_mut().unwrap().push(item);
    }

    pub fn pop(&mut self) -> Option<(Pattern, &'a Data)> {
        self.work.last_mut().unwrap().pop()
    }

    pub fn work_finished(&self) -> bool {
        self.work.len() == 1 && self.work.last().unwrap().work_finished()
    }

    pub fn path(&mut self, mut patterns : Vec<Pattern>, data : &'a Data) {
        patterns.reverse();
        let first_pattern = patterns.pop().unwrap();
        self.work.push(WorkQueue::new());
        self.push((first_pattern, data));
    }
}

#[derive(Debug, Clone)]
pub struct Matches<'a> {
    matches : MatchMap<'a>,
    current_work : Work<'a>,
    future_work : Vec<(MatchMap<'a>, Work<'a>)>,
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
        if self.current_work.work_finished() {
            return None;
        }

        while let Some(w) = self.current_work.pop() {
            match w {
                (Pattern::CaptureVar(name), data) => { self.matches.push((name.into(), data)); },
                (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == 0 && ds.len() == 0 => { /* pass */ },
                (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
                    qw!(self.current_work, ps, ds);
                },

                (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
                    if pname == *dname && pparams.len() == dparams.len() => {

                    qw!(self.current_work, pparams, dparams);
                },

                (Pattern::ListPath(ps), Data::List(_)) if ps.len() == 0 => { /* pass */ },
                (Pattern::ListPath(ps), Data::List(ds)) if ps.len() <= ds.len() => {
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
                (Pattern::Symbol(p), Data::Symbol(d)) if p == *d => { /* pass */ }, 
                (Pattern::String(p), Data::String(d)) if p == *d => { /* pass */ },

                (Pattern::TemplateVar(var), data) => {
                    let var : Slot = var.into();
                    let (_, d) = self.matches.iter().find(|(k, _)| k == &var ).unwrap();
                    let p = data_to_pattern(d);
                    self.current_work.push((p, data));
                },

                (Pattern::And(a, b), data) => {
                    self.current_work.push((*b, data));
                    self.current_work.push((*a, data));
                },

                (Pattern::Path(ps), _) if ps.len() == 0 => { /* pass */ },
                (Pattern::Path(ps), data) => {
                    self.current_work.path(ps.clone(), data);   
                },

                (Pattern::Or(a, b), data) => {
                    let mut work = self.current_work.clone();
                    work.push((*b, data));
                    self.future_work.push((self.matches.clone(), work));
                    self.current_work.push((*a, data));
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

fn data_to_pattern(data : &Data) -> Pattern {
    match data {
        Data::String(s) => Pattern::String(s.clone()), 
        Data::Symbol(s) => Pattern::Symbol(s.clone()),
        Data::Cons { name, params } => Pattern::Cons { name: name.clone(), params: params.iter().map(data_to_pattern).collect() },
        Data::List(ds) => Pattern::ExactList(ds.iter().map(data_to_pattern).collect()),
    }
}
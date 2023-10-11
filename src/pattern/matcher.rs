
use crate::data::*;
use super::data::*;
use super::check::*;


pub type MatchMap<'a> = Vec<(Slot, &'a Data)>;

// TODO : replace the (public:  actually just a internal one should do the trick) pattern match with another version that accepts cows 
// and then calls the reference version

// TODO : Consider * pattern and some sub tree pattern, but only allow a single instance in the list path / path 
// TODO : Consider .. pattern so that cons and exact list can ignore fields

// TODO : allow list of functions that take the matches and return a pattern of known type
//        probably needs to specify the type
// TODO : allow list of functions that "match" ?
// TODO : pattern match function that takes Matchable impl instead of Data (have data implement matchable)

pub fn pattern_match<'data>(pattern : &TypeChecked, data : &'data Data) -> Vec<MatchMap<'data>> {
    inner_match(pattern.pattern(), data, &vec![])
}

fn alt_inner_matches<'a>(pattern : &Pattern, 
                         data : &'a Data, 
                         previous_match_groups : Vec<MatchMap<'a>>,
                         top_matches : &MatchMap<'a>) 
    -> Vec<MatchMap<'a>> {

    let mut results = vec![];
    for previous_matches in &previous_match_groups {
        let matches = previous_matches.iter().chain(top_matches.iter()).map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        let mut current_result_groups = inner_match(pattern, data, &matches);

        for mut current_results in &mut current_result_groups {
            let mut prev = previous_matches.clone();
            prev.append(&mut current_results);
            std::mem::swap(current_results, &mut prev);
        }

        results.append(&mut current_result_groups);
    }
    results 
}

fn inner_match<'data>(pattern : &Pattern, data : &'data Data, matches : &MatchMap<'data>) -> Vec<MatchMap<'data>> {

    macro_rules! pass { 
        () => { vec![ vec![] ] };
    } 
    macro_rules! fail {
        () => { vec![] };
    }

    match (pattern, data) {
        (Pattern::CaptureVar(name), data) => vec![ [(name.into(), data)].into() ],
        (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == 0 && ds.len() == 0 => pass!(),
        (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
            ps.iter().zip(ds.iter()).fold(pass!(), |previous_match_groups, (p, d)| { 
                alt_inner_matches(p, d, previous_match_groups, matches)
            })
        },

        (Pattern::ListPath(ps), Data::List(_)) if ps.len() == 0 => pass!(),
        (Pattern::ListPath(ps), Data::List(ds)) if ps.len() > ds.len() => fail!(),
        (Pattern::ListPath(ps), Data::List(ds)) => {
            let p_len = ps.len();
            let mut ret = vec![];
            for i in 0..=(ds.len() - p_len) {
                let target = &ds[i..(i + p_len)];
                let results = ps.iter().zip(target.iter()).fold(pass!(), |previous_match_groups, (p, d)| { 
                    alt_inner_matches(p, d, previous_match_groups, matches)
                });
                ret.push(results);
            }
            ret.into_iter().flatten().collect()
        }

        (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
            if pname == dname && pparams.len() == dparams.len() => {
            pparams.iter().zip(dparams.iter()).fold(pass!(), |previous_match_groups, (p, d)| { 
                alt_inner_matches(p, d, previous_match_groups, matches)
            })
        },
         
        (Pattern::Wild, _) => pass!(),
        (Pattern::String(p), Data::String(d)) if p == d => pass!(),
        (Pattern::Symbol(p), Data::Symbol(d)) if p == d => pass!(), 
        (Pattern::PathNext, data) => vec![ [(Slot::Next, data)].into() ],
        (Pattern::Path(ps), _) if ps.len() == 0 => pass!(),
        (Pattern::Path(ps), data) => {
            let mut outer : Vec<Vec<MatchMap>> = vec![];
            let results = inner_match(&ps[0], data, matches);
            for result in results {
                let nexts : Vec<&Data> = result.iter().filter_map(|r| match r { (Slot::Next, d) => Some(*d), _ => None }).collect();

                let top : MatchMap = result.iter().filter_map(|r| match r { (Slot::Next, _) => None, (s, d) => Some((s.clone(), *d)) }).collect();
                if nexts.len() == 0 {
                    outer.push(vec![top]);
                }
                else {
                    let mut inner : Vec<MatchMap> = vec![];
                    for next in nexts {
                        let rest = ps[1..].iter().map(|x| x.clone()).collect::<Vec<_>>();
                        let mut inner_results = alt_inner_matches(&Pattern::Path(rest), next, vec![top.clone()], matches);
                        inner.append(&mut inner_results);
                    }
                    outer.push(inner);
                }
            }
            outer.into_iter().flatten().collect()
        },

        (Pattern::And(a, b), data) => {
            let a_results = inner_match(a, data, matches);
            if a_results.len() == 0 {
                fail!()
            }
            else {
                alt_inner_matches(b, data, a_results, matches)
            }
        },

        (Pattern::Or(a, b), data) => {
            let mut a_results = inner_match(a, data, matches);
            let mut b_results = inner_match(b, data, matches);
            a_results.append(&mut b_results);
            a_results
        },

        (Pattern::TemplateVar(var), data) => {
            let var : Slot = var.into();
            let (_, d) = matches.iter().find(|(k, _)| k == &var ).unwrap();
            let p = data_to_pattern(d);
            inner_match(&p, data, matches)
        },

        _ => fail!(),
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


use std::collections::HashMap;

use crate::data::*;
use super::data::*;
use super::check::*;


pub type MatchMap<K, V> = Vec<(K, V)>;

fn product<'a>(a : Vec<MatchMap<Slot, &'a Data>>, b : Vec<MatchMap<Slot, &'a Data>>) -> Vec<MatchMap<Slot, &'a Data>> {
    let mut ret = vec![];
    for blet in b {
        for alet in a.iter() {
            let mut alet = alet.clone();
            let mut blet = blet.clone();
            alet.append(&mut blet);
            ret.push(alet);
        }
    }
    ret
}

fn collapse<'a>(input : Vec<MatchMap<Slot, &'a Data>>) -> MatchMap<Slot, &'a Data> {
    input.into_iter().flat_map(|hm| hm.into_iter()).collect()
}

// TODO : replace the (public:  actually just a internal one should do the trick) pattern match with another version that accepts cows 
// and then calls the reference version

pub fn pattern_match<'data>(pattern : &TypeChecked, data : &'data Data) -> Vec<MatchMap<Slot, &'data Data>> {
    inner_match(pattern.pattern(), data, &vec![ vec![] ])
}

fn inner_match<'data>(pattern : &Pattern, data : &'data Data, matches : &Vec<MatchMap<Slot, &'data Data>>) -> Vec<MatchMap<Slot, &'data Data>> {
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
            ps.iter().zip(ds.iter()).fold(pass!(), |accum, (p, d)| { 
                let r = inner_match(p, d, &accum);
                product(accum, r)
            })
        },

        (Pattern::ListPath(ps), Data::List(_)) if ps.len() == 0 => pass!(),
        (Pattern::ListPath(ps), Data::List(ds)) if ps.len() > ds.len() => fail!(),
        (Pattern::ListPath(ps), Data::List(ds)) => {
            let p_len = ps.len();
            let mut ret = vec![];
            for i in 0..=(ds.len() - p_len) {
                let target = &ds[i..(i + p_len)];
                let results = ps.iter().zip(target.iter()).fold(pass!(), |accum, (p, d)| { 
                    let r = inner_match(p, d, &accum);
                    product(accum, r)
                });
                ret.push(results);
            }
            ret.into_iter().flatten().collect()
        }

        (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
            if pname == dname && pparams.len() == dparams.len() => {
            pparams.iter().zip(dparams.iter()).fold(pass!(), |accum, (p, d)| { 
                let r = inner_match(p, d, &accum);
                product(accum, r)
            })
        },
         
        (Pattern::Wild, _) => pass!(),
        (Pattern::String(p), Data::String(d)) if p == d => pass!(),
        (Pattern::Symbol(p), Data::Symbol(d)) if p == d => pass!(), 
        (Pattern::PathNext, data) => vec![ [(Slot::Next, data)].into() ],
        (Pattern::Path(ps), _) if ps.len() == 0 => pass!(),
        (Pattern::Path(ps), data) => {
            let mut outer : Vec<Vec<MatchMap<_, _>>> = vec![];
            let results = inner_match(&ps[0], data, &pass!());
            for result in results {
                let nexts : Vec<&Data> = result.iter().filter_map(|r| match r { (Slot::Next, d) => Some(*d), _ => None }).collect();

                let top : MatchMap<Slot, &Data> = result.iter().filter_map(|r| match r { (Slot::Next, _) => None, (s, d) => Some((s.clone(), *d)) }).collect();
                if nexts.len() == 0 {
                    outer.push(vec![top]);
                }
                else {
                    let mut inner : Vec<MatchMap<_, _>> = vec![];
                    for next in nexts {
                        let rest = ps[1..].iter().map(|x| x.clone()).collect::<Vec<_>>();
                        let inner_results = inner_match(&Pattern::Path(rest), next, &pass!());
                        let mut inner_results_with_top : Vec<MatchMap<_, _>> = inner_results.into_iter().map(|x| collapse(vec![top.clone(), x])).collect();
                        inner.append(&mut inner_results_with_top);
                    }
                    outer.push(inner);
                }
            }
            outer.into_iter().flatten().collect()
        },

        (Pattern::And(a, b), data) => {
            let a_results = inner_match(a, data, &pass!());
            if a_results.len() == 0 {
                fail!()
            }
            else {
                let b_results = inner_match(b, data, &pass!());
                product(a_results, b_results)
            }
        },

        // TODO:  Should both branches generate results if they're both true?
        (Pattern::Or(a, b), data) => {
            let a_results = inner_match(a, data, &pass!());
            if a_results.len() != 0 {
                a_results
            }
            else {
                inner_match(b, data, &pass!())
            }
        },

        (Pattern::Func { params, ret }, data) => {

            let mut results = vec![];
            for m in matches {
                let map : HashMap<Slot, &Data> = m.iter().map(|(k, v)| (k.clone(), *v)).collect();

                let p = template_pattern(ret, &map);

                let mut output = inner_match(&p, data, &pass!()); // TODO this seems incorrect because then you can't have a func return a func

                results.append(&mut output);
            }

            results
        },

        _ => fail!(),
    }
}

fn template_pattern(p : &Pattern, map : &HashMap<Slot, &Data>) -> Pattern {
    use Pattern::*;
    match p {
        // TODO type checking should ensure that the var is in the map
        TemplateVar(var) => data_to_pattern(*map.get(&var.into()).unwrap()),

        x @ String(_) => x.clone(), 
        x @ Symbol(_) => x.clone(),
        Wild => Wild,
        x @ CaptureVar(_) => x.clone(),
        Cons { params, name } => Cons { name: name.clone(), params: params.iter().map(|p| template_pattern(p, map)).collect() },
        ExactList(ps) => ExactList(ps.iter().map(|p| template_pattern(p, map)).collect()),
        ListPath(ps) => ListPath(ps.iter().map(|p| template_pattern(p, map)).collect()),
        PathNext => PathNext,
        Path(ps) => Path(ps.iter().map(|p| template_pattern(p, map)).collect()),
        And(a, b) => And(Box::new(template_pattern(a, map)), Box::new(template_pattern(b, map))),
        Or(a, b) => Or(Box::new(template_pattern(a, map)), Box::new(template_pattern(b, map))),
        Func { params, ret } => Func { params: params.clone(), ret: Box::new(template_pattern(ret, map)) },
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn product_should_work() {
        let d1 = ":a".parse::<Data>().unwrap();
        let d2 = ":b".parse::<Data>().unwrap();
        let a = vec![ [("x".into(), &d1), ("y".into(), &d1)].into(), [("x".into(), &d2), ("y".into(), &d2)].into() ];
        let b = vec![ [("z".into(), &d1), ("w".into(), &d1)].into(), [("z".into(), &d2), ("w".into(), &d2)].into() ];

        let output = product(a, b).into_iter().map(|x| x.into_iter().collect::<HashMap<_, _>>()).collect::<Vec<_>>();

        assert_eq!( output.len(), 4 );
        assert_eq!( **output[0].get(&"x".into()).unwrap(), d1 );
        assert_eq!( **output[0].get(&"y".into()).unwrap(), d1 );
        assert_eq!( **output[0].get(&"z".into()).unwrap(), d1 );
        assert_eq!( **output[0].get(&"w".into()).unwrap(), d1 );

        assert_eq!( **output[1].get(&"x".into()).unwrap(), d2 );
        assert_eq!( **output[1].get(&"y".into()).unwrap(), d2 );
        assert_eq!( **output[1].get(&"z".into()).unwrap(), d1 );
        assert_eq!( **output[1].get(&"w".into()).unwrap(), d1 );

        assert_eq!( **output[2].get(&"x".into()).unwrap(), d1 );
        assert_eq!( **output[2].get(&"y".into()).unwrap(), d1 );
        assert_eq!( **output[2].get(&"z".into()).unwrap(), d2 );
        assert_eq!( **output[2].get(&"w".into()).unwrap(), d2 );

        assert_eq!( **output[3].get(&"x".into()).unwrap(), d2 );
        assert_eq!( **output[3].get(&"y".into()).unwrap(), d2 );
        assert_eq!( **output[3].get(&"z".into()).unwrap(), d2 );
        assert_eq!( **output[3].get(&"w".into()).unwrap(), d2 );
    }
}
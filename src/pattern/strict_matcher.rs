
use std::collections::HashMap;

use crate::data::*;
use super::data::*;

fn product<'a>(mut input : Vec<Vec<HashMap<Slot, &'a Data>>> ) -> Vec<Vec<HashMap<Slot, &'a Data>>> {
    if input.len() == 1 {
        return input.pop().unwrap().into_iter().map(|x| vec![x]).collect();
    }

    match input.pop() {
        None => vec![],
        Some(results) => {
            let mut sub_solution = product(input);
            let mut ret = vec![];
            for result in results {
                for solution in sub_solution.iter_mut() {
                    let mut s = solution.clone();
                    s.push(result.clone());
                    ret.push(s);
                }
            }
            ret
        },
    }
}

fn collapse<'a>(input : Vec<Vec<HashMap<Slot, &'a Data>>>) -> Vec<HashMap<Slot, &'a Data>> {
    input.into_iter()
         .map(|results| results.into_iter().flat_map(|hm| hm.into_iter()).collect())
         .collect()
}

fn get_id() -> usize { // TODO:  Would like something different
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ID : AtomicUsize = AtomicUsize::new(0);
    ID.fetch_add(1, Ordering::Relaxed)
}

// TODO : Vec<(Slot, Data)> might be better than HashMap                
// Although naming it with an alias would probably be a good idea 
pub fn strict_pattern_match<'data>(pattern : &Pattern, data : &'data Data) -> Vec<HashMap<Slot, &'data Data>> {
    macro_rules! pass { 
        () => { vec![ HashMap::new() ] };
    } 
    macro_rules! fail {
        () => { vec![] };
    }

    match (pattern, data) {
        (Pattern::CaptureVar(name), data) => vec![ [(name.into(), data)].into() ],
        (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == 0 && ds.len() == 0 => pass!(),
        (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
            let results : Vec<Vec<HashMap<_, _>>> = ps.iter().zip(ds.iter()).map(|(p, d)| strict_pattern_match(p, d)).collect();
            collapse(product(results))
        },

        // TODO add a test that fields are fine even if they are sorted differently
        (Pattern::Struct { name: pname, fields: pfields }, Data::Struct { name: dname, fields: dfields } )
            if pname == dname && pfields.len() == dfields.len() => {

            // Note:  'Typechecking' will process structs such that their fields are sorted

            for (p_field_name, d_field_name) in pfields.iter()
                                                        .zip(dfields.iter())
                                                        .map(|((p, _), (d, _))| (p, d)) {
                if p_field_name != d_field_name {
                    return fail!();
                }
            }

            let ps = pfields.iter().map(|(_, p)| p);
            let ds = dfields.iter().map(|(_, d)| d);

            let results : Vec<Vec<HashMap<_, _>>> = ps.zip(ds).map(|(p, d)| strict_pattern_match(p, d)).collect();
            collapse(product(results))
        },
        // TODO do empty cons need to be prevented?
        (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
            if pname == dname && pparams.len() == dparams.len() => {
            let results : Vec<Vec<HashMap<_, _>>> = pparams.iter().zip(dparams.iter()).map(|(p, d)| strict_pattern_match(p, d)).collect();
            collapse(product(results))
        },
         
        (Pattern::Wild, _) => pass!(),
        (Pattern::Number(p), Data::Number(d)) if p == d => pass!(),
        (Pattern::String(p), Data::String(d)) if p == d => pass!(),
        (Pattern::Symbol(p), Data::Symbol(d)) if p == d => pass!(), 
        (Pattern::PathNext, data) => vec![ [(Slot::Next(get_id()), data)].into() ],
        (Pattern::Path(ps), _) if ps.len() == 0 => pass!(),
        (Pattern::Path(ps), data) => {
            let mut outer : Vec<Vec<HashMap<_, _>>> = vec![];
            let results = strict_pattern_match(&ps[0], data);
            for result in results {
                let mut inner : Vec<Vec<HashMap<_, _>>> = vec![];
                let nexts : Vec<&Data> = result.iter().filter_map(|r| match r { (Slot::Next(_), d) => Some(*d), _ => None }).collect();
                let top = result.iter().filter_map(|r| match r { (Slot::Next(_), _) => None, (s, d) => Some((s.clone(), *d)) }).collect::<HashMap<Slot, &Data>>();
                for next in nexts {
                    let w = ps[1..].iter().map(|x| x.clone()).collect::<Vec<_>>();
                    let ww = Pattern::Path(w);
                    let q = strict_pattern_match(&ww, next);
                    let j = collapse(vec![vec![top.clone()], q]);
                    inner.push(j);
                }
                let blarg : Vec<HashMap<_, _>> = inner.into_iter().flatten().collect::<Vec<_>>();
                outer.push(blarg);
            }
            outer.into_iter().flatten().collect()
        },
        _ => fail!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn product_should_generate_hashmap_product() {
        let d1 = ":a".parse::<Data>().unwrap();
        let d2 = ":b".parse::<Data>().unwrap();
        let input : Vec<Vec<HashMap<Slot, &Data>>> = vec![
            vec![ [("x".into(), &d1), ("y".into(), &d1)].into(), [("x".into(), &d2), ("y".into(), &d2)].into() ],
            vec![ [("z".into(), &d1), ("w".into(), &d1)].into(), [("z".into(), &d2), ("w".into(), &d2)].into() ]
        ];

        let output = product(input);

        assert_eq!( output.len(), 4 );
        assert_eq!( output[0].len(), 2 );
        assert_eq!( **output[0][0].get(&"x".into()).unwrap(), d1 );
        assert_eq!( **output[0][0].get(&"y".into()).unwrap(), d1 );
        assert_eq!( **output[0][1].get(&"z".into()).unwrap(), d1 );
        assert_eq!( **output[0][1].get(&"w".into()).unwrap(), d1 );

        assert_eq!( output[1].len(), 2 );
        assert_eq!( **output[1][0].get(&"x".into()).unwrap(), d2 );
        assert_eq!( **output[1][0].get(&"y".into()).unwrap(), d2 );
        assert_eq!( **output[1][1].get(&"z".into()).unwrap(), d1 );
        assert_eq!( **output[1][1].get(&"w".into()).unwrap(), d1 );

        assert_eq!( output[2].len(), 2 );
        assert_eq!( **output[2][0].get(&"x".into()).unwrap(), d1 );
        assert_eq!( **output[2][0].get(&"y".into()).unwrap(), d1 );
        assert_eq!( **output[2][1].get(&"z".into()).unwrap(), d2 );
        assert_eq!( **output[2][1].get(&"w".into()).unwrap(), d2 );

        assert_eq!( output[3].len(), 2 );
        assert_eq!( **output[3][0].get(&"x".into()).unwrap(), d2 );
        assert_eq!( **output[3][0].get(&"y".into()).unwrap(), d2 );
        assert_eq!( **output[3][1].get(&"z".into()).unwrap(), d2 );
        assert_eq!( **output[3][1].get(&"w".into()).unwrap(), d2 );
    }

    #[test]
    fn collapse_should_combine_hashmap_product() {
        let d1 = ":a".parse::<Data>().unwrap();
        let d2 = ":b".parse::<Data>().unwrap();
        let input : Vec<Vec<HashMap<Slot, &Data>>> = vec![
            vec![ [("x".into(), &d1), ("y".into(), &d1)].into(), [("x".into(), &d2), ("y".into(), &d2)].into() ],
            vec![ [("z".into(), &d1), ("w".into(), &d1)].into(), [("z".into(), &d2), ("w".into(), &d2)].into() ]
        ];

        let output = collapse(product(input));

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
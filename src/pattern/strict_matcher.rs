
use std::collections::HashMap;

use crate::data::*;
use super::data::*;

/*fn product<'a>(mut input : Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    if input.len() == 1 {
        return input.pop().unwrap().into_iter().map(|x| vec![x]).collect();
    }

    match input.pop() {
        None => vec![],
        Some(x) => {
            let mut blarg = product(input);

            let mut ret = vec![];
            for xlet in x {
                for b in blarg.iter_mut() {
                    let mut z = b.clone();
                    z.push(xlet.clone());
                    ret.push(z);
                }
            }
            ret
        },
    }
}*/

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

pub fn strict_pattern_match<'pattern, 'data>(pattern : &'pattern Pattern, data : &'data Data) -> Vec<HashMap<Slot, &'data Data>> {
    match (pattern, data) {
        (Pattern::CaptureVar(name), data) => vec![ [(name.into(), data)].into() ],
        /*(Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => 

            // product 
            ps.iter().zip(ds.iter()).map(|(p, d)| strict_pattern_match(p, d)),

        // TODO add a test that fields are fine even if they are sorted differently
        (Pattern::Struct { name: pname, fields: pfields }, Data::Struct { name: dname, fields: dfields } )
            if pname == dname && pfields.len() == dfields.len() => {

            // Note:  'Typechecking' will process structs such that their fields are sorted

            for (p_field_name, d_field_name) in pfields.iter()
                                                        .zip(dfields.iter())
                                                        .map(|((p, _), (d, _))| (p, d)) {
                if p_field_name != d_field_name {
                    return JoinResult::Fail;
                }
            }

            let ps = pfields.iter().map(|(_, p)| p).collect::<Vec<_>>();
            let ds = dfields.iter().map(|(_, d)| d).collect::<Vec<_>>();
            join_star!(ps, ds)
        },
        (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
            if pname == dname && pparams.len() == dparams.len() => 

            join_star!(pparams, dparams),

        (Pattern::Wild, _) => JoinResult::Pass,
        (Pattern::Number(p), Data::Number(d)) if p == d => JoinResult::Pass,
        (Pattern::String(p), Data::String(d)) if p == d => JoinResult::Pass,
        (Pattern::Symbol(p), Data::Symbol(d)) if p == d => JoinResult::Pass, 
        (Pattern::PathNext, data) => JoinResult::Join(DataPattern::Next(data)),
        (Pattern::Path(ps), _) if ps.len() == 0 => JoinResult::Pass,
        (Pattern::Path(ps), data) => JoinResult::Join(DataPattern::PathGroup(PathGroup { pattern : &ps[..], data })),*/
        _ => vec![],
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
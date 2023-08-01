
use std::collections::HashMap;

use crate::data::*;
use super::data::*;

fn product<'a>(mut input : Vec<Vec<u8>>) -> Vec<Vec<u8>> {
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
            
            /*for xlet in x {
                for mut b in blarg.iter_mut() {
                    b.push(xlet);
                }
            }
            blarg*/
        },
    }
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
    fn blarg() {
        let x = vec![vec![1, 2, 3], vec![9, 8, 7], vec![4, 5, 6]];
        let o = product(x);

        println!("{:?}", o);
    }
}
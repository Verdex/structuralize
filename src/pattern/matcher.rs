

use denest::Linearizable;
use crate::data::*;
use super::data::*;

// TODO:  phantom type type checked patterns
// * in a path each step needs at least one next except for the last one which cannot have any nexts
// * make sure that structs don't have duplicate field names
// * also make sure that struct fields are sorted

pub struct MatchResults<'pattern, 'data> {
    pattern : &'pattern Pattern,
    data : &'data Data,
    stop : bool // TODO
}

#[derive(Debug)]
struct PathGroup<'pattern, 'data> {
    pattern : &'pattern [Pattern],
    data : &'data Data,
}

impl<'pattern, 'data> Iterator for PathGroup<'pattern, 'data> {
    type Item = JoinResult<'pattern, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        //join(self.data)
        None
    }
}

#[derive(Debug)]
enum DataPattern<'pattern, 'data> {
    Capture(Slot, &'data Data),
    Next(&'data Data),
    SingleGroup(Vec<DataPattern<'pattern, 'data>>),
    PathGroup(PathGroup<'pattern, 'data>),
}

impl<'pattern, 'data> DataPattern<'pattern, 'data> {
    pub fn nexts(&self) -> Vec<&'data Data> {
        self.to_lax().filter_map(|dp| match dp { DataPattern::Next(d) => Some(*d), _ => None }).collect()
    }
}

impl<'a, 'pattern, 'data> Linearizable<'a> for DataPattern<'pattern, 'data> {
    fn l_next(&'a self) -> Vec<&'a Self> {
        match self {
            DataPattern::Capture(_, _) => vec![],
            DataPattern::Next(_) => vec![],
            DataPattern::SingleGroup(dsp) => dsp.iter().collect::<Vec<_>>(),
            DataPattern::PathGroup(_) => vec![], // TODO is this a good idea?
            _ => todo!(),
        }
    }
}

enum JoinResult<'pattern, 'data> {
    Pass,
    Fail,
    Join(DataPattern<'pattern, 'data>)
}

fn join<'pattern, 'data>(pattern : &'pattern Pattern, data : &'data Data) -> JoinResult<'pattern, 'data> {

    macro_rules! join_star {
        ($ps:ident, $ds:ident) => {{
            let mut ret = vec![];
            for (p, d) in $ps.iter().zip($ds.iter()) {
                match join(p, d) {
                    JoinResult::Pass => { },
                    JoinResult::Fail => { return JoinResult::Fail; },
                    JoinResult::Join(dp) => { ret.push(dp); },
                }
            }
            JoinResult::Join(DataPattern::SingleGroup(ret))
        }}
    }

    match (pattern, data) { 
        (Pattern::CaptureVar(name), data) => JoinResult::Join(DataPattern::Capture(name.into(), data)),
        (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => 
            join_star!(ps, ds),

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
        (Pattern::Path(ps), data) => JoinResult::Join(DataPattern::PathGroup(PathGroup { pattern : &ps[..], data })),
        _ => JoinResult::Fail,
    }
}

impl<'pattern, 'data> Iterator for MatchResults<'pattern, 'data> {
    type Item = MatchResult<'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop { return None; }
        let pattern = self.pattern;
        let data = self.data;
        match join(&pattern, &data) {
            JoinResult::Fail => None,
            JoinResult::Pass => { self.stop = true; Some(MatchResult::new()) },
            JoinResult::Join(data_pattern) => { 
                self.stop = true; 
                let result : Vec<(Slot, &'data Data)>
                    = data_pattern.to_lax()
                    // TODO see if we can get rid of the clone for slot
                                  .flat_map(|dp| match dp { DataPattern::Capture(n, d) => vec![(n.clone(), *d)], _ => vec![] })
                                  .collect::<Vec<_>>();
                Some(result.into()) 
            }, // TODO 
        }
    }
}

pub fn pattern_match<'pattern, 'data>(pattern : &'pattern Pattern, data : &'data Data) -> MatchResults<'pattern, 'data> {
    MatchResults { pattern, data, stop : false }
}

#[cfg(test)] 
mod test {
    use super::*;

    // TODO we need this anyway for the hopefully upcoming type check
    fn p(input : &'static str) -> Pattern {
        input.parse().unwrap()
    }

    // TODO : path pattern that has path patterns inside of it (needs more impl before this will work)
    #[test]
    fn should_match_nested_nexts_in_path() {
        let pattern = p("{| cons(cons(^, ^), ^), [^], x |}");
        let data : Data = "cons(cons([:a], [:b]), [:c])".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 3);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":c".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());

        let observed = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed, &":b".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_only_valid_paths() {
        let pattern = p("{| cons(^, ^), [^], x |}");
        let data : Data = "cons([:a], :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_not_match_path() {
        let pattern = p("{| cons(:whatever) |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_empty_path() {
        let pattern = p("{| |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_multi_step_multi_next_path() {
        let pattern = p("{| cons(^, ^), [^, ^], x |}");
        let data : Data = "cons( [:a, :b], [:c, :d] )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 4);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":d".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":c".parse::<Data>().unwrap());

        let observed = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed, &":b".parse::<Data>().unwrap());

        let observed = results[3].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_path() {
        let pattern = p("cons( {| cons(^, ^), [^], x |}, {| cons(^, ^), [^], y |} )");
        let data : Data = "cons( cons([:a], [1.1]), cons([:b], [2.2]) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 4);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_y= results[0].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &"2.2".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_y= results[1].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &"2.2".parse::<Data>().unwrap());

        let observed_x = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_y= results[2].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());

        let observed_x = results[3].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_y= results[3].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_capture_after() {
        let pattern = p("cons( {| cons(^, ^), [^], x |}, outer )");
        let data : Data = "cons( cons([:a], [1.1]), :outer )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_outer = results[0].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_outer = results[1].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_capture_before() {
        let pattern = p("cons( outer, {| cons(^, ^), [^], x |} )");
        let data : Data = "cons( :outer, cons([:a], [1.1]) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_outer = results[0].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_outer = results[1].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());
    }
}
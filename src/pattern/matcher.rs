

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

#[derive(Clone)]
enum ToMatch<'a> {
    Concrete { pattern : Pattern, data : &'a Data}
}

impl<'a> From<(Pattern, &'a Data)> for ToMatch<'a> {
    fn from(item : (Pattern, &'a Data)) -> Self {
        ToMatch::Concrete { pattern: item.0, data: item.1 }
    }
}

struct State<'a> {
    pattern : Pattern,
    data : &'a Data,
    captures : Vec<(Slot, &'a Data)>,
    match_queue : Vec<ToMatch<'a>>,
}

impl<'a> State<'a> {
    pub fn new(pattern: Pattern, data: &'a Data) -> State<'a> {
        State { pattern, data, captures : vec![], match_queue : vec![] }
    }
}

pub struct MatchResults<'a> {
    match_states : Vec<State<'a>>,
    next_id : usize,
}

impl<'a> Iterator for MatchResults<'a> {
    type Item = MatchResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer : loop {
            if self.match_states.len() == 0 {
                return None;
            }

            let current_state = self.match_states.pop().unwrap();

            let mut captures : Vec<(Slot, &'a Data)> = current_state.captures;
            let mut match_queue : Vec<ToMatch<'a>> = current_state.match_queue;

            match_queue.push((current_state.pattern, current_state.data).into());

            while match_queue.len() > 0 {
                let current_match = match match_queue.pop().unwrap() {
                    ToMatch::Concrete { pattern, data } => (pattern, data),
                };

                match current_match {
                    (Pattern::CaptureVar(name), data) => {
                        captures.push((name.into(), data));
                    },
                    (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
                        
                        let mut to_match = ps.into_iter().zip(ds.iter()).map(|x| x.into()).collect::<Vec<_>>();
                        match_queue.append(&mut to_match);
                    },
                    (Pattern::Struct { name: pname, fields: pfields }, Data::Struct { name: dname, fields: dfields } )
                        if pname == *dname && pfields.len() == dfields.len() => {

                        for (p_field_name, d_field_name) in pfields.iter()
                                                                   .zip(dfields.iter())
                                                                   .map(|((p, _), (d, _))| (p, d)) {
                            if p_field_name != d_field_name {
                                continue 'outer;
                            }
                        }

                        let mut to_match = pfields.into_iter().zip(dfields.iter()).map(|((_, p), (_, d))| (p, d).into()).collect::<Vec<_>>();
                        match_queue.append(&mut to_match);
                    },
                    (Pattern::Cons {name: pname, params: pparams}, Data::Cons {name: dname, params: dparams}) 
                        if pname == *dname && pparams.len() == dparams.len() => {

                        let mut to_match = pparams.into_iter().zip(dparams.iter()).map(|x| x.into()).collect::<Vec<_>>();
                        match_queue.append(&mut to_match);
                    },
                    (Pattern::Wild, _) => { },
                    (Pattern::Number(pn), Data::Number(dn)) if pn == *dn => { },
                    (Pattern::String(p), Data::String(d)) if p == *d => { },
                    (Pattern::Symbol(p), Data::Symbol(d)) if p == *d => { },
                    (Pattern::PathNext, data) => {
                        captures.push((Slot::Next(self.next_id), data));
                        self.next_id += 1;
                    },
                    (Pattern::Path(ps), mut data) => {
                        // TODO clean up this clause

                        let mut pi = 0;

                        while pi < ps.len() {
                            
                            let mut internal_match_results = pattern_match(ps[pi].clone(), data).collect::<Vec<_>>(); // is collect right here?

                            if internal_match_results.len() == 0 {
                                // nothing matches
                                continue 'outer;
                            }
                            else if internal_match_results.len() == 1 {
                                let mut result = internal_match_results.pop().unwrap();

                                let mut nexts = result.extract_nexts();

                                let mut internal_captures = result.clone().extract(); // TODO not so much with clone

                                captures.append(&mut internal_captures);

                                // TODO make sure that the type checker makes sure that each path pattern
                                // has at least one next except the last one which should have none

                                if nexts.len() == 0 {
                                    // okay if there are no more items in ps left to process
                                    // otherwise the match fails
                                }
                                else {
                                    let first_next = nexts.remove(0);
                                    // if there are nexts but no more ps, then this should be some sort 
                                    // of typecheck error
                                    for next in nexts {
                                        let jabber = ps[pi + 1 ..].iter().map(|x| x.clone()).collect::<Vec<_>>();
                                        let mut state = State::new(Pattern::Path(jabber), next); // TODO 
                                        state.captures = captures.clone();
                                        state.match_queue = match_queue.clone();
                                        self.match_states.push(state);
                                    }
                                    data = first_next;
                                }
                            }
                            else {
                                // if the first pattern is a path pattern that ends up with multiple matches
                                panic!("!");
                            }

                            pi += 1;
                        }
                    },
                    _ => {
                        continue 'outer;
                    },
                }
            }

            return Some(captures.into());
        }
    }
}

pub fn pattern_match<'a>(pattern : Pattern, data : &'a Data) -> MatchResults<'a> {
    MatchResults { match_states : vec![State::new(pattern, data)], next_id: 0 }
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
    fn should_match_only_valid_paths() {
        let pattern = p("{| cons(^, ^), [^], x |}");
        let data : Data = "cons([:a], :b)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_not_match_path() {
        let pattern = p("{| cons(:whatever) |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_empty_path() {
        let pattern = p("{| |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_multi_step_multi_next_path() {
        let pattern = p("{| cons(^, ^), [^, ^], x |}");
        let data : Data = "cons( [:a, :b], [:c, :d] )".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
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

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
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

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
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

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
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
    fn should_match_multiple_paths_with_cons_and_list() { 
        let pattern = p("{| cons(^, ^), [^], x |}");
        let data : Data = "cons([:a], [1.1])".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &"1.1".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_with_cons_and_list() {
        let pattern = p("{| cons(^, _), [^], x |}");
        let data : Data = "cons([:a], 1.1)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_due_to_number() {
        let pattern = p("cons(a, 1.1)");
        let data : Data = "cons(:a, 1.1)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_number() {
        let pattern = p("cons(a, 1.1)");
        let data : Data = "cons(:a, 1.2)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_wild() {
        let pattern = p("_");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_due_to_symbol() {
        let pattern = p("cons(a, :b)");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_symbol() {
        let pattern = p("cons(a, :a)");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_due_to_string() {
        let pattern = p("cons(a, \"leta\")");
        let data : Data = "cons(:a, \"leta\")".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_string() {
        let pattern = p("cons(a, \"leta\")");
        let data : Data = "cons(:a, \"letb\")".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_single_var() {
        let pattern = p("x");
        let data : Data = "cons(:a)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &data);
    }

    #[test]
    fn should_fail_match_due_to_nested_cons_internal_mismatch() {
        let pattern = p("cons( :a, :b, :c, cons(:x) )");
        let data : Data = "cons(:a, :b, :c, cons(:a) )".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_internal_mismatch() {
        let pattern = p("cons( :a, :b, :c, :x )");
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_name_mismatch() {
        let pattern = p("other( x, y, z )");
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_length_mismatch() {
        let pattern = p("cons( x, y, z )");
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_cons_with_vars() {
        let pattern = p("cons( x, y, z )");
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        let observed_y = results[0].get(&"y".into()).unwrap();
        let observed_z = results[0].get(&"z".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());
        assert_eq!(observed_z, &":c".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_struct() {
        let pattern = p("struct { a: 1, b: 2, c: 3 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_struct_with_inner_var() {
        let pattern = p("struct { a: 1, b: 2, c: x }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"3".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_struct_due_to_length() {
        let pattern = p("struct { a: 1, b: 2 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_struct_due_to_inner_name_mismatch() {
        let pattern = p("struct { a: 1, b: 2, x: 3 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_exact_list() {
        let pattern = p("[1, x, :a]");
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"2".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_empty_exact_list() {
        let pattern = p("[]");
        let data : Data = "[]".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_length() {
        let pattern = p("[1, x, :a, :x]");
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_value() {
        let pattern = p("[1, x, :a, :x]");
        let data : Data = "[1, 2, :a, :y]".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_nested_list_mismatch() {
        let pattern = p("[1, x, :a, [:x, :x]]");
        let data : Data = "[1, 2, :a, [:x, :y]]".parse().unwrap();

        let results = pattern_match(pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }
}
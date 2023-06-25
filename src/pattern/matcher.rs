

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub struct MatchResults<'a, 'b> {
    target : Vec<(&'a Pattern, &'b Data)>,
}

impl<'a, 'b> Iterator for MatchResults<'a, 'b> {
    type Item = MatchResult<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer : loop {
            if self.target.len() == 0 {
                return None;
            }

            let x = self.target.pop().unwrap();

            let mut result : Vec<(Slot, &'b Data)> = vec![];
            let mut q : Vec<(&'a Pattern, &'b Data)> = vec![];

            q.push(x);

            while q.len() > 0 {
                let target = q.pop().unwrap();

                match target {
                    (Pattern::CaptureVar(name), data) => {
                        result.push((name.into(), data));
                    },
                    (Pattern::ExactList(ps), Data::List(ds)) if ps.len() == ds.len() => {
                        
                        let mut z = ps.iter().zip(ds.iter()).collect::<Vec<_>>();
                        q.append(&mut z);
                    },
                    (Pattern::Struct { name: pname, fields: pfields }, Data::Struct { name: dname, fields: dfields } )
                        if pname == dname && pfields.len() == dfields.len() => {

                        for (p_field_name, d_field_name) in pfields.iter()
                                                                   .zip(dfields.iter())
                                                                   .map(|((p, _), (d, _))| (p, d)) {
                            if p_field_name != d_field_name {
                                continue 'outer;
                            }
                        }

                        let mut z = pfields.iter().zip(dfields.iter()).map(|((_, p), (_, d))| (p, d)).collect::<Vec<_>>();
                        q.append(&mut z);
                    },
                    (Pattern::Cons {name: pname, params: pparam}, Data::Cons {name: dname, params: dparam}) 
                        if pname == dname && pparam.len() == dparam.len() => {

                        let mut z = pparam.iter().zip(dparam.iter()).collect::<Vec<_>>();
                        q.append(&mut z);
                    },
                    (Pattern::Wild, _) => { },
                    (Pattern::Number(pn), Data::Number(dn)) if pn == dn => { },
                    (Pattern::String(p), Data::String(d)) if p == d => { },
                    (Pattern::Symbol(p), Data::Symbol(d)) if p == d => { },
                    _ => {
                        continue 'outer;
                    },
                }
            }

            return Some(result.into());
        }
    }
}

pub fn pattern_match<'a, 'b>(pattern : &'a Pattern, data : &'b Data) -> MatchResults<'a, 'b> {
    MatchResults { target : vec![(pattern, data)]}
}

#[cfg(test)] 
mod test {
    use super::*;

    #[test]
    fn should_match_due_to_number() {
        let pattern : Pattern = "cons(a, 1.1)".parse().unwrap();
        let data : Data = "cons(:a, 1.1)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_number() {
        let pattern : Pattern = "cons(a, 1.1)".parse().unwrap();
        let data : Data = "cons(:a, 1.2)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_wild() {
        let pattern : Pattern = "_".parse().unwrap();
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_due_to_symbol() {
        let pattern : Pattern = "cons(a, :b)".parse().unwrap();
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_symbol() {
        let pattern : Pattern = "cons(a, :a)".parse().unwrap();
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_due_to_string() {
        let pattern : Pattern = "cons(a, \"leta\")".parse().unwrap();
        let data : Data = "cons(:a, \"leta\")".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_string() {
        let pattern : Pattern = "cons(a, \"leta\")".parse().unwrap();
        let data : Data = "cons(:a, \"letb\")".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_single_var() {
        let pattern : Pattern = "x".parse().unwrap();
        let data : Data = "cons(:a)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &data);
    }

    #[test]
    fn should_fail_match_due_to_nested_cons_internal_mismatch() {
        let pattern : Pattern = "cons( :a, :b, :c, cons(:x) )".parse().unwrap();
        let data : Data = "cons(:a, :b, :c, cons(:a) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_internal_mismatch() {
        let pattern : Pattern = "cons( :a, :b, :c, :x )".parse().unwrap();
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_name_mismatch() {
        let pattern : Pattern = "other( x, y, z )".parse().unwrap();
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_length_mismatch() {
        let pattern : Pattern = "cons( x, y, z )".parse().unwrap();
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_cons_with_vars() {
        let pattern : Pattern = "cons( x, y, z )".parse().unwrap();
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
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
        let pattern : Pattern = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_struct_with_inner_var() {
        let pattern : Pattern = "struct { a: 1, b: 2, c: x }".parse().unwrap();
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"3".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_struct_due_to_length() {
        let pattern : Pattern = "struct { a: 1, b: 2 }".parse().unwrap();
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_struct_due_to_inner_name_mismatch() {
        let pattern : Pattern = "struct { a: 1, b: 2, x: 3 }".parse().unwrap();
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_struct_due_to_nested_struct_mismatch() {
        let pattern : Pattern = "struct { a: 1, b: 2, c: inner { x: 1 } }".parse().unwrap();
        let data : Data = "struct { a: 1, b: 2, c: inner { d: 1} }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_exact_list() {
        let pattern : Pattern = "[1, x, :a]".parse().unwrap();
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"2".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_empty_exact_list() {
        let pattern : Pattern = "[]".parse().unwrap();
        let data : Data = "[]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_length() {
        let pattern : Pattern = "[1, x, :a, :x]".parse().unwrap();
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_value() {
        let pattern : Pattern = "[1, x, :a, :x]".parse().unwrap();
        let data : Data = "[1, 2, :a, :y]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_nested_list_mismatch() {
        let pattern : Pattern = "[1, x, :a, [:x, :x]]".parse().unwrap();
        let data : Data = "[1, 2, :a, [:x, :y]]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }
}
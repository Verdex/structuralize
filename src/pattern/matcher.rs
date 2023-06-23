

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

}
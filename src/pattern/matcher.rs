

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub struct MatchResults<'a, 'b> {
    target : Vec<(&'a Pattern, &'b Data)>,
}

impl<'a, 'b> Iterator for MatchResults<'a, 'b> {
    type Item = MatchResult<'b>;

    fn next(&mut self) -> Option<Self::Item> {
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
                (Pattern::Cons {name: pname, params: pparam}, Data::Cons {name: dname, params: dparam}) 
                    if pname == dname && pparam.len() == dparam.len() => {
                    let mut z = pparam.iter().zip(dparam.iter()).collect::<Vec<_>>();
                    q.append(&mut z)
                },
                _ => todo!(),
            }

        }

        Some(result.into())
    }
}

pub fn pattern_match<'a, 'b>(pattern : &'a Pattern, data : &'b Data) -> MatchResults<'a, 'b> {
    MatchResults { target : vec![(pattern, data)]}
}

#[cfg(test)] 
mod test {
    use super::*;

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
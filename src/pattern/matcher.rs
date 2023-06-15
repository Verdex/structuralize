

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub struct MatchResults<'a, 'b> {
    target : Vec<(&'a Pattern, &'b Data)>,
}

impl<'a, 'b> Iterator for MatchResults<'a, 'b> {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.target.pop();

        match x {
            (Pattern::CaptureVar(name), data) => {

            }
        }

        None
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
        let pattern = Pattern::CaptureVar("x".into());
        let data : Data = "cons(a)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &data);
    }

    #[test]
    fn should_match_cons_with_vars() {
        let pattern = Pattern::Cons { name : "cons".into()
                                    , params: vec![ Pattern::CaptureVar("x".into())
                                                  , Pattern::CaptureVar("y".into())
                                                  , Pattern::CaptureVar("z".into())
                                                  ]
                                   };
        let data : Data = "cons(a, b, c)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        let observed_y = results[0].get(&"y".into()).unwrap();
        let observed_z = results[0].get(&"z".into()).unwrap();
        assert_eq!(observed_x, &"a".parse::<Data>().unwrap());
        assert_eq!(observed_y, &"b".parse::<Data>().unwrap());
        assert_eq!(observed_z, &"c".parse::<Data>().unwrap());
    }

}
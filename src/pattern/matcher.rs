

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub fn pattern_match(pattern : &Pattern, data : &Data) -> Vec<MatchResult> {
    use Pattern::*;
    match pattern {
        CaptureVar(s) => vec![[(s.into(), data)].into()],
        As(s, p) => {
            let x = pattern_match(p, data);
            let as_result : MatchResult = [(s.into(), data)].into();
            x.into_iter().map(|mut y| { y.merge(as_result.clone()); y }).collect()
        },
        _ => todo!(),
    }
}

#[cfg(test)] 
mod test {
    use super::*;

    #[test]
    fn should_capture_var() {
        let data = Data::Number(Number::Usize(0));
        let results = pattern_match(&Pattern::CaptureVar("a".to_string()), &data);

        assert_eq!(results.len(), 1);
    } 
}
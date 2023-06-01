

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub fn pattern_match(pattern : &Pattern, data : &Data) -> Vec<MatchResult> {
    use Pattern::*;
    match (pattern, data) {
        (CaptureVar(s), data) => vec![[(s.into(), data)].into()],
        (As(s, p), data) => {
            let x = pattern_match(p, data);
            let as_result : MatchResult = [(s.into(), data)].into();
            x.into_iter().map(|mut y| { y.merge(as_result.clone()); y }).collect()
        },
        (Cons { name, params }, Data::Cons { name: cons_name, params: cons_params }) 
            if name == cons_name && params.len() == cons_params.len() => {

            let x = params.into_iter().zip(cons_params)
                                      .map(|(p, d)| pattern_match(p, d));
                                      //.fold(MatchResult::new(), |a, b| a.iter().flat_map(|alet| b.iter().map(move |b)))
            vec![]
        },
        (Cons { .. }, _) => vec![],
        _ => todo!(),
    }
}

/*fn combine_results(a : Vec<MatchResult>, b : Vec<MatchResult>) -> Vec<MatchResult> {
    a.iter().flat_map(|alet| b.iter().map(move |blet| {alet.merge(blet); alet})).map(|x| x.clone()).collect()
}*/

#[cfg(test)] 
mod test {
    use super::*;

    #[test]
    fn should_capture_var() {
        let data = Data::Number(Number::Usize(0));
        let pattern = Pattern::CaptureVar("a".to_string());
        let results = pattern_match(&pattern, &data);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get(&"a".into()), Some(&data));
    } 
}
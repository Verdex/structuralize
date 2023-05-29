

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

// TODO:  it would be nice if the vec was some sort of iter
pub fn pattern_match(pattern : &Pattern, data : &Data) -> Vec<MatchResult> {
    use Pattern::*;

    match pattern {
        CaptureVar(sym) => Ok(vec![ [(sym.into(), data)].into() ]),
        As(sym, pat) => {
            let 
            todo!()
        },
        _ => todo!(),
    }
}
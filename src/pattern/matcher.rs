
use std::collections::HashMap;

use crate::data::*;

use super::data::*;

// TODO:  it would be nice if the vec was some sort of iter
pub fn pattern_match(pattern : &Pattern, data : &Data) -> Result<Vec<MatchResult>, MatchError> {
    use Pattern::*;

    match pattern {
        CaptureVar(sym) => Ok(vec![ [(sym, data)].into() ]),
        As(sym, pat) => {
            todo!()
        },
        _ => todo!(),
    }
}
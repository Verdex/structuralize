
use super::data::*;
 
pub type PatternSig = Vec<Box<str>>;

#[derive(Debug)]
pub struct TypeChecked(Pattern, PatternSig);

#[derive(Debug)]
pub struct TypeMatches(Pattern);

#[derive(Debug)]
pub enum TypeCheckError {
    DuplicateSlot,
    OrPatternHasUnequalSig,
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeCheckError::DuplicateSlot => write!(f, "Pattern TypeCheckError: DuplicateSlot"),
            TypeCheckError::OrPatternHasUnequalSig=> write!(f, "Pattern TypeCheckError: OrPatternHasUnequalSig"),
        }
    }
}

impl std::error::Error for TypeCheckError { }

pub fn check_pattern(pattern : Pattern) -> Result<TypeChecked, TypeCheckError> {
    Err(TypeCheckError::DuplicateSlot)
}

pub fn pattern_sig(pattern : &Pattern) -> Result<PatternSig, TypeCheckError> {
    use Pattern::*;
    const EMPTY : Result<PatternSig, TypeCheckError> = Ok(vec![]);

    macro_rules! star {
        ($input : expr) => {{
            let mut sig = vec![];
            for i in $input {
                let s = pattern_sig(i)?;
                sig.push(s);
            }

            sig.sort();
            let total = sig.len();
            sig.dedup();

            if total != sig.len() {
                Err(TypeCheckError::DuplicateSlot)
            }
            else {
                Ok(sig.into_iter().flatten().collect())
            }
        }};
    }

    match pattern {
        Number(_) => EMPTY,
        String(_) => EMPTY, 
        Symbol(_) => EMPTY,
        Wild => EMPTY,
        CaptureVar(v) => Ok(vec![v.clone()]),
        Cons { name: _, params } => EMPTY,
        Struct { name: _, fields } => EMPTY,
        ExactList(ps) => star!(ps),
        ListPath(ps) => star!(ps),
        PathNext => EMPTY, 
        Path(ps) => star!(ps),
        And(a, b) => { 
            star!([a, b])
        },
        Or(a, b) => {
            let a_sig = pattern_sig(a)?;
            let b_sig = pattern_sig(b)?;

            if a_sig != b_sig {
                Error(TypeCheckError::OrPatternHasUnequalSig)
            }
            else {
                Ok(a_sig)
            }
        },
    }
}

    // TODO allow parsing path next whereever, but make sure the type check
    // prevents path next anywhere except inside of a path pattern

// TODO make sure that pattern type checks ensure that slot names don't collide and that
// if there are OR patterns both have the same 'signature'
// also need to have something for function patterns

// TODO:  phantom type type checked patterns
// * in a path each step needs at least one next except for the last one which cannot have any nexts
// * make sure that structs don't have duplicate field names
// * also make sure that struct fields are sorted

            
// Note:  'Typechecking' will process structs such that their fields are sorted
// TODO: this isn't going to work unless you also sort the data



        // TODO do empty cons need to be prevented? (let's say yes)

#[cfg(test)]
mod test {
    use super::*;

}
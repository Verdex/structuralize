
use super::data::*;
 
pub type PatternSig = Vec<Box<str>>;

#[derive(Debug)]
pub struct TypeChecked(Pattern, PatternSig);

impl TypeChecked {
    pub fn pattern<'a>(&'a self) -> &'a Pattern {
        &self.0
    }
    pub fn signature<'a>(&'a self) -> &'a PatternSig {
        &self.1
    }
}

#[derive(Debug)]
pub struct TypeMatches(TypeChecked);

#[derive(Debug)]
pub enum TypeCheckError {
    DuplicateSlot,
    OrPatternHasUnequalSig,
    TypeDoesNotMatch { found: PatternSig, expected: PatternSig }
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        use TypeCheckError::*;
        match self {
            DuplicateSlot => write!(f, "Pattern TypeCheckError: DuplicateSlot"),
            OrPatternHasUnequalSig => write!(f, "Pattern TypeCheckError: OrPatternHasUnequalSig"),
            TypeDoesNotMatch { found, expected } => write!(f, "Pattern TypeCheckError: Types do not match.  Found {:?}, but expected {:?}", found, expected),
        }
    }
}

impl std::error::Error for TypeCheckError { }

pub fn check_pattern(pattern : Pattern) -> Result<TypeChecked, TypeCheckError> {
    let sig = pattern_sig(&pattern)?;
    Ok(TypeChecked(pattern, sig))
}

fn check_next_usage(pattern : &Pattern) -> bool {
    fn r(pattern : &Pattern, in_path : bool) -> Option<usize> {
        use Pattern::*;
        match pattern {
            Number(_) => Some(0),
            String(_) => Some(0), 
            Symbol(_) => Some(0),
            Wild => Some(0),
            CaptureVar(_) => Some(0),
            Cons { params, .. } => params.iter().map(|p| r(p, in_path)).sum(),
            Struct { fields: fs, .. } => fs.iter().map(|(_, p)| r(p, in_path)).sum(),
            ExactList(ps) => ps.iter().map(|p| r(p, in_path)).sum(),
            ListPath(ps) => ps.iter().map(|p| r(p, in_path)).sum(),
            PathNext if in_path => Some(1),
            PathNext => None,
            Path(ps) if ps.len() == 0 => Some(0),
            Path(ps) => {
                let last_count = r(ps.last().unwrap(), true);
                if ! matches!(last_count, Some(0)) {
                    None
                }
                else {
                    let l = ps.len() - 1;
                    if (&ps[..l]).iter().map(|p| r(p, true)).all(|x| match x { Some(v) if v > 0 => true, _ => false }) {
                        Some(0)
                    }
                    else {
                        None
                    }
                }
            },
            And(a, b) => [r(&**a, in_path), r(&**b, in_path)].into_iter().sum(),
            Or(a, b) => [r(&**a, in_path), r(&**b, in_path)].into_iter().sum(),
        }
    }

    r(pattern, false).is_some()
}

pub fn pattern_sig_matches(pattern : TypeChecked, sig : PatternSig) -> Result<TypeMatches, TypeCheckError> {
    if pattern.signature() == &sig {
        Ok(TypeMatches(pattern))
    }
    else {
        Err(TypeCheckError::TypeDoesNotMatch { found: pattern.signature().clone(), expected: sig.clone() })
    }
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
                Err(TypeCheckError::OrPatternHasUnequalSig)
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
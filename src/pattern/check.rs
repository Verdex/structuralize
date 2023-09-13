
use denest::Linearizable;

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
    IncorrectNextUsage,
    ConsPatternsNeedAtLeastOneParam,
    TypeDoesNotMatch { found: PatternSig, expected: PatternSig },
    FuncReferencesUnknownCaptureVariable(Box<str>),
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        use TypeCheckError::*;
        match self {
            DuplicateSlot => write!(f, "Pattern TypeCheckError: DuplicateSlot"),
            OrPatternHasUnequalSig => write!(f, "Pattern TypeCheckError: OrPatternHasUnequalSig"),
            IncorrectNextUsage => write!(f, "Pattern TypeCheckError: IncorrectNextUsage"),
            ConsPatternsNeedAtLeastOneParam => write!(f, "Pattern TypeCheckError: ConsPatternsNeedAtLeastOneParam"),
            TypeDoesNotMatch { found, expected } => write!(f, "Pattern TypeCheckError: Types do not match.  Found {:?}, but expected {:?}", found, expected),
            FuncReferencesUnknownCaptureVariable(var) => write!(f, "Pattern TypeCheckError:  Function references unknown variable: {}", var),
        }
    }
}

impl std::error::Error for TypeCheckError { }

pub fn check_pattern(pattern : Pattern) -> Result<TypeChecked, TypeCheckError> {

    if ! check_next_usage(&pattern) {
        return Err(TypeCheckError::IncorrectNextUsage);
    }

    if ! pattern.to_lax().map(|p| check_cons_have_params(p)).all(|x| x) {
        return Err(TypeCheckError::ConsPatternsNeedAtLeastOneParam);
    }

    if let Some(error) = check_template_usage(&pattern) {
        return Err(error);
    }

    let sig = pattern_sig(&pattern)?;
    Ok(TypeChecked(pattern, sig))
}

fn check_cons_have_params(pattern : &Pattern) -> bool {
    match pattern {
        Pattern::Cons { params, .. } if params.len() == 0 => false,
        _ => true,
    }
}

fn check_template_usage(pattern : &Pattern) -> Option<TypeCheckError> {
    fn problem(x : &Option<TypeCheckError>) -> bool {
        match x {
            Some(_) => true,
            None => false,
        }
    }

    fn r(pattern : &Pattern, available_captures : &mut Vec<Box<str>>, in_func : bool) -> Option<TypeCheckError> {
        use Pattern::*;
        match pattern {
            String(_) => None, 
            Symbol(_) => None,
            Wild => None,
            CaptureVar(var) => { available_captures.push(var.clone()); None },
            Cons { params, .. } => params.iter().map(|p| r(p, available_captures, in_func)).find(problem)?,
            ExactList(ps) => ps.iter().map(|p| r(p, available_captures, in_func)).find(problem)?,
            ListPath(ps) => ps.iter().map(|p| r(p, available_captures, in_func)).find(problem)?,
            PathNext => None,
            Path(ps) => ps.iter().map(|p| r(p, available_captures, in_func)).find(problem)?, 

            And(a, b) => r(a, available_captures, in_func).or(r(b, available_captures, in_func)),
            Or(a, b) => r(a, available_captures, in_func).or(r(b, available_captures, in_func)),
            Func(p) => r(p, available_captures, true), 
            TemplateVar(var) if available_captures.iter().find(|x| *x == var).is_none()
                => Some(TypeCheckError::FuncReferencesUnknownCaptureVariable(var.clone())),
            TemplateVar(_) => None, 
        }
    }

    r(pattern, &mut vec![], false)
}

fn check_next_usage(pattern : &Pattern) -> bool {
    fn sgtz(input : Option<usize>) -> bool {
        match input { Some(v) if v > 0 => true, _ => false }
    }
    fn r(pattern : &Pattern, in_path : bool) -> Option<usize> {
        use Pattern::*;
        match pattern {
            String(_) => Some(0), 
            Symbol(_) => Some(0),
            Wild => Some(0),
            CaptureVar(_) => Some(0),
            Cons { params, .. } => params.iter().map(|p| r(p, in_path)).sum(),
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
                    if (&ps[..l]).iter().map(|p| r(p, true)).all(sgtz) {
                        Some(0)
                    }
                    else {
                        None
                    }
                }
            },
            And(a, b) => [r(&**a, in_path), r(&**b, in_path)].into_iter().sum(),
            Or(a, b) => {
                let a_s = r(&**a, in_path);
                let b_s = r(&**b, in_path);

                if a_s.is_none() || b_s.is_none() {
                    None
                }
                else if !sgtz(a_s) || !sgtz(b_s) {
                    Some(0)
                }
                else {
                    Some(1)
                }
            },
            Func(p) => r(p, in_path), 
            TemplateVar(_) => Some(0),
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

            let mut sig = sig.into_iter().flatten().collect::<Vec<_>>();

            sig.sort();
            let total = sig.len();
            sig.dedup();

            if total != sig.len() {
                Err(TypeCheckError::DuplicateSlot)
            }
            else {
                Ok(sig)
            }
        }};
    }

    match pattern {
        String(_) => EMPTY, 
        Symbol(_) => EMPTY,
        Wild => EMPTY,
        CaptureVar(v) => Ok(vec![v.clone()]),
        Cons { params, .. } => star!(params),
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
        Func(p) => pattern_sig(p), 
        TemplateVar(_) => EMPTY,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_template_usage_should_pass() {
        fn t(input : &str) {
            let p : Pattern = input.parse().unwrap();
            let output = check_template_usage(&p);
            assert!(output.is_none(), "{input}");
        }

        t(":symbol");
        t("[:one, :two, :three]");
        t("[\"1\",:symbol, :three]");
        t("cons(:a, :b, :c)");
        t("cons(:a, [:a, :b], :c)");
        t("[| a, _, c |]");
        t("x");
        t("\"x\"");
        t("\"x\".and( :symbol )");
        t("\"x\".or( :symbol )");
        t("{| [ {| ^, :zero |}, ^], :four |}");
        t("[| a, b, c |]");


        t("[:one, :two, :three]");
        t("[a, b, [$a, $b], :three]");
        t("[a, b, $a, :three]");
        t("[a, b, [cons($a, $b, c, $c)], :three]");
        t("{| [^, a], $a |}");
        t("[| a, b, [$a, $b] |]");
    }
    
    #[test]
    fn check_template_usage_should_not_pass() {
        fn t(input : &str) {
            let p : Pattern = input.parse().unwrap();
            let output = check_template_usage(&p);
            assert!(output.is_some(), "{input}");
        }

        t("[$a]");
        t("cons($a)");
        t("[|$a|]");
        t("{| ^, $a |}");
        t("[a, b, [$a, $c], :three]");
        t("[a, b, $c, :three]");
        t("[a, b, [cons($a, $c, c, $c)], :three]");
        t("{| [^, a], $c |}");
        t("[| a, b, [$c, $b] |]");
    }

    #[test]
    fn check_next_usage_should_pass() {
        fn t(input : &str) {
            let p : Pattern = input.parse().unwrap();
            let output = check_next_usage(&p);
            assert!(output, "{input}");
        }

        t(":symbol");
        t("[:one, :two, :three]");
        t("[\"1\",:symbol, :three]");
        t("cons(:a, :b, :c)");
        t("cons(:a, [:a, :b], :c)");
        t("[| a, _, c |]");
        t("x");
        t("\"x\"");
        t("\"x\".and( :symbol )");
        t("\"x\".or( :symbol )");

        t("{|  |}");
        t("{| x |}");
        t("{| [^, ^], :five |}");
        t("{| [cons( [:eight].and(^) ), :six], :five |}");
        t("{| [cons( [^].or(^) ), :six], :five |}");
        t("{| ^, cons(cons(^, ^)), a |}");
        t("{| [ {| ^, :zero |}, ^], :four |}");
    }

    #[test]
    fn check_next_usage_should_fail() {
        fn t(input : &str) {
            let p : Pattern = input.parse().unwrap();
            let output = check_next_usage(&p);
            assert!(!output, "{input}");
        }

        t("{| [cons( [:eight].or(^) ), :six], :five |}"); 
        t("{| {| ^, :zero |}, :four |}");
        t("{| cons(^), ^ |}");
        t("cons(^)");
    }
}

use crate::data::*;

use denest::*;

pub enum MatchKind<'a, TMatchable : Matchable> {
    Atom(&'a TMatchable::Atom),
    Object(&'a TMatchable::Object),
    Cons(&'a str, &'a [TMatchable]),
    List(&'a [TMatchable]),
}

pub trait Matchable {
    type Atom : Clone + PartialEq;
    type Object; 

    fn kind(&self) -> MatchKind<Self> where Self : Sized;
    fn to_pattern(&self) -> Pattern<Self::Atom>;
}

impl Matchable for Data {
    type Atom = SymStr;
    type Object = ();

    fn kind(&self) -> MatchKind<Self> {
        match self {
            Data::SymStr(s) => MatchKind::Atom(s),
            Data::Cons { name, params } => MatchKind::Cons(name, params),
            Data::List(ds) => MatchKind::List(ds),
        }
    }

    fn to_pattern(&self) -> Pattern<Self::Atom> {
        match self {
            Data::SymStr(s) => Pattern::Atom(s.clone()), 
            Data::Cons { name, params } => Pattern::Cons { name: name.clone(), params: params.iter().map(|x| x.to_pattern()).collect() },
            Data::List(ds) => Pattern::ExactList(ds.iter().map(|x| x.to_pattern()).collect()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern<TAtom : Clone> {
    Atom(TAtom),
    Fail,
    Wild,
    CaptureVar(Box<str>),
    Cons { name: Box<str>, params: Vec<Pattern<TAtom>> },
    ExactList(Vec<Pattern<TAtom>>),
    ListPath(Vec<Pattern<TAtom>>),
    PathNext,
    Path(Vec<Pattern<TAtom>>),
    And(Box<Pattern<TAtom>>, Box<Pattern<TAtom>>),
    Or(Box<Pattern<TAtom>>, Box<Pattern<TAtom>>),
    TemplateVar(Box<str>), 
}

impl<'a, T : Clone> Linearizable<'a> for Pattern<T> {
    fn l_next(&'a self) -> Vec<&'a Self> {
        use Pattern::*;
        match self {
            Atom(_) => vec![], 
            Fail => vec![],
            Wild => vec![],
            CaptureVar(_) => vec![],
            Cons { params, name: _ } => params.iter().collect(),
            ExactList(ps) => ps.iter().collect(),
            ListPath(ps) => ps.iter().collect(),
            PathNext => vec![],
            Path(ps) => ps.iter().collect(),
            And(a, b) => vec![&**a, &**b],
            Or(a, b) => vec![&**a, &**b],
            TemplateVar(_) => vec![],
        }
    }
}

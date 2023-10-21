
use denest::*;


pub enum MatchKind<'a, TMatchable : Matchable> {
    Atom(TMatchable::Atom),
    Object(TMatchable::Object),
    Cons(Box<str>, &'a [TMatchable]),
    List(&'a [TMatchable]),
}

pub trait Matchable {
    type Atom : PartialEq;
    type Object;

    fn kind(&self) -> MatchKind<Self> where Self : Sized;
}

#[derive(Debug, Clone)]
pub enum Pattern {
    String(Box<str>), 
    Symbol(Box<str>),
    Wild,
    CaptureVar(Box<str>),
    Cons { name: Box<str>, params: Vec<Pattern> },
    ExactList(Vec<Pattern>),
    ListPath(Vec<Pattern>),
    PathNext,
    Path(Vec<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
    TemplateVar(Box<str>), 
}

impl<'a> Linearizable<'a> for Pattern {
    fn l_next(&'a self) -> Vec<&'a Self> {
        use Pattern::*;
        match self {
            String(_) => vec![], 
            Symbol(_) => vec![],
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

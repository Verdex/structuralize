
#[derive(Debug, Clone)]
pub enum Pattern {
    Var(String),
    As(String, Box<Pattern>),
    Cons { name: String, params: Vec<Pattern> },
}

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work
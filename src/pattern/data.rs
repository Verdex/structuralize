
#[derive(Debug, Clone)]
pub enum Pattern {
    Var(String),
    As(String, Box<Pattern>),
    Cons { name: String, params: Vec<Pattern> },
}

// TODO probably need comparison patterns in order to avoid needing expressions
    // This probably means that if-patterns aren't going to work

#[derive(Debug)]
pub enum MatchError {

}

impl std::fmt::Display for MatchError {
   fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TODO")
    }
}

impl std::error::Error for MatchError {}
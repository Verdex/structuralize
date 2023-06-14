

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub struct MatchResults {

}

impl Iterator for MatchResults {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn pattern_match(pattern : &Pattern, data : &Data) -> MatchResults {
    MatchResults{}
}

#[cfg(test)] 
mod test {
    use super::*;

}
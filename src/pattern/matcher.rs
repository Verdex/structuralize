

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns

pub struct MatchResults<'a, 'b> {
    target : Vec<(&'a Pattern, &'b Data)>,
}

impl<'a, 'b> Iterator for MatchResults<'a, 'b> {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.target.pop();

        match x {
            (Pattern::CaptureVar(name), data) => {

            }
        }

        None
    }
}

pub fn pattern_match<'a, 'b>(pattern : &'a Pattern, data : &'b Data) -> MatchResults<'a, 'b> {
    MatchResults { target : vec![(pattern, data)]}
}

#[cfg(test)] 
mod test {
    use super::*;

}


// TODO:  Implement FromStr trait

use super::data::*;

impl std::str::FromStr for Data {
    type Err = u8; // TODO ???

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        Err(0)        
    }
}
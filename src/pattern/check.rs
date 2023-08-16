
use super::data::*;
 
pub fn check_pattern(pattern : &Pattern) -> bool {
    true
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
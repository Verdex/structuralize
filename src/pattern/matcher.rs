

use crate::data::*;

use super::data::*;

// TODO:  phantom type type checked patterns
// * in a path each step needs at least one next except for the last one which cannot have any nexts

pub struct MatchResults<'a, 'b> {
    pattern : &'a Pattern,
    data : &'b Data,
}

#[derive(Debug)]
pub enum DataPattern<'a> {
    CaptureVar(Box<str>, &'a Data),
    Cons { name: Box<str>, params: Vec<DataPattern<'a>> },
}

fn blarg<'a>(pattern : &Pattern, data : &'a Data) -> Option<DataPattern<'a>> {
    match (pattern, data) { 
    }
}


impl<'a, 'b> Iterator for MatchResults<'a, 'b> {
    type Item = MatchResult<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        let pattern = self.pattern;
        let data = self.data;
        None
    }
}

pub fn pattern_match<'a, 'b>(pattern : &'a Pattern, data : &'b Data) -> MatchResults<'a, 'b> {
    MatchResults { pattern, data }
}

#[cfg(test)] 
mod test {
    use super::*;

    // TODO we need this anyway for the hopefully upcoming type check
    fn p(input : &'static str) -> Pattern {
        input.parse().unwrap()
    }

    // TODO : path pattern that has path patterns inside of it (needs more impl before this will work)

    #[test]
    fn should_match_nested_nexts_in_path() {
        let pattern = p("{| cons(cons(^, ^), ^), [^], x |}");
        let data : Data = "cons(cons([:a], [:b]), [:c])".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 3);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":c".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());

        let observed = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed, &":b".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_only_valid_paths() {
        let pattern = p("{| cons(^, ^), [^], x |}");
        let data : Data = "cons([:a], :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_not_match_path() {
        let pattern = p("{| cons(:whatever) |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_empty_path() {
        let pattern = p("{| |}");
        let data : Data = ":whatever".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_multi_step_multi_next_path() {
        let pattern = p("{| cons(^, ^), [^, ^], x |}");
        let data : Data = "cons( [:a, :b], [:c, :d] )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 4);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":d".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":c".parse::<Data>().unwrap());

        let observed = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed, &":b".parse::<Data>().unwrap());

        let observed = results[3].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_path() {
        let pattern = p("cons( {| cons(^, ^), [^], x |}, {| cons(^, ^), [^], y |} )");
        let data : Data = "cons( cons([:a], [1.1]), cons([:b], [2.2]) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 4);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_y= results[0].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &"2.2".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_y= results[1].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &"2.2".parse::<Data>().unwrap());

        let observed_x = results[2].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_y= results[2].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());

        let observed_x = results[3].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_y= results[3].get(&"y".into()).unwrap();
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_capture_after() {
        let pattern = p("cons( {| cons(^, ^), [^], x |}, outer )");
        let data : Data = "cons( cons([:a], [1.1]), :outer )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_outer = results[0].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_outer = results[1].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_and_capture_before() {
        let pattern = p("cons( outer, {| cons(^, ^), [^], x |} )");
        let data : Data = "cons( :outer, cons([:a], [1.1]) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"1.1".parse::<Data>().unwrap());

        let observed_outer = results[0].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());

        let observed_x = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());

        let observed_outer = results[1].get(&"outer".into()).unwrap();
        assert_eq!(observed_outer, &":outer".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_multiple_paths_with_cons_and_list() { 
        let pattern = p("{| cons(^, ^), [^], x |}");
        let data : Data = "cons([:a], [1.1])".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 2);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &"1.1".parse::<Data>().unwrap());

        let observed = results[1].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_with_capture_before() {
        let pattern = p("{| cons(a, ^), [^], x |}");
        let data : Data = "cons(1.1, [:a])".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &"1.1".parse::<Data>().unwrap());

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_path_with_capture_after() {
        let pattern = p("{| cons(^, a), [^], x |}");
        let data : Data = "cons([:a], 1.1)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &"1.1".parse::<Data>().unwrap());

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }
    
    #[test]
    fn should_match_path_with_cons_and_list() {
        let pattern = p("{| cons(^, _), [^], x |}");
        let data : Data = "cons([:a], 1.1)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_due_to_number() {
        let pattern = p("cons(a, 1.1)");
        let data : Data = "cons(:a, 1.1)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_number() {
        let pattern = p("cons(a, 1.1)");
        let data : Data = "cons(:a, 1.2)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_wild() {
        let pattern = p("_");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_due_to_symbol() {
        let pattern = p("cons(a, :b)");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_symbol() {
        let pattern = p("cons(a, :a)");
        let data : Data = "cons(:a, :b)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_due_to_string() {
        let pattern = p("cons(a, \"leta\")");
        let data : Data = "cons(:a, \"leta\")".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"a".into()).unwrap();
        assert_eq!(observed, &":a".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_due_to_string() {
        let pattern = p("cons(a, \"leta\")");
        let data : Data = "cons(:a, \"letb\")".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_single_var() {
        let pattern = p("x");
        let data : Data = "cons(:a)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed, &data);
    }

    #[test]
    fn should_fail_match_due_to_nested_cons_internal_mismatch() {
        let pattern = p("cons( :a, :b, :c, cons(:x) )");
        let data : Data = "cons(:a, :b, :c, cons(:a) )".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_internal_mismatch() {
        let pattern = p("cons( :a, :b, :c, :x )");
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_name_mismatch() {
        let pattern = p("other( x, y, z )");
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_due_to_cons_length_mismatch() {
        let pattern = p("cons( x, y, z )");
        let data : Data = "cons(:a, :b, :c, :d)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_cons_with_vars() {
        let pattern = p("cons( x, y, z )");
        let data : Data = "cons(:a, :b, :c)".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        let observed_y = results[0].get(&"y".into()).unwrap();
        let observed_z = results[0].get(&"z".into()).unwrap();
        assert_eq!(observed_x, &":a".parse::<Data>().unwrap());
        assert_eq!(observed_y, &":b".parse::<Data>().unwrap());
        assert_eq!(observed_z, &":c".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_struct() {
        let pattern = p("struct { a: 1, b: 2, c: 3 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_match_struct_with_inner_var() {
        let pattern = p("struct { a: 1, b: 2, c: x }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);

        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"3".parse::<Data>().unwrap());
    }

    #[test]
    fn should_fail_match_struct_due_to_length() {
        let pattern = p("struct { a: 1, b: 2 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_struct_due_to_inner_name_mismatch() {
        let pattern = p("struct { a: 1, b: 2, x: 3 }");
        let data : Data = "struct { a: 1, b: 2, c: 3 }".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_match_exact_list() {
        let pattern = p("[1, x, :a]");
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        let observed_x = results[0].get(&"x".into()).unwrap();
        assert_eq!(observed_x, &"2".parse::<Data>().unwrap());
    }

    #[test]
    fn should_match_empty_exact_list() {
        let pattern = p("[]");
        let data : Data = "[]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_length() {
        let pattern = p("[1, x, :a, :x]");
        let data : Data = "[1, 2, :a]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_value() {
        let pattern = p("[1, x, :a, :x]");
        let data : Data = "[1, 2, :a, :y]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn should_fail_match_exact_list_due_to_nested_list_mismatch() {
        let pattern = p("[1, x, :a, [:x, :x]]");
        let data : Data = "[1, 2, :a, [:x, :y]]".parse().unwrap();

        let results = pattern_match(&pattern, &data).collect::<Vec<_>>();
        assert_eq!(results.len(), 0);
    }
}
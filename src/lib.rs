
mod parsing;
pub mod data;
pub mod pattern;


#[cfg(test)]
mod tests {

    macro_rules! t {
        ($name:ident $matcher:ident = pattern $pat:expr; data $dat:expr; $({ $($s:expr => $d:expr);* })* ) => {
            #[test]
            fn $name() {
                use std::collections::HashMap;

                let pattern : Pattern = $pat.parse().expect(&format!("{}", $pat));
                let data : Data = $dat.parse().expect(&format!("{}", $pat));

                let type_checked_pattern : TypeChecked = check_pattern(pattern).expect(&format!("{}", $pat));

                #[allow(unused_mut)]
                let mut results = $matcher(&type_checked_pattern, &data).into_iter()
                                                           .map(|x| x.into_iter().collect::<HashMap<_,_>>())
                                                           .collect::<Vec<_>>();

                $(
                    let _r = results.remove(0);

                    $(
                        let data = *_r.get(&$s.into()).unwrap();
                        assert_eq!( data, &$d.parse::<Data>().unwrap());
                    )*
                )*

                assert_eq!( results.len(), 0 );
            }
        };
    }

    macro_rules! all {
        ($target:ident) => {
            mod $target {
                use crate::data::*;
                use crate::pattern::*;
                use crate::pattern::check::*;

                t! { should_match_path_with_next_inside_list_path $target =
                        pattern "{| [| ^, 1 |], cons(:a, a) |}";
                        data "[ cons(:a, :b), 1, cons(:a, :c), 1, cons(:a, :d), 2, cons(:x, :e), 1, cons(:a, :f), 1]";
                        { "a" => ":b" }
                        { "a" => ":c" }
                        { "a" => ":f" }
                }

                t! { should_match_or_with_path $target =
                        pattern "{| [^, ^], cons(a, 1) |} |> or({| [^, ^], cons(a, 2) |})";
                        data "[cons(0, 2), cons(9, 2)]";
                        { "a" => "0" }
                        { "a" => "9" }
                }

                t! { should_match_and_with_list_path $target = 
                        pattern "[| a, b |] |> and( [| c, 5, d |] )";
                        data "[1, 5, 2, 3, 5, 4]";
                        { "a" => "1"; "b" => "5"; "c" => "1"; "d" => "2" }
                        { "a" => "5"; "b" => "2"; "c" => "1"; "d" => "2" }
                        { "a" => "2"; "b" => "3"; "c" => "1"; "d" => "2" }
                        { "a" => "3"; "b" => "5"; "c" => "1"; "d" => "2" }
                        { "a" => "5"; "b" => "4"; "c" => "1"; "d" => "2" }
                        { "a" => "1"; "b" => "5"; "c" => "3"; "d" => "4" }
                        { "a" => "5"; "b" => "2"; "c" => "3"; "d" => "4" }
                        { "a" => "2"; "b" => "3"; "c" => "3"; "d" => "4" }
                        { "a" => "3"; "b" => "5"; "c" => "3"; "d" => "4" }
                        { "a" => "5"; "b" => "4"; "c" => "3"; "d" => "4" }
                }

                t! { should_match_nested_or $target = 
                        pattern "[x, 9, 9] |> or( [0, x, 2] |> or([9, 9, x]) )";
                        data "[0, 1, 2]";
                        { "x" => "1" }
                }

                t! { should_match_nested_and $target = 
                        pattern "[x, 1, 2] |> and( [0, y, 2] |> and ([0, 1, z]) )";
                        data "[0, 1, 2]";
                        { "x" => "0"; "y" => "1"; "z" => "2" }
                }

                t! { should_match_and $target =
                        pattern "[a] |> and(b)";
                        data "[1]";
                        { "a" => "1"; "b" => "[1]" }
                }

                t! { should_match_chained_ands $target =
                        pattern "[a, _, _] |> and(b) |> and([_, :x, :y])";
                        data "[1, :x, :y]";
                        { "a" => "1"; "b" => "[1, :x, :y]" }
                }

                t! { should_not_match_chained_ands $target =
                        pattern "[a, _, _] |> and(7) |> and([_, :x, :y])";
                        data "[1, :x, :y]";
                }

                t! { should_match_or_with_first_passing $target = 
                        pattern "[x] |> or(cons(x))";
                        data "[1]";
                        { "x" => "1" }
                }

                t! { should_match_or_with_second_passing $target = 
                        pattern "[x] |> or(cons(x))";
                        data "cons(1)";
                        { "x" => "1" }
                }

                t! { should_match_or_both_passing $target = 
                        pattern "cons(x) |> or(cons(x))";
                        data "cons(1)";
                        { "x" => "1" }
                }

                t! { should_match_chained_ors $target =
                        pattern "[x, 1] |> or([x, 2]) |> or([x, 3])";
                        data "[0, 3]";
                        { "x" => "0" }
                }

                t! { should_match_with_path_inside_of_list_path $target = 
                        pattern "[| {| cons(^, ^), a |}, :target |]";
                        data "[ cons(1, 2), :target, cons(3, 4), :other ]";
                        { "a" => "1" }
                        { "a" => "2" }
                }

                t! { should_match_list_path_in_list_path $target = 
                        pattern "[| [| a, b |], [| c, d |] |]";
                        data "[ [1, 2, 3], [4, 5, 6], [7, 8, 9] ]";
                        { "a" => "1"; "b" => "2"; "c" => "4"; "d" => "5" }
                        { "a" => "2"; "b" => "3"; "c" => "4"; "d" => "5" }
                        { "a" => "1"; "b" => "2"; "c" => "5"; "d" => "6" }
                        { "a" => "2"; "b" => "3"; "c" => "5"; "d" => "6" }
                        { "a" => "4"; "b" => "5"; "c" => "7"; "d" => "8" }
                        { "a" => "5"; "b" => "6"; "c" => "7"; "d" => "8" }
                        { "a" => "4"; "b" => "5"; "c" => "8"; "d" => "9" }
                        { "a" => "5"; "b" => "6"; "c" => "8"; "d" => "9" }
                }

                t! { should_match_multiple_items_list_path $target =
                        pattern "[| a, b |]";
                        data "[1, 2, 3, 4, 5]";
                        { "a" => "1"; "b" => "2" }
                        { "a" => "2"; "b" => "3" }
                        { "a" => "3"; "b" => "4" }
                        { "a" => "4"; "b" => "5" }
                }

                t! { should_match_single_item_list_path $target = 
                        pattern "[| a |]";
                        data "[1, 2, 3, 4]";
                        { "a" => "1" }
                        { "a" => "2" }
                        { "a" => "3" }
                        { "a" => "4" }
                }

                t! { should_match_empty_list_path $target =
                        pattern "[| |]";
                        data "[:whatever]";
                        { }
                }

                t! { should_not_match_empty_list_path_against_non_list $target =
                        pattern "[| |]";
                        data ":whatever";
                }

                t! { should_not_match_list_path_with_more_patterns_than_list $target =
                        pattern "[| :a, _ |]";
                        data "[:a]";
                }

                t! { should_match_multiple_paths_in_cons $target = 
                        pattern "cons( {| cons(^, ^), [a, b] |}, {| cons(^, ^), [c, d] |} )";
                        data "cons( cons([1, 2], [3, 4]), cons([5, 6,], [7, 8]) )";
                        { "a" => "1"; "b" => "2"; "c" => "5"; "d" => "6" }
                        { "a" => "3"; "b" => "4"; "c" => "5"; "d" => "6" }
                        { "a" => "1"; "b" => "2"; "c" => "7"; "d" => "8" }
                        { "a" => "3"; "b" => "4"; "c" => "7"; "d" => "8" }
                }

                t! { should_match_multiple_paths_in_list $target = 
                        pattern "[ {| cons(^, ^), [a, b] |}, {| cons(^, ^), [c, d] |} ]";
                        data "[ cons([1, 2], [3, 4]), cons([5, 6,], [7, 8]) ]";
                        { "a" => "1"; "b" => "2"; "c" => "5"; "d" => "6" }
                        { "a" => "3"; "b" => "4"; "c" => "5"; "d" => "6" }
                        { "a" => "1"; "b" => "2"; "c" => "7"; "d" => "8" }
                        { "a" => "3"; "b" => "4"; "c" => "7"; "d" => "8" }
                }

                t! { should_match_path_pattern_inside_of_path_pattern $target = 
                        pattern "{| cons( {| inner(^, ^), inner(1, a, b) |} , ^, ^), outer(1, c, d) |}";
                        data "cons( inner(inner(1, :a, :b), inner(1, :c, :d)), outer(1, :e, :f), outer(1, :g, :h) )";
                        { "a" => ":a"; "b" => ":b"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":a"; "b" => ":b"; "c" => ":g"; "d" => ":h" }
                        { "a" => ":c"; "b" => ":d"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":c"; "b" => ":d"; "c" => ":g"; "d" => ":h" }
                }

                t! { should_match_path_pattern_inside_of_path_pattern_and_ignore_failures $target = 
                        pattern "{| cons( {| inner(^, ^), inner(1, a, b) |} , ^, ^), outer(1, c, d) |}";
                        data "cons( inner(inner(2, :a, :b), inner(1, :c, :d)), outer(1, :e, :f), outer(2, :g, :h) )";
                        { "a" => ":c"; "b" => ":d"; "c" => ":e"; "d" => ":f" }
                }

                t! { should_match_nested_nexts_in_path $target =
                        pattern "{| cons(cons(^, ^), ^), [^], x |}";
                        data "cons(cons([:a], [:b]), [:c])";
                        { "x" => ":a" }
                        { "x" => ":b" }
                        { "x" => ":c" }
                }

                t! { should_match_only_valid_paths $target =
                        pattern "{| cons(^, ^), [^], x |}";
                        data "cons([:a], :b)";
                        { "x" => ":a" }
                }

                t! { should_not_match_path $target = 
                        pattern "{| cons(:whatever) |}";
                        data ":whatever";
                }

                t! { should_match_empty_path $target =
                        pattern "{| |}";
                        data ":whatever";
                        { }
                }

                t! { should_match_multi_step_multi_next_path $target = 
                        pattern "{| cons(^, ^), [^, ^], x |}";
                        data "cons( [:a, :b], [:c, :d] )";
                        { "x" => ":a" }
                        { "x" => ":b" }
                        { "x" => ":c" }
                        { "x" => ":d" }
                }

                t! { should_match_path_and_path $target = 
                        pattern "cons( {| cons(^, ^), [^], x |}, {| cons(^, ^), [^], y |} )";
                        data "cons( cons([:a], [1.1]), cons([:b], [2.2]) )";
                        { "x" => ":a"; "y" => ":b" }
                        { "x" => "1.1"; "y" => ":b" }
                        { "x" => ":a"; "y" => "2.2" }
                        { "x" => "1.1"; "y" => "2.2" }
                }

                t! { should_match_path_and_capture_after $target =
                        pattern "cons( {| cons(^, ^), [^], x |}, outer )";
                        data "cons( cons([:a], [1.1]), :outer )";
                        { "x" => ":a"; "outer" => ":outer" }
                        { "x" => "1.1"; "outer" => ":outer" }
                }

                t! { should_match_path_and_capture_before $target =
                        pattern "cons( outer, {| cons(^, ^), [^], x |} )";
                        data "cons( :outer, cons([:a], [1.1]) )";
                        { "x" => ":a"; "outer" => ":outer" }
                        { "x" => "1.1"; "outer" => ":outer" }
                }

                t! { should_match_multiple_paths_with_cons_and_list $target =
                        pattern "{| cons(^, ^), [^], x |}";
                        data "cons([:a], [1.1])";
                        { "x" => ":a" }
                        { "x" => "1.1" }
                }

                t! { should_match_path_with_capture_before $target = 
                        pattern "{| cons(a, ^), [^], x |}";
                        data "cons(1.1, [:a])";
                        { "a" => "1.1"; "x" => ":a" }
                }

                t! { should_match_path_with_capture_after $target = 
                        pattern "{| cons(^, a), [^], x |}";
                        data "cons([:a], 1.1)";
                        { "a" => "1.1"; "x" => ":a" }
                }

                t! { should_match_path_with_cons_and_list $target = 
                        pattern "{| cons(^, _), [^], x |}";
                        data "cons([:a], 1.1)";
                        { "x" => ":a" }
                }

                t! { should_match_due_to_number $target = 
                        pattern "cons(a, 1.1)";
                        data "cons(:a, 1.1)";
                        { "a" => ":a" }
                }
                
                t! { should_fail_match_due_to_number $target =
                        pattern "cons(a, 1.1)";
                        data "cons(:a, 1.2)";
                }

                t! { should_match_wild $target =
                        pattern "_";
                        data "cons(:a, :b)";
                        { }
                }

                t! { should_match_due_to_symbol $target = 
                        pattern "cons(a, :b)";
                        data "cons(:a, :b)";
                        { "a" => ":a" }
                }

                t! { should_fail_match_due_to_symbol $target = 
                        pattern "cons(a, :a)";
                        data "cons(:a, :b)";
                }

                t! { should_match_due_to_string $target = 
                        pattern "cons(a, \"leta\")";
                        data "cons(:a, \"leta\")";
                        { "a" => ":a" }
                }

                t! { should_fail_match_due_to_string $target = 
                        pattern "cons(a, \"leta\")";
                        data "cons(:a, \"letb\")";
                }

                t! { should_match_single_var $target =
                        pattern "x";
                        data "cons(:a)";
                        { "x" => "cons(:a)"}
                }

                t! { should_fail_match_due_to_nested_cons_internal_mismatch $target =
                        pattern "cons( :a, :b, :c, cons(:x) )";
                        data "cons(:a, :b, :c, cons(:a) )";
                }

                t! { should_fail_match_due_to_cons_internal_mismatch $target = 
                        pattern "cons( :a, :b, :c, :x )";
                        data "cons(:a, :b, :c, :d)";
                }

                t! { should_fail_match_due_to_cons_name_mismatch $target =
                        pattern "other( x, y, z )";
                        data "cons(:a, :b, :c)";
                }

                t! { should_match_exact_list $target = 
                        pattern "[1, x, :a]";
                        data "[1, 2, :a]";
                        { "x" => "2" }
                }

                t! { should_match_empty_exact_list $target =
                        pattern "[]";
                        data "[]";
                        { }
                }

                t! { should_fail_match_exact_list_due_to_length $target = 
                        pattern "[1, x, :a, :x]";
                        data "[1, 2, :a]";
                }

                t! { should_fail_match_exact_list_due_to_value $target = 
                        pattern "[1, x, :a, :x]";
                        data "[1, 2, :a, :y]";
                }

                t! { should_fail_match_exact_list_due_to_nested_list_mismatch $target =
                        pattern "[1, x, :a, [:x, :x]]";
                        data "[1, 2, :a, [:x, :y]]";
                }

                t!{ should_fail_match_due_to_cons_length_mismatch $target =
                        pattern "cons( x, y, z )";
                        data "cons(:a, :b, :c, :d)";
                }

                t!{ should_match_cons_with_vars $target = 
                        pattern "cons( x, y, z )";
                        data "cons(:a, :b, :c)"; 
                        { "x" => ":a"; "y" => ":b"; "z" => ":c" } 
                }
            }
        };
    }

    all!(pattern_match);

    #[test]
    fn should_try_it() {

        use crate::data::*;
        use crate::pattern::*;
        use crate::pattern::check::*;

        let pattern : Pattern = "[| [| a, b |], [| c, d |] |]".parse().unwrap();
        let data : Data = "[ [1, 2, 3], [4, 5, 6], [7, 8, 9] ]".parse().unwrap();

        let type_checked_pattern : TypeChecked = check_pattern(pattern).unwrap();

        let results = pattern_match(&type_checked_pattern, &data);
        for r in results {
            println!("{:?}\n\n", r);
        }
    }
}

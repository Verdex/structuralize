
mod parsing;
pub mod data;
pub mod pattern;


#[cfg(test)]
mod tests {

    macro_rules! t {
        ($name:ident $matcher:ident = pattern $pat:expr; data $dat:expr; $({ $($s:expr => $d:expr);* })* ) => {
            #[test]
            fn $name() {
                #[allow(unused_imports)]
                use std::collections::HashMap;

                let pattern : Pattern = $pat.parse().expect(&format!("{}", $pat));
                let data : Data = $dat.parse().expect(&format!("{}", $pat));

                let type_checked_pattern : TypeChecked = check_pattern(pattern).expect(&format!("{}", $pat));

                #[allow(unused_mut)]
                let mut results = $matcher(&type_checked_pattern, &data).into_iter().collect::<Vec<_>>();

                $(
                    let r = results.remove(0);
                    let r_len = r.len();

                    let _r = r.into_iter().collect::<HashMap<_, _>>();

                    #[allow(unused_mut)]
                    let mut expected_map_count = 0;

                    $(
                        let data = *_r.get(&$s.into()).unwrap();
                        expected_map_count += 1;
                        assert_eq!( data, &$d.parse::<Data>().unwrap(), "{} found incorrect result", $s);
                    )*

                    assert_eq!( r_len, expected_map_count, "more captures found than were tested" );
                )*

                assert_eq!( results.len(), 0, "more results than were tested" );
            }
        };
    }

    macro_rules! all {
        ($target:ident) => {
            mod $target {
                use crate::data::*;
                use crate::pattern::*;
                use crate::pattern::lazy_matcher::pattern_match as lazy_pattern_match;
                use crate::pattern::check::*;

                t! { should_match_path_with_next_inside_list_path $target =
                        pattern "{| [| ^, :one |], cons(:a, a) |}";
                        data "[ cons(:a, :b), :one, cons(:a, :c), :one, cons(:a, :d), :two, cons(:x, :e), :one, cons(:a, :f), :one]";
                        { "a" => ":b" }
                        { "a" => ":c" }
                        { "a" => ":f" }
                }

                t! { should_match_or_with_path $target =
                        pattern "{| [^, ^], cons(a, :one) |}.or({| [^, ^], cons(a, :two) |})";
                        data "[cons(:zero, :two), cons(:nine, :two)]";
                        { "a" => ":zero" }
                        { "a" => ":nine" }
                }

                t! { should_match_and_with_list_path $target = 
                        pattern "[| a, b |].and( [| c, :five, d |] )";
                        data "[:one, :five, :two, :three, :five, :four]";
                        { "a" => ":one";   "b" => ":five";  "c" => ":one";   "d" => ":two" }
                        { "a" => ":one";   "b" => ":five";  "c" => ":three"; "d" => ":four" }
                        { "a" => ":five";  "b" => ":two";   "c" => ":one";   "d" => ":two" }
                        { "a" => ":five";  "b" => ":two";   "c" => ":three"; "d" => ":four" }
                        { "a" => ":two";   "b" => ":three"; "c" => ":one";   "d" => ":two" }
                        { "a" => ":two";   "b" => ":three"; "c" => ":three"; "d" => ":four" }
                        { "a" => ":three"; "b" => ":five";  "c" => ":one";   "d" => ":two" }
                        { "a" => ":three"; "b" => ":five";  "c" => ":three"; "d" => ":four" }
                        { "a" => ":five";  "b" => ":four";  "c" => ":one";   "d" => ":two" }
                        { "a" => ":five";  "b" => ":four";  "c" => ":three"; "d" => ":four" }
                }

                t! { should_match_nested_or $target = 
                        pattern "[x, :nine, :nine].or( [:zero, x, :two].or([:nine, :nine, x]) )";
                        data "[:zero, :one, :two]";
                        { "x" => ":one" }
                }

                t! { should_match_nested_and $target = 
                        pattern "[x, :one, :two].and( [:zero, y, :two].and([:zero, :one, z]) )";
                        data "[:zero, :one, :two]";
                        { "x" => ":zero"; "y" => ":one"; "z" => ":two" }
                }

                t! { should_match_and $target =
                        pattern "[a].and(b)";
                        data "[:one]";
                        { "a" => ":one"; "b" => "[:one]" }
                }

                t! { should_match_chained_ands $target =
                        pattern "[a, _, _].and(b).and([_, :x, :y])";
                        data "[:one, :x, :y]";
                        { "a" => ":one"; "b" => "[:one, :x, :y]" }
                }

                t! { should_not_match_chained_ands $target =
                        pattern "[a, _, _].and(:seven).and([_, :x, :y])";
                        data "[:one, :x, :y]";
                }

                t! { should_match_or_with_first_passing $target = 
                        pattern "[x].or(cons(x))";
                        data "[:one]";
                        { "x" => ":one" }
                }

                t! { should_match_or_with_second_passing $target = 
                        pattern "[x].or(cons(x))";
                        data "cons(:one)";
                        { "x" => ":one" }
                }

                t! { should_match_or_both_passing $target = 
                        pattern "cons(x).or(cons(x))";
                        data "cons(:one)";
                        { "x" => ":one" }
                }

                t! { should_match_chained_ors $target =
                        pattern "[x, :one].or([x, :two]).or([x, :three])";
                        data "[:zero, :three]";
                        { "x" => ":zero" }
                }

                t! { should_match_with_path_inside_of_list_path $target = 
                        pattern "[| {| cons(^, ^), a |}, :target |]";
                        data "[ cons(:one, :two), :target, cons(:three, :four), :other ]";
                        { "a" => ":one" }
                        { "a" => ":two" }
                }

                t! { should_match_list_path_in_list_path $target = 
                        pattern "[| [| a, b |], [| c, d |] |]";
                        data "[ [:a, :b, :c], [:d, :e, :f], [:g, :h, :i] ]";
                        { "a" => ":a"; "b" => ":b"; "c" => ":d"; "d" => ":e" }
                        { "a" => ":a"; "b" => ":b"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":b"; "b" => ":c"; "c" => ":d"; "d" => ":e" }
                        { "a" => ":b"; "b" => ":c"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":d"; "b" => ":e"; "c" => ":g"; "d" => ":h" }
                        { "a" => ":d"; "b" => ":e"; "c" => ":h"; "d" => ":i" }
                        { "a" => ":e"; "b" => ":f"; "c" => ":g"; "d" => ":h" }
                        { "a" => ":e"; "b" => ":f"; "c" => ":h"; "d" => ":i" }
                }

                t! { should_match_multiple_items_list_path $target =
                        pattern "[| a, b |]";
                        data "[:a, :b, :c, :d, :e]";
                        { "a" => ":a"; "b" => ":b" }
                        { "a" => ":b"; "b" => ":c" }
                        { "a" => ":c"; "b" => ":d" }
                        { "a" => ":d"; "b" => ":e" }
                }

                t! { should_match_single_item_list_path $target = 
                        pattern "[| a |]";
                        data "[:one, :two, :three, :four]";
                        { "a" => ":one" }
                        { "a" => ":two" }
                        { "a" => ":three" }
                        { "a" => ":four" }
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
                        data "cons( cons([:one, :two], [:three, :four]), cons([:five, :six,], [:seven, :eight]) )";
                        { "a" => ":one";   "b" => ":two";  "c" => ":five";  "d" => ":six" }
                        { "a" => ":one";   "b" => ":two";  "c" => ":seven"; "d" => ":eight" }
                        { "a" => ":three"; "b" => ":four"; "c" => ":five";  "d" => ":six" }
                        { "a" => ":three"; "b" => ":four"; "c" => ":seven"; "d" => ":eight" }
                }

                t! { should_match_multiple_paths_in_list $target = 
                        pattern "[ {| cons(^, ^), [a, b] |}, {| cons(^, ^), [c, d] |} ]";
                        data "[ cons([:one, :two], [:three, :four]), cons([:five, :six,], [:seven, :eight]) ]";
                        { "a" => ":one";   "b" => ":two";  "c" => ":five";  "d" => ":six" }
                        { "a" => ":one";   "b" => ":two";  "c" => ":seven"; "d" => ":eight" }
                        { "a" => ":three"; "b" => ":four"; "c" => ":five";  "d" => ":six" }
                        { "a" => ":three"; "b" => ":four"; "c" => ":seven"; "d" => ":eight" }
                }

                t! { should_match_path_pattern_inside_of_path_pattern $target = 
                        pattern "{| cons( {| inner(^, ^), inner(:one, a, b) |} , ^, ^), outer(:one, c, d) |}";
                        data "cons( inner(inner(:one, :a, :b), inner(:one, :c, :d)), outer(:one, :e, :f), outer(:one, :g, :h) )";
                        { "a" => ":a"; "b" => ":b"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":a"; "b" => ":b"; "c" => ":g"; "d" => ":h" }
                        { "a" => ":c"; "b" => ":d"; "c" => ":e"; "d" => ":f" }
                        { "a" => ":c"; "b" => ":d"; "c" => ":g"; "d" => ":h" }
                }

                t! { should_match_path_pattern_inside_of_path_pattern_and_ignore_failures $target = 
                        pattern "{| cons( {| inner(^, ^), inner(:one, a, b) |} , ^, ^), outer(:one, c, d) |}";
                        data "cons( inner(inner(:two, :a, :b), inner(:one, :c, :d)), outer(:one, :e, :f), outer(:two, :g, :h) )";
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
                        data "cons( cons([:a], [:one]), cons([:b], [:two]) )";
                        { "x" => ":a"; "y" => ":b" }
                        { "x" => ":a"; "y" => ":two" }
                        { "x" => ":one"; "y" => ":b" }
                        { "x" => ":one"; "y" => ":two" }
                }

                t! { should_match_path_and_capture_after $target =
                        pattern "cons( {| cons(^, ^), [^], x |}, outer )";
                        data "cons( cons([:a], [:one]), :outer )";
                        { "x" => ":a"; "outer" => ":outer" }
                        { "x" => ":one"; "outer" => ":outer" }
                }

                t! { should_match_path_and_capture_before $target =
                        pattern "cons( outer, {| cons(^, ^), [^], x |} )";
                        data "cons( :outer, cons([:a], [:one]) )";
                        { "x" => ":a"; "outer" => ":outer" }
                        { "x" => ":one"; "outer" => ":outer" }
                }

                t! { should_match_multiple_paths_with_cons_and_list $target =
                        pattern "{| cons(^, ^), [^], x |}";
                        data "cons([:a], [:one])";
                        { "x" => ":a" }
                        { "x" => ":one" }
                }

                t! { should_match_path_with_capture_before $target = 
                        pattern "{| cons(a, ^), [^], x |}";
                        data "cons(:one, [:a])";
                        { "a" => ":one"; "x" => ":a" }
                }

                t! { should_match_path_with_capture_after $target = 
                        pattern "{| cons(^, a), [^], x |}";
                        data "cons([:a], :one)";
                        { "a" => ":one"; "x" => ":a" }
                }

                t! { should_match_path_with_cons_and_list $target = 
                        pattern "{| cons(^, _), [^], x |}";
                        data "cons([:a], :one)";
                        { "x" => ":a" }
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
                        pattern "[:one, x, :a]";
                        data "[:one, :two, :a]";
                        { "x" => ":two" }
                }

                t! { should_match_empty_exact_list $target =
                        pattern "[]";
                        data "[]";
                        { }
                }

                t! { should_fail_match_exact_list_due_to_length $target = 
                        pattern "[:one, x, :a, :x]";
                        data "[:one, :two, :a]";
                }

                t! { should_fail_match_exact_list_due_to_value $target = 
                        pattern "[:one, x, :a, :x]";
                        data "[:one, :two, :a, :y]";
                }

                t! { should_fail_match_exact_list_due_to_nested_list_mismatch $target =
                        pattern "[:one, x, :a, [:x, :x]]";
                        data "[:one, :two, :a, [:x, :y]]";
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

                t!{ should_match_template_in_cons $target =
                        pattern "cons( a, %a )";
                        data "cons(:a, :a)";
                        { "a" => ":a" }
                }

                t!{ should_not_match_template_in_cons $target =
                        pattern "cons( a, %a )";
                        data "cons(:a, :b)";
                }

                t!{ should_match_template_in_exact_list $target =
                        pattern "[ a, %a ]";
                        data "[:a, :a]";
                        { "a" => ":a" }
                }

                t!{ should_not_match_template_in_exact_list $target =
                        pattern "[ a, %a ]";
                        data "[:a, :b]";
                }

                t!{ should_match_template_in_list_path $target =
                        pattern "[| a, %a |]";
                        data "[:a, :a, :b, :b, :c, :d, :d]";
                        { "a" => ":a" }
                        { "a" => ":b" }
                        { "a" => ":d" }
                }

                t!{ should_match_template_in_path $target =
                        pattern "{| [^, ^, ^], cons(a, %a) |}";
                        data "[ cons(:a, :a), cons(:b, :b), cons(:c, :d)]";
                        { "a" => ":a" }
                        { "a" => ":b" }
                }

                t!{ should_match_template_in_and $target =
                        pattern "a.and( %a )";
                        data ":a";
                        { "a" => ":a" }
                }

                t!{ should_not_match_template_in_and $target =
                        pattern "a.and( [%a] )";
                        data ":a";
                }

                t!{ should_match_template_in_or $target =
                        pattern "cons(a, [%a]).or( cons(a, %a) )";
                        data "cons(:a, :a)";
                        { "a" => ":a" }
                }

                t!{ should_not_match_template_in_or $target =
                        pattern "cons(a, %a).or( cons(a, %a) )";
                        data "cons(:a, :b)";
                }

                t!{ should_match_template_with_template $target = 
                        pattern "cons(a, [b, %a, %b ] )";
                        data "cons(:a, [:b, :a, :b])";
                        { "a" => ":a"; "b" => ":b" }
                }

                t!{ should_not_match_template_with_template $target = 
                        pattern "cons(a, [b, %a, %b] )";
                        data "cons(:a, [:b, :a, :c])";
                }
                
                t! { should_match_exact_list_nested_template $target =
                        pattern "[a, b, [[ %a, c ]], %c]";
                        data "[:a, :b, [[:a, :c]], :c]";
                        { "a" => ":a"; "b" => ":b"; "c" => ":c" }
                }

                t! { should_match_cons_nested_template $target =
                        pattern "cons(a, b, cons(cons( %a, c )), %c)";
                        data "cons(:a, :b, cons(cons( :a, :c)), :c)";
                        { "a" => ":a"; "b" => ":b"; "c" => ":c" }
                }

                t! { should_match_list_path_nested_template $target =
                        pattern "[a, b, [[| %a, c |]], %c]";
                        data "[:a, :b, [[:a, :c, :a, :c]], :c]";
                        { "a" => ":a"; "b" => ":b"; "c" => ":c" }
                        { "a" => ":a"; "b" => ":b"; "c" => ":c" }
                }

                t! { should_match_path_nested_template $target =
                        pattern "[a, b, [{| [^, ^, %a], c |}], %c]";
                        data "[:a, :b, [[:x, :y, :a]], :x]";
                        { "a" => ":a"; "b" => ":b"; "c" => ":x" }
                }

                t! { should_match_and_nested_template $target =
                        pattern "[a, b.and( [%a, c] ), %c]";
                        data "[:a, [:a, :c], :c]";
                        { "a" => ":a"; "b" => "[:a, :c]"; "c" => ":c" }
                }

                t! { should_match_or_nested_template $target =
                        pattern "[a, [c].or( [%a, c] ), %c]";
                        data "[:a, [:a, :c], :c]";
                        { "a" => ":a"; "c" => ":c" }
                }

                t! { should_match_list_data_in_template $target =
                        pattern "[a, %a]";
                        data "[[:a], [:a]]";
                        { "a" => "[:a]" }
                }

                t! { should_match_cons_data_in_template $target =
                        pattern "[a, %a]";
                        data "[cons(:a), cons(:a)]";
                        { "a" => "cons(:a)" }
                }

                t! { should_match_string_data_in_template $target =
                        pattern "[a, %a]";
                        data "[\"a\", \"a\"]";
                        { "a" => "\"a\"" }
                }
            }
        };
    }

    all!(pattern_match);
    
    all!(lazy_pattern_match);
}

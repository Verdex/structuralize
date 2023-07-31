
mod parsing;
pub mod data;
pub mod pattern;

/* 
   TODO parser: 
                * parser
                * fn impl
                * macro_rules ?
    
   TODO pattern matching:
                * parser
                * fn impl
                * macro_rules ?
*/

#[cfg(test)]
mod tests {

    macro_rules! t {
        ($name:ident $matcher:ident = pattern $pat:expr; data $dat:expr; $({ $($s:expr => $d:expr);* })* ) => {
            #[test]
            fn $name() {
                let pattern : Pattern = $pat.parse().unwrap();
                let data : Data = $dat.parse().unwrap();

                let mut results = $matcher(&pattern, &data).into_iter().collect::<Vec<_>>();

                $(
                    let _r = results.remove(0);

                    $(
                        let data = _r.get(&$s.into()).unwrap();
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

                // TODO : path pattern that has path patterns inside of it (needs more impl before this will work)

                t! { should_match_nested_nexts_in_path $target =
                        pattern "{| cons(cons(^, ^), ^), [^], x |}";
                        data "cons(cons([:a], [:b]), [:c])";
                        { "x" => ":c" }
                        { "x" => ":a" }
                        { "x" => ":b" }
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
                        { "x" => ":d" }
                        { "x" => ":c" }
                        { "x" => ":b" }
                        { "x" => ":a" }
                }

                t! { should_match_path_and_path $target = 
                        pattern "cons( {| cons(^, ^), [^], x |}, {| cons(^, ^), [^], y |} )";
                        data "cons( cons([:a], [1.1]), cons([:b], [2.2]) )";
                        { "x" => "1.1"; "y" => "2.2" }
                        { "x" => ":a"; "y" => "2.2" }
                        { "x" => "1.1"; "y" => ":b" }
                        { "x" => ":a"; "y" => ":b" }
                }

                t! { should_match_path_and_capture_after $target =
                        pattern "cons( {| cons(^, ^), [^], x |}, outer )";
                        data "cons( cons([:a], [1.1]), :outer )";
                        { "x" => "1.1"; "outer" => ":outer" }
                        { "x" => ":a"; "outer" => ":outer" }
                }

                t! { should_match_path_and_capture_before $target =
                        pattern "cons( outer, {| cons(^, ^), [^], x |} )";
                        data "cons( :outer, cons([:a], [1.1]) )";
                        { "x" => "1.1"; "outer" => ":outer" }
                        { "x" => ":a"; "outer" => ":outer" }
                }

                t! { should_match_multiple_paths_with_cons_and_list $target =
                        pattern "{| cons(^, ^), [^], x |}";
                        data "cons([:a], [1.1])";
                        { "x" => "1.1" }
                        { "x" => ":a" }
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

                t! { should_match_struct_with_inner_var $target = 
                        pattern "struct { a: 1, b: 2, c: x }";
                        data "struct { a: 1, b: 2, c: 3 }";
                        { "x" => "3" }
                }

                t! { should_fail_match_struct_due_to_length $target = 
                        pattern "struct { a: 1, b: 2 }";
                        data "struct { a: 1, b: 2, c: 3 }";
                }

                t! { should_fail_match_struct_due_to_inner_name_mismatch $target = 
                        pattern "struct { a: 1, b: 2, x: 3 }";
                        data "struct { a: 1, b: 2, c: 3 }";
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

                t!{ should_match_struct $target =
                        pattern "struct { a: 1, b: 2, c: 3 }";
                        data "struct { a: 1, b: 2, c: 3 }";
                        {  }
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

}

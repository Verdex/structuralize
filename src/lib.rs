
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

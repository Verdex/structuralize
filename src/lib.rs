
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

    use super::*;
    use crate::data::*;
    use crate::pattern::*;

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

    t!{ should_fail_match_due_to_cons_length_mismatch pattern_match =
            pattern "cons( x, y, z )";
            data "cons(:a, :b, :c, :d)";
    }


    t!{ should_match_struct pattern_match =
            pattern "struct { a: 1, b: 2, c: 3 }";
            data "struct { a: 1, b: 2, c: 3 }";
            {  }
    }

    t!{ should_match_cons_with_vars pattern_match = 
            pattern "cons( x, y, z )";
            data "cons(:a, :b, :c)"; 
            { "x" => ":a"; "y" => ":b"; "z" => ":c" } 
    }

}

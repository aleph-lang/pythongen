use aleph_syntax_tree::syntax::AlephTree as at;

/// Generates an `If`/`LetRec` branch's tail node. Statement-like nodes
/// (control flow, assignment, sequencing, side-effecting ops) already emit
/// a complete, self-contained line and are recursed into as-is; anything
/// else is a bare expression and needs an explicit `return` to actually
/// produce Python with the same semantics as the Aleph source. The
/// previous implementation allow-listed only `Var`/`Int`/`App`/`Ident` as
/// return-worthy, silently dropping `return` for every binary/unary
/// operator (`Mul`, `Add`, `Not`, ...) and every other literal type
/// (`Float`, `Bool`, `String`, ...) — e.g. `factorial(n-1) * n` as an
/// `else` branch compiled to valid Python that always returned `None`.
fn branch_gen(node: at, indent: i64) -> String {
    match &node {
        at::If{..} | at::While{..} | at::Stmts{..} | at::Let{..} | at::LetRec{..}
        | at::Put{..} | at::Remove{..} | at::Match{..} | at::Iprt{..} | at::Clss{..}
        | at::Return{..} | at::Comment{..} | at::CommentMulti{..} | at::Break | at::Continue
        | at::Assert{..} | at::Unit => gen(node, indent + 1),
        _ => format!("{}return {}", aleph_syntax_tree::comp_indent(indent + 1), gen(node, 0)),
    }
}

fn gen(ast: at, indent: i64) -> String {
    let c_indent=aleph_syntax_tree::comp_indent(indent);
    match ast {
        at::Unit => format!("{}", ""),
        at::Ellipsis => format!("{}", "..."),
        at::Int{value} => format!("{}{}", c_indent, value),
        at::Float{value} => format!("{}{}", c_indent, value),
        at::Bool{value} => format!("{}{}", c_indent, if value=="true" { "True" } else { "False" }),
        at::String{value} => format!("{}{}", c_indent, value),
        at::Ident{value} => format!("{}{}", c_indent, value),
        at::Complex{real, imag} => format!("{}complexe({}, {})", c_indent, real, imag),
        at::Bytes{elems} => format!("{}", String::from_utf8(elems).expect("Found invalid UTF-8")),
        at::Tuple{elems} => format!("{}", aleph_syntax_tree::gen_list_expr_sep(elems, gen, ",")),
        at::Array{elems} => format!("[{}]", aleph_syntax_tree::gen_list_expr_sep(elems, gen, ",")),
        at::Neg{expr} => format!("{}-{}", c_indent, gen(*expr, 0)),
        at::Not{bool_expr} => format!("{}not({})", c_indent, gen(*bool_expr, 0)),
        at::And{bool_expr1, bool_expr2} => format!("{}{} and {}", c_indent, gen(*bool_expr1, 0), gen(*bool_expr2, 0)),
        at::Or{bool_expr1, bool_expr2} => format!("{}{} or {}", c_indent, gen(*bool_expr1, 0), gen(*bool_expr2, 0)),
        at::Add{number_expr1, number_expr2} => format!("{}{} + {}", c_indent, gen(*number_expr1, 0), gen(*number_expr2, 0)),
        at::Sub{number_expr1, number_expr2} => format!("{}{} - {}", c_indent, gen(*number_expr1, 0), gen(*number_expr2, 0)),
        at::Mul{number_expr1, number_expr2} => format!("{}{} * {}", c_indent, gen(*number_expr1, 0), gen(*number_expr2, 0)),
        at::Div{number_expr1, number_expr2} => format!("{}{} / {}", c_indent, gen(*number_expr1, 0), gen(*number_expr2, 0)),
        at::Eq{expr1, expr2} => format!("{}{} == {}", c_indent, gen(*expr1, 0), gen(*expr2, 0)),
        at::LE{expr1, expr2} => format!("{}{} <= {}", c_indent, gen(*expr1, 0), gen(*expr2, 0)),
        at::In{expr1, expr2} => format!("{}{} in {}", c_indent, gen(*expr1, 0), gen(*expr2, 0)),
        at::If{condition, then, els} => match *els {
            at::Unit => format!("{}if({}):\n{}", c_indent, gen(*condition, 0), gen(*then, indent+1)),
            _ => {
                let then_return = branch_gen(*then, indent);
                let else_return = branch_gen(*els, indent);
                format!("{}if({}):\n{}\n{}else:\n{}", c_indent, gen(*condition, 0), then_return, c_indent, else_return)
            },
        },
        at::While{init_expr, condition, loop_expr, post_expr} => match *post_expr{
            at::Unit => format!("{init}\n{id}while({cond}):\n{loop_ex}",init=gen(*init_expr,indent),id=c_indent,cond=gen(*condition, 0),loop_ex=gen(*loop_expr, indent+1)),
            _ => format!("{init}\n{id}while({cond}):\n{loop_ex}\n{post}",init=gen(*init_expr,indent),id=c_indent,cond=gen(*condition, 0),loop_ex=gen(*loop_expr, indent+1),post=gen(*post_expr,indent+1)),
        },
        at::Let{var, is_pointer: _, value, expr} => match *expr {
            at::Unit => match *value{
                at::Remove{array_name, elem, is_value} => format!("{}{}", c_indent, gen(at::Remove{array_name, elem, is_value}, 0)),
                _ => format!("{}{} = {}", c_indent, var, gen(*value, 0)),
            },
            _ => match *value{
                at::Remove{array_name, elem, is_value} => format!("{}{}\n{}", c_indent, gen(at::Remove{array_name, elem, is_value}, 0), gen(*expr, indent)),
                _ => format!("{}{} = {}\n{}", c_indent, var, gen(*value, 0), gen(*expr, indent)),
            }
        },
        at::LetRec{name, args, body} => {
            let formatted_body = match body.as_ref() {
                at::If{..} => gen(*body, indent+1), 
                at::Stmts{expr1, expr2} => {
                    let last_expr = match expr2.as_ref() {
                        at::If{..} => gen(*expr2.clone(), indent+1),
                        _ => format!("{}return {}", aleph_syntax_tree::comp_indent(indent+1), gen(*expr2.clone(), 0)),
                    };
                    format!("{}\n{}", gen(*expr1.clone(), indent+1), last_expr)
                },
                _ => format!("{}return {}", aleph_syntax_tree::comp_indent(indent+1), gen(*body, 0)),
            };
            format!("{}def {}({}):\n{}\n", c_indent, name, aleph_syntax_tree::gen_list_expr_sep(args, gen, ", "), formatted_body)
        },
        at::Get{array_name, elem} => format!("{}{}[{}]", c_indent, array_name, gen(*elem, 0)),
        at::Put{array_name, elem, value, insert} => if insert.eq("true") {
            format!("{}{}.insert({},{})", c_indent, array_name, gen(*elem, 0), gen(*value, 0))
        } else {
            format!("{}{}[{}] = {}", c_indent, array_name, gen(*elem, 0), gen(*value, 0))
        },
        at::Remove{array_name, elem, is_value} => if is_value.eq("true") {
            format!("{}{}.remove({})", c_indent, array_name, gen(*elem, 0))
        } else {
            format!("{}{}.pop({})",c_indent,array_name, gen(*elem, 0))
        },
        at::Length{var} => format!("{}len({})", c_indent, var),
        at::Match{expr, case_list} => format!("{}match {} with\n{}", c_indent, gen(*expr, 0), aleph_syntax_tree::gen_list_expr(case_list, gen)),
        at::MatchLine{condition, case_expr} => format!("{}: {} -> {}\n", c_indent, gen(*condition, 0), gen(*case_expr, 0)),
        at::Var{var, is_pointer: _} => format!("{}{}",c_indent, var),
        at::App{object_name, fun, param_list} => format!("{}{}{}({})",c_indent, (if object_name.ne("") {format!("{}.", object_name)} else {String::from("")}), gen(*fun, 0), aleph_syntax_tree::gen_list_expr_sep(param_list, gen, ", ")),
        at::Stmts{expr1, expr2} => format!("{}\n{}", gen(*expr1, indent), gen(*expr2, indent)),
        at::Iprt{name, ..} => format!("{}import {}", c_indent, name),
        at::Clss{name, attribute_list, body, ..} => format!("{}class {} {{\n{}{}\n{}\n}}", c_indent, name, aleph_syntax_tree::comp_indent(indent+1), attribute_list.join(&format!("\n{}", aleph_syntax_tree::comp_indent(indent+1))), gen(*body, indent+1)),
        at::Return{value} => format!("return {}", gen(*value, 0)),
        at::Comment{value} => format!("{}{}", c_indent, value),
        at::CommentMulti{value} => format!("{}{}", c_indent, value),
        at::Break => format!("{}break", c_indent),
        at::Continue => format!("{}continue", c_indent),
        at::Assert {condition, message} => format!("{}assert({}, {})", c_indent, gen(*condition, indent), gen(*message, indent)),
        // Cognitive layer — no Python equivalent yet, emit a comment
        at::Intend{name, ..}   => format!("{}# intention {}", c_indent, name),
        at::Suggest{var, ..}   => format!("{}# suggest {}", c_indent, var),
        at::Act{intention, ..} => format!("{}# act {}", c_indent, gen(*intention, 0)),
        at::Remember{key, ..}  => format!("{}# remember {}", c_indent, gen(*key, 0)),
        at::Perceive{var, ..}  => format!("{}# perceive {:?}", c_indent, var),
        _ => String::new()
    }
}

pub fn generate(ast: at) -> String {
    gen(ast, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aleph_syntax_tree::syntax::AlephTree as at;

    fn wrapped_in_if_else(else_branch: at) -> at {
        at::LetRec {
            name: "f".to_string(),
            args: vec![Box::new(at::Ident { value: "n".to_string() })],
            body: Box::new(at::If {
                condition: Box::new(at::LE {
                    expr1: Box::new(at::Ident { value: "n".to_string() }),
                    expr2: Box::new(at::Int { value: "1".to_string() }),
                }),
                then: Box::new(at::Int { value: "1".to_string() }),
                els: Box::new(else_branch),
            }),
        }
    }

    #[test]
    fn if_else_branch_that_is_a_binary_op_gets_a_return() {
        // Regression test for the bug this fix closes: `factorial(n-1) * n`
        // as an else branch used to compile to Python that silently
        // dropped the `return`, so the function always returned `None`.
        let node = wrapped_in_if_else(at::Mul {
            number_expr1: Box::new(at::App {
                object_name: "".to_string(),
                fun: Box::new(at::Ident { value: "f".to_string() }),
                param_list: vec![Box::new(at::Sub {
                    number_expr1: Box::new(at::Ident { value: "n".to_string() }),
                    number_expr2: Box::new(at::Int { value: "1".to_string() }),
                })],
            }),
            number_expr2: Box::new(at::Ident { value: "n".to_string() }),
        });
        let out = generate(node);
        assert!(out.contains("return f(n - 1) * n"), "{}", out);
    }

    #[test]
    fn if_else_branch_that_is_an_add_gets_a_return() {
        let node = wrapped_in_if_else(at::Add {
            number_expr1: Box::new(at::Ident { value: "n".to_string() }),
            number_expr2: Box::new(at::Ident { value: "n".to_string() }),
        });
        let out = generate(node);
        assert!(out.contains("return n + n"), "{}", out);
    }

    #[test]
    fn if_else_branch_that_is_a_bare_app_still_gets_a_return() {
        // Confirms the fix didn't regress the one case the old allow-list
        // already handled correctly.
        let node = wrapped_in_if_else(at::App {
            object_name: "".to_string(),
            fun: Box::new(at::Ident { value: "f".to_string() }),
            param_list: vec![Box::new(at::Ident { value: "n".to_string() })],
        });
        let out = generate(node);
        assert!(out.contains("return f(n)"), "{}", out);
    }

    #[test]
    fn if_else_branch_that_is_a_put_is_not_wrapped_in_a_return() {
        // A statement-like branch (assignment into an array) is already a
        // complete line and must not be prefixed with `return`.
        let node = wrapped_in_if_else(at::Put {
            array_name: "res".to_string(),
            elem: Box::new(at::Int { value: "0".to_string() }),
            value: Box::new(at::Ident { value: "n".to_string() }),
            insert: "false".to_string(),
        });
        let out = generate(node);
        assert!(!out.contains("return res"), "{}", out);
        assert!(out.contains("res[0] = n"), "{}", out);
    }
}

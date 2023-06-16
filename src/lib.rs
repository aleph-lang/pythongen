use aleph_syntax_tree::syntax::AlephTree as at;

fn gen(ast: at, indent: i64) -> String {
    let c_indent=aleph_syntax_tree::comp_indent(indent);
    match ast {
        at::Unit => format!("{}", ""),
        at::Ellipsis => format!("{}", "..."),
        at::Int{value} => format!("{}{}", c_indent, value),
        at::Float{value} => format!("{}{}", c_indent, value),
        at::Bool{value} => format!("{}{}", c_indent, if value=="true" { "True" } else { "False" }),
        at::String{value} => format!("{}{}", c_indent, value),
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
        at::If{condition, then,els} => match *els {
            at::Unit => format!("{}if({}):\n{}", c_indent, gen(*condition, 0), gen(*then, indent+1)),
            _ => format!("{}if({}):\n{}\nelse:\n{}", c_indent, gen(*condition, 0), gen(*then, indent+1), gen(*els, indent+1)),
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
        at::LetRec{name, args, body} => format!("{}def {}({}):\n{}\n", c_indent, name, aleph_syntax_tree::gen_list_expr(args, gen), gen(*body, indent+1)),
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
            at::App{object_name, fun, param_list} => format!("{}{}{}({})",c_indent, (if object_name.ne("") {format!("{}.", object_name)} else {String::from("")}), gen(*fun, 0), aleph_syntax_tree::gen_list_expr(param_list, gen)),
            at::Stmts{expr1, expr2} => format!("{}\n{}", gen(*expr1, indent), gen(*expr2, indent)),
            at::Iprt{name} => format!("{}import {}", c_indent, name),
            at::Clss{name, attribute_list, body} => format!("{}class {} {{\n{}{}\n{}\n}}", c_indent, name, aleph_syntax_tree::comp_indent(indent+1), attribute_list.join(&format!("\n{}", aleph_syntax_tree::comp_indent(indent+1))), gen(*body, indent+1)),
            at::Return{value} => format!("return {}", gen(*value, 0)),
            at::Comment{value} => format!("{}{}", c_indent, value),
            at::CommentMulti{value} => format!("{}{}", c_indent, value),
            at::Break => format!("{}break", c_indent),
            at::Continue => format!("{}continue", c_indent),
            at::Assert {condition, message} => format!("{}assert({}, {})", c_indent, gen(*condition, indent), gen(*message, indent)),
    }
}

pub fn generate(ast: at) -> String {
    gen(ast, 0)
}

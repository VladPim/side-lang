use crate::ast::*;

pub fn generate(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n\n");

    // Вспомогательная функция ввода
    out.push_str("int side_input(const char* prompt) {\n");
    out.push_str("    int val;\n");
    out.push_str("    printf(\"%s\", prompt);\n");
    out.push_str("    scanf(\"%d\", &val);\n");
    out.push_str("    return val;\n");
    out.push_str("}\n\n");

    for func in &program.functions {
        let c_name = if func.name == "main" {
            "side_main".to_string()
        } else {
            format!("side_{}", func.name)
        };

        let params_str = func.params.iter()
            .map(|p| {
                match p.param_type {
                    Type::Int => format!("int {}", p.name),
                    Type::Str => format!("const char* {}", p.name),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("int {}(", c_name));
        out.push_str(&params_str);
        out.push_str(") {\n");

        generate_stmts(&func.body, &mut out, 1);
        out.push_str("    return 0;\n}\n\n");
    }

    out.push_str("int main() {\n");
    out.push_str("    return side_main();\n");
    out.push_str("}\n");
    out
}

fn generate_stmts(stmts: &[Stmt], out: &mut String, indent: usize) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            Stmt::Let { name, value } => {
                match value {
                    Expr::ArrayLiteral(elements) => {
                        // объявление массива с инициализацией
                        out.push_str(&format!("{}int {}[] = {{", pad, name));
                        for (i, elem) in elements.iter().enumerate() {
                            if i > 0 { out.push_str(", "); }
                            generate_expr(elem, out);
                        }
                        out.push_str("};\n");
                    }
                    _ => {
                        out.push_str(&format!("{}int {} = ", pad, name));
                        generate_expr(value, out);
                        out.push_str(";\n");
                    }
                }
            }
            Stmt::Assign { name, value } => {
                out.push_str(&format!("{}{} = ", pad, name));
                generate_expr(value, out);
                out.push_str(";\n");
            }
            Stmt::AssignIndex { name, index, value } => {
                out.push_str(&format!("{}{}[", pad, name));
                generate_expr(index, out);
                out.push_str("] = ");
                generate_expr(value, out);
                out.push_str(";\n");
            }
            Stmt::IoPrint(args) => {
                for arg in args {
                    match arg {
                        Expr::StringLiteral(_) => {
                            out.push_str(&format!("{}printf(\"%s\", ", pad));
                            generate_expr(arg, out);
                            out.push_str(");\n");
                        }
                        _ => {
                            out.push_str(&format!("{}printf(\"%d\", ", pad));
                            generate_expr(arg, out);
                            out.push_str(");\n");
                        }
                    }
                }
                out.push_str(&format!("{}printf(\"\\n\");\n", pad));
            }
            Stmt::If { condition, then_body, else_body } => {
                out.push_str(&format!("{}if (", pad));
                generate_expr(condition, out);
                out.push_str(") {\n");
                generate_stmts(then_body, out, indent + 1);
                out.push_str(&format!("{}}}\n", pad));
                if let Some(else_stmts) = else_body {
                    out.push_str(&format!("{}else {{\n", pad));
                    generate_stmts(else_stmts, out, indent + 1);
                    out.push_str(&format!("{}}}\n", pad));
                }
            }
            Stmt::While { condition, body } => {
                out.push_str(&format!("{}while (", pad));
                generate_expr(condition, out);
                out.push_str(") {\n");
                generate_stmts(body, out, indent + 1);
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::Return(expr) => {
                out.push_str(&format!("{}return ", pad));
                generate_expr(expr, out);
                out.push_str(";\n");
            }
            Stmt::Break => {
                out.push_str(&format!("{}break;\n", pad));
            }
            Stmt::Continue => {
                out.push_str(&format!("{}continue;\n", pad));
            }
            Stmt::CallStmt { name, args } => {
                let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                out.push_str(&format!("{}{}(", pad, c_name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    generate_expr(arg, out);
                }
                out.push_str(");\n");
            }
        }
    }
}

fn generate_expr(expr: &Expr, out: &mut String) {
    match expr {
        Expr::Number(n) => out.push_str(&n.to_string()),
        Expr::StringLiteral(s) => out.push_str(&format!("\"{}\"", s)),
        Expr::Variable(name) => out.push_str(name),
        Expr::Input(prompt) => out.push_str(&format!("side_input(\"{}\")", prompt)),
        Expr::Call { name, args } => {
            let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
            out.push_str(&format!("{}(", c_name));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(arg, out);
            }
            out.push(')');
        }
        Expr::ArrayLiteral(_) => {
            // В выражениях не ожидается, но на всякий случай – фигурные скобки
            out.push_str("{/*array literal not supported in expression*/}");
        }
        Expr::Index { array, index } => {
            generate_expr(array, out);
            out.push('[');
            generate_expr(index, out);
            out.push(']');
        }
        Expr::Binary { left, op, right } => {
            out.push('(');
            generate_expr(left, out);
            let op_str = match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Eq => " == ",
                BinOp::NotEq => " != ",
                BinOp::Less => " < ",
                BinOp::Greater => " > ",
                BinOp::LessEq => " <= ",
                BinOp::GreaterEq => " >= ",
                BinOp::And => " && ",
                BinOp::Or => " || ",
            };
            out.push_str(op_str);
            generate_expr(right, out);
            out.push(')');
        }
        Expr::Unary { op, expr } => {
            match op {
                UnaryOp::Not => {
                    out.push_str("!(");
                    generate_expr(expr, out);
                    out.push(')');
                }
                UnaryOp::Neg => {
                    out.push_str("-(");
                    generate_expr(expr, out);
                    out.push(')');
                }
            }
        }
    }
}

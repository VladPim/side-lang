use crate::ast::*;

/// Генерирует C-код из AST
pub fn generate(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n\n");

    for func in &program.functions {
        let c_name = if func.name == "main" {
            // чтобы настоящая main вызывала нашу, переименуем в side_main
            "side_main".to_string()
        } else {
            format!("side_{}", func.name)
        };
        out.push_str(&format!("int {}() {{\n", c_name));
        generate_stmts(&func.body, &mut out, 1);
        out.push_str("    return 0;\n}\n\n");
    }

    // Добавляем настоящую точку входа
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
                out.push_str(&format!("{}int {} = ", pad, name));
                generate_expr(value, out);
                out.push_str(";\n");
            }
            Stmt::IoPrint(expr) => {
                match expr {
                    // Если выражение – строковый литерал, выводим как "%s\n"
                    Expr::StringLiteral(_) => {
                        out.push_str(&format!("{}printf(\"%s\\n\", ", pad));
                        generate_expr(expr, out);
                        out.push_str(");\n");
                    }
                    // Для всего остального (числа, переменные, бинарные) используем "%d\n"
                    _ => {
                        out.push_str(&format!("{}printf(\"%d\\n\", ", pad));
                        generate_expr(expr, out);
                        out.push_str(");\n");
                    }
                }
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
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
        }
    }
}

fn generate_expr(expr: &Expr, out: &mut String) {
    match expr {
        Expr::Number(n) => out.push_str(&n.to_string()),
        Expr::StringLiteral(s) => out.push_str(&format!("\"{}\"", s)),
        Expr::Variable(name) => out.push_str(name),
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
            };
            out.push_str(op_str);
            generate_expr(right, out);
            out.push(')');
        }
    }
}
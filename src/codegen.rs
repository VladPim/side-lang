use crate::ast::*;

pub fn generate(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n");
    out.push_str("#include <stdlib.h>\n");
    out.push_str("#include <string.h>\n\n");

    // Вспомогательные функции для динамических массивов
    out.push_str("// Dynamic array functions\n");
    out.push_str("void side_arr_push(int** arr, int* size, int* cap, int value) {\n");
    out.push_str("    if (*size >= *cap) {\n");
    out.push_str("        *cap = (*cap == 0) ? 2 : (*cap) * 2;\n");
    out.push_str("        *arr = realloc(*arr, (*cap) * sizeof(int));\n");
    out.push_str("    }\n");
    out.push_str("    (*arr)[*size] = value;\n");
    out.push_str("    (*size)++;\n");
    out.push_str("}\n");
    out.push_str("int side_arr_pop(int** arr, int* size, int* cap) {\n");
    out.push_str("    if (*size <= 0) return -1;\n");
    out.push_str("    int val = (*arr)[*size - 1];\n");
    out.push_str("    (*size)--;\n");
    out.push_str("    return val;\n");
    out.push_str("}\n");
    out.push_str("int* side_arr_create(int n, int* vals) {\n");
    out.push_str("    int* arr = malloc(n * sizeof(int));\n");
    out.push_str("    for (int i = 0; i < n; i++) arr[i] = vals[i];\n");
    out.push_str("    return arr;\n");
    out.push_str("}\n\n");

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
            .map(|p| match p.param_type {
                Type::Int => format!("int {}", p.name),
                Type::Str => format!("const char* {}", p.name),
                Type::Array => format!("int* {}", p.name), // + size/capacity вне сигнатуры?
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
            Stmt::Let { name, var_type, value } => {
                match var_type {
                    Type::Int => {
                        out.push_str(&format!("{}int {} = ", pad, name));
                        generate_expr(value, out);
                        out.push_str(";\n");
                    }
                    Type::Str => {
                        out.push_str(&format!("{}const char* {} = ", pad, name));
                        generate_expr(value, out);
                        out.push_str(";\n");
                    }
                    Type::Array => {
                        // Объявляем три переменные: data, size, cap
                        out.push_str(&format!("{}int* {} = NULL;\n", pad, name));
                        out.push_str(&format!("{}int {}_size = 0;\n", pad, name));
                        out.push_str(&format!("{}int {}_cap = 0;\n", pad, name));
                        if let Expr::ArrayLiteral(elements) = value {
                            if !elements.is_empty() {
                                // Инициализация через side_arr_create
                                out.push_str("    { int _arr_vals[] = {");
                                for (i, e) in elements.iter().enumerate() {
                                    if i > 0 { out.push_str(", "); }
                                    generate_expr(e, out);
                                }
                                out.push_str(&format!("}};\n"));
                                out.push_str(&format!("{}int _n = {};\n", pad, elements.len()));
                                out.push_str(&format!("{}{} = side_arr_create(_n, _arr_vals);\n", pad, name));
                                out.push_str(&format!("{}{}_size = _n;\n", pad, name));
                                out.push_str(&format!("{}    {}_cap = _n;\n", pad, name));
out.push_str(&format!("{}}}\n", pad));
                            }
                        }
                    }
                }
            }
            Stmt::Assign { name, value } => {
                // Только для простых int/str (массивы не присваиваем целиком)
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
            Stmt::Break => out.push_str(&format!("{}break;\n", pad)),
            Stmt::Continue => out.push_str(&format!("{}continue;\n", pad)),
            Stmt::CallStmt { name, args } => {
                // Обрабатываем встроенные вызовы как push/pop
                match name.as_str() {
                    "push" => {
                        if args.len() != 2 {
                            out.push_str("/* push requires 2 args */\n");
                        } else {
                            let arr_name = match &args[0] {
                                Expr::Variable(n) => n.clone(),
                                _ => { out.push_str("/* first arg must be variable */\n"); continue; }
                            };
                            out.push_str(&format!("{}side_arr_push(&{}, &{}_size, &{}_cap, ", pad, arr_name, arr_name, arr_name));
                            generate_expr(&args[1], out);
                            out.push_str(");\n");
                        }
                    }
                    "pop" => {
                        if args.len() != 1 {
                            out.push_str("/* pop requires 1 arg */\n");
                        } else {
                            let arr_name = match &args[0] {
                                Expr::Variable(n) => n.clone(),
                                _ => { out.push_str("/* first arg must be variable */\n"); continue; }
                            };
                            out.push_str(&format!("{}side_arr_pop(&{}, &{}_size, &{}_cap);\n", pad, arr_name, arr_name, arr_name));
                        }
                    }
                    _ => {
                        // Обычный вызов функции
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
    }
}

fn generate_expr(expr: &Expr, out: &mut String) {
    match expr {
        Expr::Number(n) => out.push_str(&n.to_string()),
        Expr::StringLiteral(s) => out.push_str(&format!("\"{}\"", s)),
        Expr::Variable(name) => out.push_str(name),
        Expr::Input(prompt) => out.push_str(&format!("side_input(\"{}\")", prompt)),
        Expr::Call { name, args } => {
            match name.as_str() {
                "len" => {
                    if args.len() != 1 {
                        out.push_str("/* len requires 1 arg */");
                        return;
                    }
                    // Пытаемся определить, массив это или строка
                    // Если аргумент - Variable, и мы знаем, что это массив? Но здесь мы не имеем контекста.
                    // Упростим: генерируем всегда (name)_size? Но для строк нужен strlen.
                    // Будем считать, что если аргумент - переменная, которая объявлена как массив, то _size, иначе strlen.
                    // Без контекста типа это сложно. Пока сделаем так: если аргумент - Variable с суффиксом? Нет.
                    // Лучше: встроенная функция len генерирует вызов side_len, которой мы передадим тип?
                    // Для простоты сделаем: если аргумент - простое имя (Variable), то пишем `(arr_name##_size)`, иначе `strlen(expr)`.
                    if let Expr::Variable(arr_name) = &args[0] {
                        out.push_str(&format!("{}_size", arr_name));
                    } else {
                        out.push_str("strlen(");
                        generate_expr(&args[0], out);
                        out.push_str(")");
                    }
                }
                _ => {
                    let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                    out.push_str(&format!("{}(", c_name));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        generate_expr(arg, out);
                    }
                    out.push(')');
                }
            }
        }
        Expr::ArrayLiteral(elements) => {
            // Генерация литерала массива невозможна в выражении, но на всякий случай
            out.push_str("{");
            for (i, e) in elements.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(e, out);
            }
            out.push_str("}");
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

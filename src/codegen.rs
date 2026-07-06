use crate::ast::*;
use std::collections::HashMap;

// --- Вспомогательная структура для позиций ---
struct SourceInfo {
    source: String,
}

impl SourceInfo {
    fn new(source: String) -> Self {
        Self { source }
    }

    fn format_error(&self, span: &Span, msg: &str) -> String {
        let (line, col) = self.location(span.start);
        format!("Error at {}:{}: {}", line, col, msg)
    }

    fn location(&self, pos: usize) -> (usize, usize) {
        let source = &self.source;
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in source.chars().enumerate() {
            if i == pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

#[derive(Clone)]
struct Scope {
    vars: HashMap<String, Type>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    fn new() -> Self { Scope { vars: HashMap::new(), parent: None } }
    fn push(&self) -> Self { Scope { vars: HashMap::new(), parent: Some(Box::new(self.clone())) } }
    fn declare(&mut self, name: &str, tp: Type) { self.vars.insert(name.to_string(), tp); }
    fn get(&self, name: &str) -> Option<Type> {
        self.vars.get(name).cloned().or_else(|| self.parent.as_ref()?.get(name))
    }
}

pub fn generate(program: &Program, source: String) -> Result<String, String> {
    let src_info = SourceInfo::new(source);
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <math.h>\n#include <time.h>\n\n");

    // Константы
    for c in &program.constants {
        let c_type = match &c.value {
            Expr::DoubleLiteral(_, _) => "const double",
            Expr::StringLiteral(_, _) => "const char*",
            _ => "const int",
        };
        out.push_str(&format!("{} {} = ", c_type, c.name));
        generate_expr(&c.value, &mut out, &Scope::new(), &program.functions, &src_info)?;
        out.push_str(";\n");
    }

    // Структуры
    for s in &program.structs {
        out.push_str(&format!("typedef struct {{\n"));
        for f in &s.fields {
            let type_str = type_to_c(&f.field_type);
            out.push_str(&format!("    {} {};\n", type_str, f.name));
        }
        out.push_str(&format!("}} side_{};\n\n", s.name));
    }

    // Встроенные функции
    out.push_str(r#"
void side_arr_push(int** arr, int* size, int* cap, int value) {
    if (*size >= *cap) { *cap = (*cap == 0) ? 2 : (*cap) * 2; *arr = realloc(*arr, (*cap) * sizeof(int)); }
    (*arr)[*size] = value; (*size)++;
}
int side_arr_pop(int** arr, int* size, int* cap) {
    if (*size <= 0) return -1; int val = (*arr)[*size - 1]; (*size)--; return val;
}
int* side_arr_create(int n, int* vals) {
    int* arr = malloc(n * sizeof(int)); for (int i = 0; i < n; i++) arr[i] = vals[i]; return arr;
}
void side_arr_push_double(double** arr, int* size, int* cap, double value) {
    if (*size >= *cap) { *cap = (*cap == 0) ? 2 : (*cap) * 2; *arr = realloc(*arr, (*cap) * sizeof(double)); }
    (*arr)[*size] = value; (*size)++;
}
double side_arr_pop_double(double** arr, int* size, int* cap) {
    if (*size <= 0) return -1.0; double val = (*arr)[*size - 1]; (*size)--; return val;
}
double* side_arr_create_double(int n, double* vals) {
    double* arr = malloc(n * sizeof(double)); for (int i = 0; i < n; i++) arr[i] = vals[i]; return arr;
}
int side_input(const char* prompt) { int val; printf("%s", prompt); scanf("%d", &val); return val; }
int side_time() { return (int)(clock() * 1000 / CLOCKS_PER_SEC); }
char* side_str(int n) { static char buf[20]; sprintf(buf, "%d", n); return buf; }
char* side_str_double(double n) { static char buf[30]; sprintf(buf, "%f", n); return buf; }
char* side_str_concat(const char* a, const char* b) {
    char* result = malloc(strlen(a) + strlen(b) + 1);
    strcpy(result, a);
    strcat(result, b);
    return result;
}
"#);

    // Генерируем функции (включая методы)
    let mut non_main = Vec::new();
    let mut mains = Vec::new();
    for func in &program.functions {
        if func.name == "main" {
            mains.push(func);
        } else {
            non_main.push(func);
        }
    }

    for func in non_main.iter().chain(mains.iter()) {
        let c_name = if let Some(ref struct_name) = func.struct_name {
            format!("side_{}_{}", struct_name, func.name)
        } else if func.name == "main" {
            "side_main".to_string()
        } else {
            format!("side_{}", func.name)
        };

        let return_c = type_to_c(&func.return_type);
        
        let mut params = Vec::new();
        if let Some(ref struct_name) = func.struct_name {
            let self_type = Type::Struct(struct_name.clone());
            params.push(format!("{} self", type_to_c(&self_type)));
        }
        for p in &func.params {
            params.push(format!("{} {}", type_to_c(&p.param_type), p.name));
        }
        let params_str = params.join(", ");

        out.push_str(&format!("{} {}({})", return_c, c_name, params_str));
        out.push_str(" {\n");

        let mut scope = Scope::new();
        if let Some(ref struct_name) = func.struct_name {
            let self_type = Type::Struct(struct_name.clone());
            scope.declare("self", self_type);
        }
        for p in &func.params {
            scope.declare(&p.name, p.param_type.clone());
        }
        generate_stmts(&func.body, &mut out, 1, &mut scope, &program.functions, &func.return_type, &src_info)?;

        if func.return_type == Type::Int || func.return_type == Type::Double {
            out.push_str("    return 0;\n");
        }
        out.push_str("}\n\n");
    }

    out.push_str("int main() { return side_main(); }\n");
    Ok(out)
}

fn generate_stmts(
    stmts: &[Stmt],
    out: &mut String,
    indent: usize,
    scope: &mut Scope,
    functions: &[Function],
    expected_return_type: &Type,
    src_info: &SourceInfo,
) -> Result<(), String> {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            Stmt::Let { name, var_type, value, span } => {
                let declared_type = var_type.clone();
                let actual_type = infer_type(value, scope, functions, declared_type.as_ref(), src_info, span)?;
                let final_type = match declared_type {
                    Some(ref dt) => {
                        if !type_compatible(dt, &actual_type) {
                            return Err(src_info.format_error(span, &format!(
                                "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                                actual_type, name, dt
                            )));
                        }
                        dt.clone()
                    }
                    None => actual_type.clone(),
                };

                let c_type = type_to_c(&final_type);
                if final_type == Type::Array || final_type == Type::DoubleArray {
                    let elem_c_type = if final_type == Type::DoubleArray { "double" } else { "int" };
                    out.push_str(&format!("{}{}* {} = NULL;\n", pad, elem_c_type, name));
                    out.push_str(&format!("{}int {}_size = 0;\n", pad, name));
                    out.push_str(&format!("{}int {}_cap = 0;\n", pad, name));
                    if let Expr::ArrayLiteral(elems, _) = value {
                        if !elems.is_empty() {
                            out.push_str(&format!("{}{{\n", pad));
                            out.push_str(&format!("{}    {} _arr_vals[] = {{", pad, elem_c_type));
                            for (i, e) in elems.iter().enumerate() {
                                if i > 0 { out.push_str(", "); }
                                generate_expr(e, out, scope, functions, src_info)?;
                            }
                            out.push_str(&format!("}};\n"));
                            let create_func = if final_type == Type::DoubleArray {
                                "side_arr_create_double"
                            } else {
                                "side_arr_create"
                            };
                            out.push_str(&format!("{}    {} = {}({}, _arr_vals);\n", pad, name, create_func, elems.len()));
                            out.push_str(&format!("{}    {}_size = {};\n", pad, name, elems.len()));
                            out.push_str(&format!("{}    {}_cap = {};\n", pad, name, elems.len()));
                            out.push_str(&format!("{}}}\n", pad));
                        }
                    }
                } else {
                    out.push_str(&format!("{}{} {} = ", pad, c_type, name));
                    generate_expr(value, out, scope, functions, src_info)?;
                    out.push_str(";\n");
                }
                scope.declare(name, final_type);
            }
            Stmt::Assign { name, value, span } => {
                let var_type = scope.get(name)
                    .ok_or(src_info.format_error(span, &format!("Variable '{}' not declared in this scope", name)))?;
                let actual_type = infer_type(value, scope, functions, Some(&var_type), src_info, span)?;
                if !type_compatible(&var_type, &actual_type) {
                    return Err(src_info.format_error(span, &format!(
                        "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                        actual_type, name, var_type
                    )));
                }
                out.push_str(&format!("{}{} = ", pad, name));
                generate_expr(value, out, scope, functions, src_info)?;
                out.push_str(";\n");
            }
            Stmt::AssignIndex { name, index, value, span } => {
                let var_type = scope.get(name)
                    .ok_or(src_info.format_error(span, &format!("Variable '{}' not declared", name)))?;
                let expected_elem = match var_type {
                    Type::Array => Type::Int,
                    Type::DoubleArray => Type::Double,
                    _ => return Err(src_info.format_error(span, "Indexing non-array")),
                };
                let actual_type = infer_type(value, scope, functions, Some(&expected_elem), src_info, span)?;
                if !type_compatible(&expected_elem, &actual_type) {
                    return Err(src_info.format_error(span, &format!(
                        "Type mismatch: cannot assign {:?} to element of {:?}",
                        actual_type, var_type
                    )));
                }
                out.push_str(&format!("{}{}[", pad, name));
                generate_expr(index, out, scope, functions, src_info)?;
                out.push_str("] = ");
                generate_expr(value, out, scope, functions, src_info)?;
                out.push_str(";\n");
            }
            Stmt::IoPrint(args, _span) => {
                for arg in args {
                    let tp = infer_type(arg, scope, functions, None, src_info, &arg.span())?;
                    let format = match tp {
                        Type::Double => "%f",
                        Type::Str => "%s",
                        _ => "%d",
                    };
                    out.push_str(&format!("{}printf(\"{}\", ", pad, format));
                    generate_expr(arg, out, scope, functions, src_info)?;
                    out.push_str(");\n");
                }
                out.push_str(&format!("{}printf(\"\\n\");\n", pad));
            }
            Stmt::If { condition, then_body, else_body, span: _ } => {
                out.push_str(&format!("{}if (", pad));
                generate_expr(condition, out, scope, functions, src_info)?;
                out.push_str(") {\n");
                let mut block_scope = scope.push();
                generate_stmts(then_body, out, indent + 1, &mut block_scope, functions, expected_return_type, src_info)?;
                out.push_str(&format!("{}}}\n", pad));
                if let Some(else_stmts) = else_body {
                    out.push_str(&format!("{}else {{\n", pad));
                    let mut else_scope = scope.push();
                    generate_stmts(else_stmts, out, indent + 1, &mut else_scope, functions, expected_return_type, src_info)?;
                    out.push_str(&format!("{}}}\n", pad));
                }
            }
            Stmt::While { condition, body, span: _ } => {
                out.push_str(&format!("{}while (", pad));
                generate_expr(condition, out, scope, functions, src_info)?;
                out.push_str(") {\n");
                let mut while_scope = scope.push();
                generate_stmts(body, out, indent + 1, &mut while_scope, functions, expected_return_type, src_info)?;
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::For { var_name, start, condition, step, body, span: _ } => {
                let start_type = infer_type(start, scope, functions, None, src_info, &start.span())?;
                let c_type = type_to_c(&start_type);
                out.push_str(&format!("{}{{\n", pad));
                out.push_str(&format!("{}    {} {} = ", pad, c_type, var_name));
                generate_expr(start, out, scope, functions, src_info)?;
                out.push_str(";\n");
                let mut for_scope = scope.push();
                for_scope.declare(var_name, start_type.clone());
                out.push_str(&format!("{}    while (", pad));
                generate_expr(condition, out, &for_scope, functions, src_info)?;
                out.push_str(") {\n");
                let mut body_scope = for_scope.push();
                generate_stmts(body, out, indent + 2, &mut body_scope, functions, expected_return_type, src_info)?;
                out.push_str(&format!("{}        {} = ", pad, var_name));
                generate_expr(step, out, &for_scope, functions, src_info)?;
                out.push_str(";\n");
                out.push_str(&format!("{}    }}\n", pad));
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::Return(expr, span) => {
                let actual_type = infer_type(expr, scope, functions, Some(expected_return_type), src_info, span)?;
                if !type_compatible(expected_return_type, &actual_type) {
                    return Err(src_info.format_error(span, &format!(
                        "Return type mismatch: expected {:?}, got {:?}",
                        expected_return_type, actual_type
                    )));
                }
                out.push_str(&format!("{}return ", pad));
                generate_expr(expr, out, scope, functions, src_info)?;
                out.push_str(";\n");
            }
            Stmt::Break(_span) => out.push_str(&format!("{}break;\n", pad)),
            Stmt::Continue(_span) => out.push_str(&format!("{}continue;\n", pad)),
            Stmt::CallStmt { name, args, span } => {
                check_function_call(name, args, functions, scope, src_info, span)?;
                match name.as_str() {
                    "push" => {
                        let arr_name = extract_var_name(&args[0], src_info, span)?;
                        let arr_type = scope.get(arr_name)
                            .ok_or(src_info.format_error(span, &format!("Variable '{}' not found", arr_name)))?;
                        if arr_type == Type::Array {
                            out.push_str(&format!("{}side_arr_push(&{}, &{}_size, &{}_cap, ", pad, arr_name, arr_name, arr_name));
                        } else if arr_type == Type::DoubleArray {
                            out.push_str(&format!("{}side_arr_push_double(&{}, &{}_size, &{}_cap, ", pad, arr_name, arr_name, arr_name));
                        } else {
                            return Err(src_info.format_error(span, "First argument of push must be an array"));
                        }
                        generate_expr(&args[1], out, scope, functions, src_info)?;
                        out.push_str(");\n");
                    }
                    "pop" => {
                        let arr_name = extract_var_name(&args[0], src_info, span)?;
                        let arr_type = scope.get(arr_name)
                            .ok_or(src_info.format_error(span, &format!("Variable '{}' not found", arr_name)))?;
                        if arr_type == Type::Array {
                            out.push_str(&format!("{}side_arr_pop(&{}, &{}_size, &{}_cap);\n", pad, arr_name, arr_name, arr_name));
                        } else if arr_type == Type::DoubleArray {
                            out.push_str(&format!("{}side_arr_pop_double(&{}, &{}_size, &{}_cap);\n", pad, arr_name, arr_name, arr_name));
                        } else {
                            return Err(src_info.format_error(span, "First argument of pop must be an array"));
                        }
                    }
                    _ => {
                        // Проверяем, не метод ли это (функция с struct_name)
                        let func = functions.iter().find(|f| f.name == *name);
                        if let Some(f) = func {
                            if let Some(ref _struct_name) = f.struct_name {
                                return Err(src_info.format_error(span, &format!("Method '{}' must be called with dot syntax", name)));
                            }
                        }
                        // Обычная функция
                        let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                        out.push_str(&format!("{}{}(", pad, c_name));
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 { out.push_str(", "); }
                            generate_expr(arg, out, scope, functions, src_info)?;
                        }
                        out.push_str(");\n");
                    }
                }
            }
        }
    }
    Ok(())
}

fn generate_expr(
    expr: &Expr,
    out: &mut String,
    scope: &Scope,
    functions: &[Function],
    src_info: &SourceInfo,
) -> Result<(), String> {
    match expr {
        Expr::Number(n, _) => out.push_str(&n.to_string()),
        Expr::DoubleLiteral(d, _) => out.push_str(&format!("{}", d)),
        Expr::StringLiteral(s, _) => out.push_str(&format!("\"{}\"", s)),
        Expr::Variable(name, _) => out.push_str(name),
        Expr::Input(prompt, _) => out.push_str(&format!("side_input(\"{}\")", prompt)),
        Expr::Call { name, args, span } => {
            match name.as_str() {
                "len" => {
                    if args.len() != 1 {
                        return Err(src_info.format_error(span, "len requires 1 argument"));
                    }
                    let tp = infer_type(&args[0], scope, functions, None, src_info, span)?;
                    if tp == Type::Array || tp == Type::DoubleArray {
                        let arr_name = extract_var_name(&args[0], src_info, span)?;
                        out.push_str(&format!("{}_size", arr_name));
                    } else if tp == Type::Str {
                        out.push_str("strlen(");
                        generate_expr(&args[0], out, scope, functions, src_info)?;
                        out.push_str(")");
                    } else {
                        return Err(src_info.format_error(span, "len argument must be array or string"));
                    }
                }
                "time" => out.push_str("side_time()"),
                "sqrt" | "pow" | "fabs" | "rand" => {
                    out.push_str(name);
                    out.push('(');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        generate_expr(arg, out, scope, functions, src_info)?;
                    }
                    out.push(')');
                }
                "str" => {
                    if args.len() != 1 {
                        return Err(src_info.format_error(span, "str() requires 1 argument"));
                    }
                    let tp = infer_type(&args[0], scope, functions, None, src_info, span)?;
                    if tp == Type::Double {
                        out.push_str("side_str_double(");
                    } else {
                        out.push_str("side_str(");
                    }
                    generate_expr(&args[0], out, scope, functions, src_info)?;
                    out.push(')');
                }
                "int" => {
                    out.push_str("atoi(");
                    generate_expr(&args[0], out, scope, functions, src_info)?;
                    out.push(')');
                }
                _ => {
                    let func = functions.iter().find(|f| f.name == *name);
                    if let Some(f) = func {
                        if let Some(ref _struct_name) = f.struct_name {
                            return Err(src_info.format_error(span, &format!("Method '{}' must be called on an instance", name)));
                        }
                    }
                    let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                    out.push_str(&format!("{}(", c_name));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        generate_expr(arg, out, scope, functions, src_info)?;
                    }
                    out.push(')');
                }
            }
        }
        Expr::MethodCall { instance, method, args, span } => {
            let instance_type = infer_type(instance, scope, functions, None, src_info, span)?;
            let struct_name = match instance_type {
                Type::Struct(name) => name,
                _ => return Err(src_info.format_error(span, "Method call on non-struct type")),
            };
            let _method_func = functions.iter().find(|f| {
                f.struct_name.as_ref() == Some(&struct_name) && f.name == *method
            }).ok_or_else(|| {
                src_info.format_error(span, &format!("Method '{}' not found for struct '{}'", method, struct_name))
            })?;
            let c_name = format!("side_{}_{}", struct_name, method);
            out.push_str(&format!("{}(", c_name));
            generate_expr(instance, out, scope, functions, src_info)?;
            for arg in args {
                out.push_str(", ");
                generate_expr(arg, out, scope, functions, src_info)?;
            }
            out.push(')');
        }
        Expr::StructLiteral { name, fields, span: _ } => {
            out.push_str(&format!("(side_{}){{", name));
            for (i, f) in fields.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(f, out, scope, functions, src_info)?;
            }
            out.push('}');
        }
        Expr::ArrayLiteral(elements, _) => {
            out.push_str("{");
            for (i, e) in elements.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(e, out, scope, functions, src_info)?;
            }
            out.push('}');
        }
        Expr::Index { array, index, span: _ } => {
            generate_expr(array, out, scope, functions, src_info)?;
            out.push('[');
            generate_expr(index, out, scope, functions, src_info)?;
            out.push(']');
        }
        Expr::FieldAccess { expr, field, span: _ } => {
            generate_expr(expr, out, scope, functions, src_info)?;
            out.push('.');
            out.push_str(field);
        }
        Expr::Binary { left, op, right, span } => {
            let tl = infer_type(left, scope, functions, None, src_info, span)?;
            let tr = infer_type(right, scope, functions, None, src_info, span)?;

            if tl == Type::Str && tr == Type::Str {
                match op {
                    BinOp::Add => {
                        out.push_str("side_str_concat(");
                        generate_expr(left, out, scope, functions, src_info)?;
                        out.push_str(", ");
                        generate_expr(right, out, scope, functions, src_info)?;
                        out.push_str(")");
                    }
                    BinOp::Eq => {
                        out.push_str("(strcmp(");
                        generate_expr(left, out, scope, functions, src_info)?;
                        out.push_str(", ");
                        generate_expr(right, out, scope, functions, src_info)?;
                        out.push_str(") == 0)");
                    }
                    BinOp::NotEq => {
                        out.push_str("(strcmp(");
                        generate_expr(left, out, scope, functions, src_info)?;
                        out.push_str(", ");
                        generate_expr(right, out, scope, functions, src_info)?;
                        out.push_str(") != 0)");
                    }
                    _ => return Err(src_info.format_error(span, "Unsupported operator for strings")),
                }
                return Ok(());
            }

            if tl == Type::Str || tr == Type::Str {
                return Err(src_info.format_error(span, "String operations only allowed with +, ==, !="));
            }

            out.push('(');
            generate_expr(left, out, scope, functions, src_info)?;
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
            generate_expr(right, out, scope, functions, src_info)?;
            out.push(')');
        }
        Expr::Unary { op, expr, span: _ } => {
            match op {
                UnaryOp::Not => {
                    out.push_str("!(");
                    generate_expr(expr, out, scope, functions, src_info)?;
                    out.push(')');
                }
                UnaryOp::Neg => {
                    out.push_str("-(");
                    generate_expr(expr, out, scope, functions, src_info)?;
                    out.push(')');
                }
            }
        }
        Expr::Ternary { condition, then_expr, else_expr, span: _ } => {
            out.push_str("(");
            generate_expr(condition, out, scope, functions, src_info)?;
            out.push_str(" ? ");
            generate_expr(then_expr, out, scope, functions, src_info)?;
            out.push_str(" : ");
            generate_expr(else_expr, out, scope, functions, src_info)?;
            out.push_str(")");
        }
    }
    Ok(())
}

// ---------- Вспомогательные функции для типов ----------
fn type_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        (Type::Int, Type::Int) => true,
        (Type::Double, Type::Double) => true,
        (Type::Double, Type::Int) => true,
        (Type::Str, Type::Str) => true,
        (Type::Array, Type::Array) => true,
        (Type::DoubleArray, Type::DoubleArray) => true,
        (Type::Array, Type::DoubleArray) => false,
        (Type::DoubleArray, Type::Array) => false,
        (Type::Struct(a), Type::Struct(b)) => a == b,
        _ => false,
    }
}

fn infer_type(
    expr: &Expr,
    scope: &Scope,
    functions: &[Function],
    expected: Option<&Type>,
    src_info: &SourceInfo,
    span: &Span,
) -> Result<Type, String> {
    match expr {
        Expr::Number(_, _) => Ok(Type::Int),
        Expr::DoubleLiteral(_, _) => Ok(Type::Double),
        Expr::StringLiteral(_, _) => Ok(Type::Str),
        Expr::Variable(name, _) => scope.get(name)
            .ok_or(src_info.format_error(span, &format!("Variable '{}' not declared", name))),
        Expr::Input(_, _) => Ok(Type::Int),
        Expr::Call { name, args: _, span: call_span } => match name.as_str() {
            "len" => Ok(Type::Int),
            "time" => Ok(Type::Int),
            "sqrt" | "pow" | "fabs" => Ok(Type::Double),
            "rand" => Ok(Type::Int),
            "str" => Ok(Type::Str),
            "int" => Ok(Type::Int),
            _ => {
                let func = functions.iter().find(|f| f.name == *name)
                    .ok_or(src_info.format_error(call_span, &format!("Function '{}' not defined", name)))?;
                Ok(func.return_type.clone())
            }
        },
        Expr::MethodCall { instance, method, args: _, span: method_span } => {
            let instance_type = infer_type(instance, scope, functions, None, src_info, method_span)?;
            let struct_name = match instance_type {
                Type::Struct(name) => name,
                _ => return Err(src_info.format_error(method_span, "Method call on non-struct type")),
            };
            let _method_func = functions.iter().find(|f| {
                f.struct_name.as_ref() == Some(&struct_name) && f.name == *method
            }).ok_or_else(|| {
                src_info.format_error(method_span, &format!("Method '{}' not found for struct '{}'", method, struct_name))
            })?;
            Ok(_method_func.return_type.clone())
        }
        Expr::StructLiteral { name, .. } => Ok(Type::Struct(name.clone())),
        Expr::ArrayLiteral(elements, _) => {
            if elements.is_empty() {
                if let Some(exp) = expected {
                    match exp {
                        Type::DoubleArray => return Ok(Type::DoubleArray),
                        Type::Array => return Ok(Type::Array),
                        _ => return Ok(Type::Array),
                    }
                } else {
                    return Ok(Type::Array);
                }
            }
            let first_type = infer_type(&elements[0], scope, functions, None, src_info, span)?;
            if first_type == Type::Double {
                Ok(Type::DoubleArray)
            } else {
                Ok(Type::Array)
            }
        }
        Expr::Index { array, .. } => {
            let arr_type = infer_type(array, scope, functions, None, src_info, span)?;
            match arr_type {
                Type::Array => Ok(Type::Int),
                Type::DoubleArray => Ok(Type::Double),
                _ => Err(src_info.format_error(span, "Indexing non-array")),
            }
        }
        Expr::FieldAccess { expr: _, field: _, span: _ } => {
            Ok(Type::Int)
        }
        Expr::Binary { left, op, right, span: bin_span } => {
            let tl = infer_type(left, scope, functions, None, src_info, bin_span)?;
            let tr = infer_type(right, scope, functions, None, src_info, bin_span)?;
            match (&tl, &tr, op) {
                (Type::Str, Type::Str, BinOp::Add) => Ok(Type::Str),
                (Type::Str, Type::Str, BinOp::Eq) => Ok(Type::Int),
                (Type::Str, Type::Str, BinOp::NotEq) => Ok(Type::Int),
                (Type::Str, _, _) | (_, Type::Str, _) => {
                    Err(src_info.format_error(bin_span, "String operations only allowed with +, ==, !="))
                }
                _ => {
                    if tl == Type::Double || tr == Type::Double {
                        Ok(Type::Double)
                    } else {
                        Ok(Type::Int)
                    }
                }
            }
        }
        Expr::Unary { op: _, expr, span: unary_span } => infer_type(expr, scope, functions, None, src_info, unary_span),
        Expr::Ternary { condition, then_expr, else_expr, span } => {
            let cond_type = infer_type(condition, scope, functions, None, src_info, span)?;
            if cond_type != Type::Int {
                return Err(src_info.format_error(span, "Condition must be integer"));
            }
            let then_type = infer_type(then_expr, scope, functions, None, src_info, span)?;
            let else_type = infer_type(else_expr, scope, functions, None, src_info, span)?;
            if !type_compatible(&then_type, &else_type) {
                return Err(src_info.format_error(span, &format!("Ternary branches have incompatible types: {:?} and {:?}", then_type, else_type)));
            }
            // Определяем результирующий тип
            if then_type == Type::Double || else_type == Type::Double {
                Ok(Type::Double)
            } else if then_type == Type::Str && else_type == Type::Str {
                Ok(Type::Str)
            } else {
                Ok(Type::Int)
            }
        }
    }
}

fn type_to_c(tp: &Type) -> String {
    match tp {
        Type::Int => "int".to_string(),
        Type::Double => "double".to_string(),
        Type::Str => "const char*".to_string(),
        Type::Array => "int*".to_string(),
        Type::DoubleArray => "double*".to_string(),
        Type::Struct(name) => format!("side_{}", name),
    }
}

fn check_function_call(
    name: &str,
    args: &[Expr],
    functions: &[Function],
    scope: &Scope,
    src_info: &SourceInfo,
    span: &Span,
) -> Result<(), String> {
    match name {
        "push" | "pop" => {
            let arr_name = extract_var_name(&args[0], src_info, span)?;
            let tp = scope.get(arr_name).ok_or(src_info.format_error(span, &format!("Variable '{}' not found", arr_name)))?;
            if tp != Type::Array && tp != Type::DoubleArray {
                return Err(src_info.format_error(span, &format!("First argument of '{}' must be an array", name)));
            }
            if name == "push" && args.len() != 2 {
                return Err(src_info.format_error(span, "push requires 2 arguments"));
            }
            if name == "pop" && args.len() != 1 {
                return Err(src_info.format_error(span, "pop requires 1 argument"));
            }
            if name == "push" {
                let expected_elem = if tp == Type::Array { Type::Int } else { Type::Double };
                let elem_type = infer_type(&args[1], scope, functions, Some(&expected_elem), src_info, span)?;
                if !type_compatible(&expected_elem, &elem_type) {
                    return Err(src_info.format_error(span, &format!("push: expected {} element, got {:?}", if tp == Type::Array { "int" } else { "double" }, elem_type)));
                }
            }
            return Ok(());
        }
        "len" => {
            if args.len() != 1 {
                return Err(src_info.format_error(span, "len requires 1 argument"));
            }
            let tp = infer_type(&args[0], scope, functions, None, src_info, span)?;
            if tp != Type::Array && tp != Type::DoubleArray && tp != Type::Str {
                return Err(src_info.format_error(span, "len argument must be array or string"));
            }
            return Ok(());
        }
        "time" | "rand" => {
            if !args.is_empty() {
                return Err(src_info.format_error(span, &format!("'{}' takes no arguments", name)));
            }
            return Ok(());
        }
        "sqrt" | "fabs" => {
            if args.len() != 1 {
                return Err(src_info.format_error(span, &format!("'{}' requires 1 argument", name)));
            }
            let tp = infer_type(&args[0], scope, functions, None, src_info, span)?;
            if tp != Type::Int && tp != Type::Double {
                return Err(src_info.format_error(span, &format!("'{}' argument must be numeric", name)));
            }
            return Ok(());
        }
        "pow" => {
            if args.len() != 2 {
                return Err(src_info.format_error(span, "pow requires 2 arguments"));
            }
            for a in args {
                let tp = infer_type(a, scope, functions, None, src_info, span)?;
                if tp != Type::Int && tp != Type::Double {
                    return Err(src_info.format_error(span, "pow arguments must be numeric"));
                }
            }
            return Ok(());
        }
        "str" | "int" => {
            if args.len() != 1 {
                return Err(src_info.format_error(span, &format!("'{}' requires 1 argument", name)));
            }
            return Ok(());
        }
        _ => {}
    }

    let func = functions.iter().find(|f| f.name == name && f.struct_name.is_none())
        .ok_or(src_info.format_error(span, &format!("Function '{}' not defined", name)))?;
    if func.params.len() != args.len() {
        return Err(src_info.format_error(span, &format!("Function '{}' expects {} arguments, got {}", name, func.params.len(), args.len())));
    }
    for (param, arg) in func.params.iter().zip(args.iter()) {
        let actual = infer_type(arg, scope, functions, Some(&param.param_type), src_info, span)?;
        if !type_compatible(&param.param_type, &actual) {
            return Err(src_info.format_error(span, &format!("Argument type mismatch in '{}': parameter '{}' expects {:?}, got {:?}",
                name, param.name, param.param_type, actual)));
        }
    }
    Ok(())
}

fn extract_var_name<'a>(expr: &'a Expr, _src_info: &SourceInfo, _span: &Span) -> Result<&'a str, String> {
    if let Expr::Variable(name, _) = expr {
        Ok(name.as_str())
    } else {
        Err("Expected a variable name".into())
    }
}

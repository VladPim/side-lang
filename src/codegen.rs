use crate::ast::*;
use std::collections::HashMap;

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

pub fn generate(program: &Program) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <math.h>\n#include <time.h>\n\n");

    for c in &program.constants {
        let c_type = match &c.value { Expr::DoubleLiteral(_) => "const double", Expr::StringLiteral(_) => "const char*", _ => "const int" };
        out.push_str(&format!("{} {} = ", c_type, c.name));
        generate_expr(&c.value, &mut out, &Scope::new())?;
        out.push_str(";\n");
    }

    for s in &program.structs {
        out.push_str(&format!("typedef struct {{\n"));
        for f in &s.fields {
            match f.field_type {
                Type::Int => out.push_str(&format!("    int {};\n", f.name)),
                Type::Double => out.push_str(&format!("    double {};\n", f.name)),
                Type::Str => out.push_str(&format!("    const char* {};\n", f.name)),
                Type::Array => out.push_str(&format!("    int* {};\n", f.name)),
                Type::DoubleArray => out.push_str(&format!("    double* {};\n", f.name)),
                Type::Struct(ref name) => out.push_str(&format!("    side_{} {};\n", name, f.name)),
            }
        }
        out.push_str(&format!("}} side_{};\n\n", s.name));
    }

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
"#);

    // Генерируем сначала все функции, кроме main, чтобы main могла их вызывать
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
        let c_name = if func.name == "main" { "side_main".to_string() } else { format!("side_{}", func.name) };
        let params_str = func.params.iter().map(|p| match p.param_type {
            Type::Int => format!("int {}", p.name),
            Type::Double => format!("double {}", p.name),
            Type::Str => format!("const char* {}", p.name),
            Type::Array => format!("int* {}", p.name),
            Type::DoubleArray => format!("double* {}", p.name),
            Type::Struct(ref name) => format!("side_{} {}", name, p.name),
        }).collect::<Vec<_>>().join(", ");

        out.push_str(&format!("{} {}(", type_to_c(&func.return_type), c_name));
        out.push_str(&params_str);
        out.push_str(") {\n");

        let mut scope = Scope::new();
        for p in &func.params { scope.declare(&p.name, p.param_type.clone()); }
        generate_stmts(&func.body, &mut out, 1, &mut scope, &program.functions, &func.return_type)?;
        out.push_str("    return 0;\n}\n\n");
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
) -> Result<(), String> {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            Stmt::Let { name, var_type, value } => {
                let declared_type = var_type.clone();
                let actual_type = infer_type(value, scope, functions, declared_type.as_ref())?;
                let final_type = match declared_type {
                    Some(ref dt) => {
                        if !type_compatible(dt, &actual_type) {
                            return Err(format!(
                                "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                                actual_type, name, dt
                            ));
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
                    if let Expr::ArrayLiteral(elems) = value {
                        if !elems.is_empty() {
                            out.push_str(&format!("{}{{\n", pad));
                            out.push_str(&format!("{}    {} _arr_vals[] = {{", pad, elem_c_type));
                            for (i, e) in elems.iter().enumerate() {
                                if i > 0 { out.push_str(", "); }
                                generate_expr(e, out, scope)?;
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
                    generate_expr(value, out, scope)?;
                    out.push_str(";\n");
                }
                scope.declare(name, final_type);
            }
            Stmt::Assign { name, value } => {
                let var_type = scope.get(name)
                    .ok_or(format!("Variable '{}' not declared in this scope", name))?;
                let actual_type = infer_type(value, scope, functions, Some(&var_type))?;
                if !type_compatible(&var_type, &actual_type) {
                    return Err(format!(
                        "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                        actual_type, name, var_type
                    ));
                }
                out.push_str(&format!("{}{} = ", pad, name));
                generate_expr(value, out, scope)?;
                out.push_str(";\n");
            }
            Stmt::AssignIndex { name, index, value } => {
                let var_type = scope.get(name)
                    .ok_or(format!("Variable '{}' not declared", name))?;
                let expected_elem = match var_type {
                    Type::Array => Type::Int,
                    Type::DoubleArray => Type::Double,
                    _ => return Err("Indexing non-array".into()),
                };
                let actual_type = infer_type(value, scope, functions, Some(&expected_elem))?;
                if !type_compatible(&expected_elem, &actual_type) {
                    return Err(format!(
                        "Type mismatch: cannot assign {:?} to element of {:?}",
                        actual_type, var_type
                    ));
                }
                out.push_str(&format!("{}{}[", pad, name));
                generate_expr(index, out, scope)?;
                out.push_str("] = ");
                generate_expr(value, out, scope)?;
                out.push_str(";\n");
            }
            Stmt::IoPrint(args) => {
                for arg in args {
                    let tp = infer_type(arg, scope, functions, None)?;
                    let format = match tp {
                        Type::Double => "%f",
                        Type::Str => "%s",
                        _ => "%d",
                    };
                    out.push_str(&format!("{}printf(\"{}\", ", pad, format));
                    generate_expr(arg, out, scope)?;
                    out.push_str(");\n");
                }
                out.push_str(&format!("{}printf(\"\\n\");\n", pad));
            }
            Stmt::If { condition, then_body, else_body } => {
                out.push_str(&format!("{}if (", pad));
                generate_expr(condition, out, scope)?;
                out.push_str(") {\n");
                let mut block_scope = scope.push();
                generate_stmts(then_body, out, indent + 1, &mut block_scope, functions, expected_return_type)?;
                out.push_str(&format!("{}}}\n", pad));
                if let Some(else_stmts) = else_body {
                    out.push_str(&format!("{}else {{\n", pad));
                    let mut else_scope = scope.push();
                    generate_stmts(else_stmts, out, indent + 1, &mut else_scope, functions, expected_return_type)?;
                    out.push_str(&format!("{}}}\n", pad));
                }
            }
            Stmt::While { condition, body } => {
                out.push_str(&format!("{}while (", pad));
                generate_expr(condition, out, scope)?;
                out.push_str(") {\n");
                let mut while_scope = scope.push();
                generate_stmts(body, out, indent + 1, &mut while_scope, functions, expected_return_type)?;
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::For { var_name, start, condition, step, body } => {
                let start_type = infer_type(start, scope, functions, None)?;
                let c_type = type_to_c(&start_type);
                out.push_str(&format!("{}{{\n", pad));
                out.push_str(&format!("{}    {} {} = ", pad, c_type, var_name));
                generate_expr(start, out, scope)?;
                out.push_str(";\n");
                let mut for_scope = scope.push();
                for_scope.declare(var_name, start_type.clone());
                out.push_str(&format!("{}    while (", pad));
                generate_expr(condition, out, &for_scope)?;
                out.push_str(") {\n");
                let mut body_scope = for_scope.push();
                generate_stmts(body, out, indent + 2, &mut body_scope, functions, expected_return_type)?;
                out.push_str(&format!("{}        {} = ", pad, var_name));
                generate_expr(step, out, &for_scope)?;
                out.push_str(";\n");
                out.push_str(&format!("{}    }}\n", pad));
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::Return(expr) => {
                let actual_type = infer_type(expr, scope, functions, Some(expected_return_type))?;
                if !type_compatible(expected_return_type, &actual_type) {
                    return Err(format!(
                        "Return type mismatch: expected {:?}, got {:?}",
                        expected_return_type, actual_type
                    ));
                }
                out.push_str(&format!("{}return ", pad));
                generate_expr(expr, out, scope)?;
                out.push_str(";\n");
            }
            Stmt::Break => out.push_str(&format!("{}break;\n", pad)),
            Stmt::Continue => out.push_str(&format!("{}continue;\n", pad)),
            Stmt::CallStmt { name, args } => {
                check_function_call(name, args, functions, scope)?;
                match name.as_str() {
                    "push" => {
                        let arr_name = extract_var_name(&args[0])?;
                        let arr_type = scope.get(arr_name)
                            .ok_or(format!("Variable '{}' not found", arr_name))?;
                        if arr_type == Type::Array {
                            out.push_str(&format!("{}side_arr_push(&{}, &{}_size, &{}_cap, ", pad, arr_name, arr_name, arr_name));
                        } else if arr_type == Type::DoubleArray {
                            out.push_str(&format!("{}side_arr_push_double(&{}, &{}_size, &{}_cap, ", pad, arr_name, arr_name, arr_name));
                        } else {
                            return Err("First argument of push must be an array".into());
                        }
                        generate_expr(&args[1], out, scope)?;
                        out.push_str(");\n");
                    }
                    "pop" => {
                        let arr_name = extract_var_name(&args[0])?;
                        let arr_type = scope.get(arr_name)
                            .ok_or(format!("Variable '{}' not found", arr_name))?;
                        if arr_type == Type::Array {
                            out.push_str(&format!("{}side_arr_pop(&{}, &{}_size, &{}_cap);\n", pad, arr_name, arr_name, arr_name));
                        } else if arr_type == Type::DoubleArray {
                            out.push_str(&format!("{}side_arr_pop_double(&{}, &{}_size, &{}_cap);\n", pad, arr_name, arr_name, arr_name));
                        } else {
                            return Err("First argument of pop must be an array".into());
                        }
                    }
                    _ => {
                        let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                        out.push_str(&format!("{}{}(", pad, c_name));
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 { out.push_str(", "); }
                            generate_expr(arg, out, scope)?;
                        }
                        out.push_str(");\n");
                    }
                }
            }
        }
    }
    Ok(())
}

fn generate_expr(expr: &Expr, out: &mut String, scope: &Scope) -> Result<(), String> {
    match expr {
        Expr::Number(n) => out.push_str(&n.to_string()),
        Expr::DoubleLiteral(d) => out.push_str(&format!("{}", d)),
        Expr::StringLiteral(s) => out.push_str(&format!("\"{}\"", s)),
        Expr::Variable(name) => out.push_str(name),
        Expr::Input(prompt) => out.push_str(&format!("side_input(\"{}\")", prompt)),
        Expr::Call { name, args } => {
            match name.as_str() {
                "len" => {
                    if args.len() != 1 {
                        return Err("len requires 1 argument".into());
                    }
                    let tp = infer_type(&args[0], scope, &[], None)?;
                    if tp == Type::Array || tp == Type::DoubleArray {
                        let arr_name = extract_var_name(&args[0])?;
                        out.push_str(&format!("{}_size", arr_name));
                    } else if tp == Type::Str {
                        out.push_str("strlen(");
                        generate_expr(&args[0], out, scope)?;
                        out.push_str(")");
                    } else {
                        return Err("len argument must be array or string".into());
                    }
                }
                "time" => out.push_str("side_time()"),
                "sqrt" | "pow" | "fabs" | "rand" => {
                    out.push_str(name);
                    out.push('(');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        generate_expr(arg, out, scope)?;
                    }
                    out.push(')');
                }
                "str" => {
                    if args.len() != 1 { return Err("str() requires 1 argument".into()); }
                    let tp = infer_type(&args[0], scope, &[], None)?;
                    if tp == Type::Double {
                        out.push_str("side_str_double(");
                    } else {
                        out.push_str("side_str(");
                    }
                    generate_expr(&args[0], out, scope)?;
                    out.push(')');
                }
                "int" => {
                    out.push_str("atoi(");
                    generate_expr(&args[0], out, scope)?;
                    out.push(')');
                }
                _ => {
                    let c_name = if name == "main" { "side_main".to_string() } else { format!("side_{}", name) };
                    out.push_str(&format!("{}(", c_name));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        generate_expr(arg, out, scope)?;
                    }
                    out.push(')');
                }
            }
        }
        Expr::StructLiteral { name, fields } => {
            out.push_str(&format!("(side_{}){{", name));
            for (i, f) in fields.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(f, out, scope)?;
            }
            out.push('}');
        }
        Expr::ArrayLiteral(elements) => {
            out.push_str("{");
            for (i, e) in elements.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                generate_expr(e, out, scope)?;
            }
            out.push('}');
        }
        Expr::Index { array, index } => {
            generate_expr(array, out, scope)?;
            out.push('[');
            generate_expr(index, out, scope)?;
            out.push(']');
        }
        Expr::FieldAccess { expr, field } => {
            generate_expr(expr, out, scope)?;
            out.push('.');
            out.push_str(field);
        }
        Expr::Binary { left, op, right } => {
            out.push('(');
            generate_expr(left, out, scope)?;
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
            generate_expr(right, out, scope)?;
            out.push(')');
        }
        Expr::Unary { op, expr } => {
            match op {
                UnaryOp::Not => {
                    out.push_str("!(");
                    generate_expr(expr, out, scope)?;
                    out.push(')');
                }
                UnaryOp::Neg => {
                    out.push_str("-(");
                    generate_expr(expr, out, scope)?;
                    out.push(')');
                }
            }
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

fn infer_type(expr: &Expr, scope: &Scope, functions: &[Function], expected: Option<&Type>) -> Result<Type, String> {
    match expr {
        Expr::Number(_) => Ok(Type::Int),
        Expr::DoubleLiteral(_) => Ok(Type::Double),
        Expr::StringLiteral(_) => Ok(Type::Str),
        Expr::Variable(name) => scope.get(name)
            .ok_or(format!("Variable '{}' not declared", name)),
        Expr::Input(_) => Ok(Type::Int),
        Expr::Call { name, args } => match name.as_str() {
            "len" => Ok(Type::Int),
            "time" => Ok(Type::Int),
            "sqrt" | "pow" | "fabs" => Ok(Type::Double),
            "rand" => Ok(Type::Int),
            "str" => Ok(Type::Str),
            "int" => Ok(Type::Int),
            _ => {
                let func = functions.iter().find(|f| f.name == *name)
                    .ok_or(format!("Function '{}' not defined", name))?;
                Ok(func.return_type.clone())
            }
        },
        Expr::StructLiteral { name, .. } => Ok(Type::Struct(name.clone())),
        Expr::ArrayLiteral(elements) => {
            if elements.is_empty() {
                // Пустой массив: если есть ожидаемый тип, используем его
                if let Some(exp) = expected {
                    match exp {
                        Type::DoubleArray => return Ok(Type::DoubleArray),
                        Type::Array => return Ok(Type::Array),
                        _ => return Ok(Type::Array), // по умолчанию int[]
                    }
                } else {
                    return Ok(Type::Array); // int[] по умолчанию
                }
            }
            // Непустой массив – проверяем первый элемент
            let first_type = infer_type(&elements[0], scope, functions, None)?;
            if first_type == Type::Double {
                Ok(Type::DoubleArray)
            } else {
                Ok(Type::Array)
            }
        }
        Expr::Index { array, .. } => {
            let arr_type = infer_type(array, scope, functions, None)?;
            match arr_type {
                Type::Array => Ok(Type::Int),
                Type::DoubleArray => Ok(Type::Double),
                _ => Err("Indexing non-array".into()),
            }
        }
        Expr::FieldAccess { .. } => Ok(Type::Int),
        Expr::Binary { left, op: _, right } => {
            let tl = infer_type(left, scope, functions, None)?;
            let tr = infer_type(right, scope, functions, None)?;
            if tl == Type::Double || tr == Type::Double {
                Ok(Type::Double)
            } else {
                Ok(Type::Int)
            }
        }
        Expr::Unary { op: _, expr } => infer_type(expr, scope, functions, None),
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

fn check_function_call(name: &str, args: &[Expr], functions: &[Function], scope: &Scope) -> Result<(), String> {
    match name {
        "push" | "pop" => {
            let arr_name = extract_var_name(&args[0])?;
            let tp = scope.get(arr_name).ok_or(format!("Variable '{}' not found", arr_name))?;
            if tp != Type::Array && tp != Type::DoubleArray {
                return Err(format!("First argument of '{}' must be an array", name));
            }
            if name == "push" && args.len() != 2 {
                return Err("push requires 2 arguments".into());
            }
            if name == "pop" && args.len() != 1 {
                return Err("pop requires 1 argument".into());
            }
            if name == "push" {
                let expected_elem = if tp == Type::Array { Type::Int } else { Type::Double };
                let elem_type = infer_type(&args[1], scope, functions, Some(&expected_elem))?;
                if !type_compatible(&expected_elem, &elem_type) {
                    return Err(format!("push: expected {} element, got {:?}", if tp == Type::Array { "int" } else { "double" }, elem_type));
                }
            }
            return Ok(());
        }
        "len" => {
            if args.len() != 1 { return Err("len requires 1 argument".into()); }
            let tp = infer_type(&args[0], scope, functions, None)?;
            if tp != Type::Array && tp != Type::DoubleArray && tp != Type::Str {
                return Err("len argument must be array or string".into());
            }
            return Ok(());
        }
        "time" | "rand" => {
            if !args.is_empty() { return Err(format!("'{}' takes no arguments", name)); }
            return Ok(());
        }
        "sqrt" | "fabs" => {
            if args.len() != 1 { return Err(format!("'{}' requires 1 argument", name)); }
            let tp = infer_type(&args[0], scope, functions, None)?;
            if tp != Type::Int && tp != Type::Double { return Err(format!("'{}' argument must be numeric", name)); }
            return Ok(());
        }
        "pow" => {
            if args.len() != 2 { return Err("pow requires 2 arguments".into()); }
            for a in args {
                let tp = infer_type(a, scope, functions, None)?;
                if tp != Type::Int && tp != Type::Double { return Err("pow arguments must be numeric".into()); }
            }
            return Ok(());
        }
        "str" | "int" => {
            if args.len() != 1 { return Err(format!("'{}' requires 1 argument", name)); }
            return Ok(());
        }
        _ => {}
    }

    // Пользовательские функции
    let func = functions.iter().find(|f| f.name == name)
        .ok_or(format!("Function '{}' not defined", name))?;
    if func.params.len() != args.len() {
        return Err(format!("Function '{}' expects {} arguments, got {}", name, func.params.len(), args.len()));
    }
    for (param, arg) in func.params.iter().zip(args.iter()) {
        let actual = infer_type(arg, scope, functions, Some(&param.param_type))?;
        if !type_compatible(&param.param_type, &actual) {
            return Err(format!("Argument type mismatch in '{}': parameter '{}' expects {:?}, got {:?}",
                name, param.name, param.param_type, actual));
        }
    }
    Ok(())
}

fn extract_var_name(expr: &Expr) -> Result<&str, String> {
    if let Expr::Variable(name) = expr {
        Ok(name.as_str())
    } else {
        Err("Expected a variable name".into())
    }
}

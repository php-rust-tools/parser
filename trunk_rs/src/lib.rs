use trunk_lexer::Lexer;
use trunk_parser::{Parser, Statement, Expression, InfixOp};

pub fn compile(file: String) -> Result<String, CompileError> {
    let contents = match std::fs::read_to_string(file) {
        Ok(c) => c,
        _ => return Err(CompileError::FileError)
    };

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse().unwrap();
    let (fns, main): (Vec<_>, Vec<_>) = ast.iter().partition(|n| match n {
        Statement::Function { .. } => true,
        _ => false
    });

    let mut source = String::from(r#"
mod prelude;
use prelude::*;

"#);

    for statement in fns {
        compile_statement(statement, &mut source);
    }

    source.push_str("fn main() {");
    for statement in main {
        compile_statement(statement, &mut source);
    }
    source.push_str("}");

    Ok(source)
}

fn compile_statement(statement: &Statement, source: &mut String) {
    match statement {
        Statement::Function { name, params, body, .. } => {
            source.push_str("fn ");
            source.push_str(&name.name);
            source.push('(');
            for (i, param) in params.iter().enumerate() {
                source.push_str(match &param.name {
                    Expression::Variable(n) => &n,
                    _ => unreachable!()
                });
                source.push_str(": Value");
                if i != params.len() - 1 {
                    source.push(',');
                }
            }
            source.push_str(") -> Value");
            source.push('{');
            for s in body {
                compile_statement(s, source);
            }
            source.push('}');
        },
        Statement::If { condition, then, .. } => {
            source.push_str("if ");
            source.push_str(&compile_expression(condition));
            source.push('{');
            for s in then {
                compile_statement(s, source);
            }
            source.push('}');
        },
        Statement::Return { value } => {
            source.push_str("return");
            if value.is_some() {
                source.push(' ');
                source.push_str(&compile_expression(value.as_ref().unwrap()));
            }
            source.push(';');
        },
        Statement::Echo { values } => {
            source.push_str("_internal_echo(&[");
            for value in values {
                source.push_str(&compile_expression(value));
            }
            source.push_str("]);");
        },
        _ => todo!(),
    }
}

fn compile_expression(expression: &Expression) -> String {
    match expression {
        Expression::Variable(var) => format!("{}.clone()", var.to_string()),
        Expression::Infix(lhs, op, rhs) => {
            if op == &InfixOp::Concat {
                return format!("_internal_concat({}, {})", compile_expression(lhs), compile_expression(rhs));
            }

            format!(
                "{} {} {}",
                compile_expression(lhs),
                match op {
                    InfixOp::Add => "+",
                    InfixOp::LessThan => "<",
                    InfixOp::Sub => "-",
                    _ => todo!("{:?}", op)
                },
                compile_expression(rhs)
            )
        },
        Expression::Identifier(i) => {
            match &i[..] {
                "PHP_EOL" => format!("_internal_constant_to_string(\"{}\").into()", i),
                _ => i.to_string()
            }
        },
        Expression::Call(target, args) => {
            let mut buffer = format!("{}(", compile_expression(target));
            for (i, arg) in args.iter().enumerate() {
                buffer.push_str(&compile_expression(arg));

                if i != args.len() - 1 {
                    buffer.push(',');
                }
            }
            buffer.push(')');
            buffer
        },
        Expression::Int(n) => format!("Value::Int({})", n),
        _ => todo!("{:?}", expression),
    }
}

#[derive(Debug)]
pub enum CompileError {
    FileError,
}
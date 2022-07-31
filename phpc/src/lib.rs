use trunk_lexer::Lexer;
use trunk_parser::{Parser, Statement, Expression};

mod runtime;

pub fn compile(file: String) -> Result<String, CompileError> {
    let contents = match std::fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(_) => return Err(CompileError::FailedToReadFile),
    };

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    let (fns, main): (Vec<_>, Vec<_>) = ast.iter().partition(|statement| match statement {
        Statement::Function { .. } => true,
        _ => false,
    });

    let mut source = String::new();
    source.push_str(include_str!("./runtime.rs"));

    for function in fns {
        compile_function(function, &mut source)?;
    }

    source.push_str("fn main() {");

    for statement in main {
        compile_statement(statement, &mut source)?;
    }

    source.push('}');

    Ok(source)
}

fn compile_function(function: &Statement, source: &mut String) -> Result<(), CompileError> {
    let (name, params, body) = match function {
        Statement::Function { name, params, body, .. } => (name, params, body),
        _ => unreachable!(),
    };

    source.push_str("fn ");
    source.push_str(&name.name);
    source.push('(');

    for param in params {
        source.push_str(match &param.name {
            Expression::Variable(n) => &n,
            _ => unreachable!(),
        });
        source.push_str(": PhpValue, ");
    }

    source.push_str(") -> PhpValue {");

    for statement in body {
        compile_statement(statement, source)?;
    }

    source.push('}');

    Ok(())
}

fn compile_statement(statement: &Statement, source: &mut String) -> Result<(), CompileError> {
    match statement {
        Statement::Return { value } => {
            source.push_str("return");
            if let Some(value) = value {
                source.push(' ');
                source.push_str(&compile_expression(value)?);
            } else {
                todo!();
            }
            source.push(';');
        },
        Statement::Echo { values } => {
            for value in values {
                source.push_str("_php_echo(");
                source.push_str(&compile_expression(value)?);
                source.push_str(");");
            }
        },
        _ => todo!(),
    };

    Ok(())
}

fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        Expression::ConstantString(value) => {
            format!(r#"PhpValue::from("{}")"#, value)
        },
        Expression::Call(target, args) => {
            let mut buffer = String::new();

            buffer.push_str(&compile_expression(target)?);
            buffer.push('(');

            for arg in args {
                buffer.push_str(&compile_expression(arg)?);
                buffer.push_str(", ");
            }

            buffer.push(')');
            buffer
        },
        Expression::Identifier(i) => i.to_string(),
        _ => todo!(),
    };

    Ok(result)
}

#[derive(Debug)]
pub enum CompileError {
    FailedToReadFile,
}
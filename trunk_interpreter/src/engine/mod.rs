use hashbrown::HashMap;
use std::path::PathBuf;

use trunk_lexer::Lexer;
use trunk_parser::{Statement, Param, Expression, InfixOp, CastKind, MagicConst, Parser};

use self::environment::Environment;
use self::value::Value;

mod environment;
mod value;

pub struct Function {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Statement>,
}

pub struct Engine {
    pub(crate) filename: PathBuf,
    pub(crate) global_environment: Environment,
    pub(crate) function_table: HashMap<String, Function>,
    pub(crate) scopes: Vec<Environment>,
}

impl Engine {
    pub fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            global_environment: Environment::new(),
            function_table: HashMap::default(),
            scopes: Vec::new(),
        }
    }
}

pub fn eval(filename: PathBuf, program: Vec<Statement>) -> Result<(), Escape> {
    let mut engine = Engine::new(filename);
    for statement in program {
        eval_statement(&mut engine, statement)?;
    }
    Ok(())
}

pub enum Escape {
    Return(Value),
}

macro_rules! extract {
    ($target:expr, $pattern:pat, $return:expr) => {
        match $target {
            $pattern => $return,
            _ => unreachable!(),
        }
    };
}

pub fn eval_statement(engine: &mut Engine, statement: Statement) -> Result<(), Escape> {
    match statement {
        Statement::Function { .. } => eval_function(engine, statement)?,
        Statement::Echo { .. } => eval_echo(engine, statement)?,
        Statement::If { .. } => eval_if(engine, statement)?,
        Statement::Return { .. } => return Err(Escape::Return(eval_return(engine, statement))),
        Statement::Include { .. } => eval_include(engine, statement)?,
        _ => unimplemented!("{:?}", statement)
    };

    Ok(())
}

fn eval_function(engine: &mut Engine, statement: Statement) -> Result<(), Escape> {
    let (name, params, body, return_type, by_ref) = match statement {
        Statement::Function { name, params, body, return_type, by_ref } => (name, params, body, return_type, by_ref),
        _ => unreachable!(),
    };

    let func = Function {
        params,
        body
    };

    engine.function_table.insert(name.name.into(), func);

    Ok(())
}

fn eval_echo(engine: &mut Engine, statement: Statement) -> Result<(), Escape> {
    let values = match statement {
        Statement::Echo { values } => values,
        _ => unreachable!()
    };

    for value in values {
        let value = eval_expression(engine, value);

        print!("{}", value);
    }

    Ok(())
}

fn eval_if(engine: &mut Engine, statement: Statement) -> Result<(), Escape> {
    let (condition, then, ..) = match statement {
        Statement::If { condition, then, else_ifs, r#else } => (condition, then, else_ifs, r#else),
        _ => unreachable!()
    };

    let condition = eval_expression(engine, condition);

    if condition.is_truthy() {
        for statement in then {
            eval_statement(engine, statement)?;
        }
    }

    Ok(())
}

fn eval_return(engine: &mut Engine, statement: Statement) -> Value {
    let value = extract!(statement, Statement::Return { value }, value);

    if let Some(value) = value {
        return eval_expression(engine, value);
    }

    return Value::Null;
}

fn eval_include(engine: &mut Engine, statement: Statement) -> Result<(), Escape> {
    let (kind, path) = extract!(statement, Statement::Include { kind, path }, (kind, path));

    let path = extract!(eval_expression(engine, path), Value::String(value), value);
    let original_filename = engine.filename.clone();
    let contents = std::fs::read_to_string(&path).unwrap();

    engine.filename = PathBuf::from(&path);

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(None);
    let ast = parser.parse(tokens).unwrap();

    for statement in ast {
        eval_statement(engine, statement)?;
    }

    engine.filename = original_filename;

    Ok(())
}

fn eval_expression(engine: &mut Engine, expression: Expression) -> Value {
    match expression {
        Expression::Infix { .. } => eval_infix_expression(engine, expression),
        Expression::Call { .. } => eval_call_expression(engine, expression),
        Expression::Variable { .. } => eval_variable_expression(engine, expression),
        Expression::Cast { .. } => eval_cast_expression(engine, expression),
        Expression::MagicConst { constant } => eval_magic_const(engine, constant),
        Expression::Int { i } => Value::Int(i),
        Expression::ConstantString { value } => Value::String(value.into()),
        _ => panic!("unhandled expression: {:?}", expression)
    }
}

fn eval_infix_expression(engine: &mut Engine, expression: Expression) -> Value {
    let (lhs, op, rhs) = match expression {
        Expression::Infix { lhs, op, rhs } => (lhs, op, rhs),
        _ => unreachable!(),
    };

    let lhs = eval_expression(engine, *lhs);
    let rhs = eval_expression(engine, *rhs);

    match op {
        InfixOp::Add => lhs + rhs,
        InfixOp::Sub => lhs - rhs,
        InfixOp::LessThan => Value::Bool(lhs < rhs),
        InfixOp::Concat => {
            match (lhs, rhs) {
                (Value::String(a), Value::String(b)) => {
                    let mut s = String::with_capacity(a.len() + b.len());
                    s.push_str(&a);
                    s.push_str(&b);
                    Value::String(s)
                },
                _ => todo!()
            }
        },
        _ => todo!("infix: {:?}", op)
    }
}

fn eval_call_expression(engine: &mut Engine, expression: Expression) -> Value {
    let (target, args) = match expression {
        Expression::Call { target, args } => (target, args),
        _ => unreachable!()
    };

    let target: String = match *target {
        Expression::Identifier { name } => name.into(),
        _ => unreachable!(),
    };

    if !engine.function_table.contains_key(&target) {
        panic!("undefined function: {}", target);
    }

    let mut arg_values = Vec::new();
    for arg in args {
        let value = eval_expression(engine, arg.value);
        arg_values.push(value);
    }

    let func = engine.function_table.get(&target).unwrap();
    let mut environment = Environment::new();

    for (i, param) in func.params.clone().into_iter().enumerate() {
        let name: String = match param.name {
            Expression::Variable { name } => name.into(),
            _ => todo!()
        };

        environment.set(&name, arg_values.get(i).unwrap().clone());
    }

    engine.scopes.push(environment);

    let mut return_value = Value::Null;

    for statement in func.body.clone() {
        match eval_statement(engine, statement) {
            Err(Escape::Return(value)) => {
                return_value = value.clone();
                break;
            },
            _ => {},
        }
    }

    engine.scopes.pop();

    return_value
}

fn eval_variable_expression(engine: &mut Engine, expression: Expression) -> Value {
    let name: String = match expression {
        Expression::Variable { name } => name.into(),
        _ => unreachable!(),
    };

    if let Some(scope) = engine.scopes.last() {
        if let Some(value) = scope.get(&name) {
            return value.clone();
        } else {
            panic!("undefined variable: {}", name);
        }
    } else {
        if let Some(value) = engine.global_environment.get(&name) {
            return value.clone();
        } else {
            panic!("undefined variable: {}", name);
        }
    }
}

fn eval_cast_expression(engine: &mut Engine, expression: Expression) -> Value {
    let (kind, value) = match expression {
        Expression::Cast { kind, value } => (kind, value),
        _ => unreachable!()
    };

    let value = eval_expression(engine, *value);

    match (kind, &value) {
        (CastKind::String, Value::Int(i)) => Value::String(i.to_string()),
        _ => value,
    }
}

fn eval_magic_const(engine: &mut Engine, constant: MagicConst) -> Value {
    match constant {
        MagicConst::Dir => {
            // FIXME: Sort this nasty code out.
            Value::String(engine.filename.parent().unwrap().to_str().unwrap().to_string())
        },
        _ => todo!()
    }
}
use crate::{Statement, Program};

pub trait Visitor {
    fn visit(&mut self, statement: &Statement);

    fn traverse(&mut self, ast: Program) {
        for statement in &ast {
            self.visit(statement);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Parser, Program, Visitor};
    use trunk_lexer::Lexer;

    struct CountVisitor {
        count: usize,
    }

    impl Visitor for CountVisitor {
        fn visit(&mut self, statement: &crate::Statement) {
            self.count += 1;
        }
    }

    #[test]
    fn it_can_walk_an_ast() {
        let ast = get_ast("<?php foo();");
        let mut visitor = CountVisitor { count: 0 };
        visitor.traverse(ast);
        assert_eq!(visitor.count, 1);
    }

    fn get_ast(source: &str) -> Program {
        let mut lexer = Lexer::new(None);
        let tokens = lexer.tokenize(source).unwrap();

        let mut parser = Parser::new(None);
        let ast = parser.parse(tokens).unwrap();

        ast
    }
}
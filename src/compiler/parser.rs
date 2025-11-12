use crate::{
    compiler::{SourceLoc, ast::*, token::*, tokeniser::*},
    types::Type,
};

#[derive(Debug)]
pub enum ParserErrorReason {
    GenericError, // todo(caleb): Remove me!
    ExpectedToken {
        expected: TokenKind,
        token: Option<Token>,
    },
    ExpectedTopLevelDefinition,
    UnknownBinaryOperator,
    UnexpectedEOF,
    ExpectedExpression,
}

#[derive(Debug)]
pub struct ParserError {
    loc: SourceLoc,
    reason: ParserErrorReason,
}

type ParserResult<X> = Result<X, ParserError>;

pub struct Parser<'a> {
    tokeniser: Tokeniser<'a>,
    mode: TokeniserMode,
}

impl<'a> Parser<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            tokeniser: Tokeniser::new(contents),
            mode: TokeniserMode::Regex,
        }
    }

    fn error<T>(&mut self, reason: ParserErrorReason) -> ParserResult<T> {
        Err(ParserError {
            loc: SourceLoc {
                line: self.tokeniser.line_no(),
                col: self.tokeniser.col_no(),
                len: 0,
            },
            reason,
        })
    }

    fn expr(&mut self, kind: ExprKind, loc: SourceLoc) -> Expr {
        Expr {
            kind,
            loc,
            typ: Box::new(Type::default()),
        }
    }

    fn test(&mut self, expected: TokenKind) -> bool {
        let token = self.tokeniser.peek(self.mode);
        if let Some(token) = token.clone()
            && token.kind == expected
        {
            return true;
        } else {
            return false;
        }
    }

    fn expect(&mut self, expected: TokenKind) -> ParserResult<Token> {
        let token = self.tokeniser.next(self.mode);
        if let Some(token) = token.clone()
            && token.kind == expected
        {
            return Ok(token);
        } else {
            return self.error(ParserErrorReason::ExpectedToken { expected, token });
        }
    }

    fn next(&mut self) -> ParserResult<Token> {
        let token = self.tokeniser.next(self.mode);
        if let Some(token) = token.clone() {
            return Ok(token);
        } else {
            return self.error(ParserErrorReason::UnexpectedEOF);
        }
    }

    fn source_loc(&self) -> SourceLoc {
        SourceLoc {
            line: self.tokeniser.line_no(),
            col: self.tokeniser.col_no(),
            len: 0,
        }
    }

    pub fn parse_module(&mut self) -> ParserResult<Box<Module>> {
        let mut module = Box::new(Module::default());
        while let Some(next) = self.tokeniser.peek(TokeniserMode::Regex) {
            match next.kind {
                TokenKind::Keyword(Keywords::Func) => {
                    let func = self.parse_function()?;
                    module.functions.push(func);
                }
                _ => return self.error(ParserErrorReason::ExpectedTopLevelDefinition),
            }
        }
        Ok(module)
    }

    pub fn parse_function(&mut self) -> ParserResult<Box<Func>> {
        let mut function = Box::new(Func::default());
        function.loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Func))?;
        let id = self.expect(TokenKind::Identifier)?;
        function.signature.id = id.get_string();
        self.expect(TokenKind::Punctuation(Punctuation::LeftParenthesis))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightParenthesis)) {
            let param_id = self.expect(TokenKind::Identifier)?;
            let param_id = param_id.get_string();
            self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
            let param_type = self.parse_type()?;
            function.signature.params.push(Param {
                id: param_id,
                type_annotation: param_type,
            });
            if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
            } else {
                break;
            }
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
        if self.test(TokenKind::Punctuation(Punctuation::Colon)) {
            self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
            let return_type = self.parse_type()?;
            function.signature.return_type = Some(return_type);
        }
        let body = self.parse_block_statement()?;
        function.body = body;
        Ok(function)
    }

    fn parse_statement(&mut self) -> ParserResult<Stmt> {
        if self.test(TokenKind::Keyword(Keywords::If)) {
            let if_ = self.parse_if()?;
            return Ok(Stmt::If(if_));
        } else if self.test(TokenKind::Keyword(Keywords::Const)) {
            let var_decl = self.parse_var_decl_statement()?;
            return Ok(Stmt::VarDecl(var_decl));
        } else if self.test(TokenKind::Keyword(Keywords::Let)) {
            let var_decl = self.parse_var_decl_statement()?;
            return Ok(Stmt::VarDecl(var_decl));
        } else if self.test(TokenKind::Keyword(Keywords::Return)) {
            let return_stmt = self.parse_return_statement()?;
            return Ok(Stmt::Return(return_stmt));
        } else if self.test(TokenKind::Keyword(Keywords::For)) {
            todo!()
        } else if self.test(TokenKind::Keyword(Keywords::While)) {
            todo!()
        } else if self.test(TokenKind::Punctuation(Punctuation::LeftBrace)) {
            let block = self.parse_block_statement()?;
            return Ok(Stmt::Block(block));
        } else {
            let loc = self.source_loc();
            let expr = self.parse_expression()?;
            self.expect(TokenKind::Punctuation(Punctuation::SemiColon))?;
            return Ok(Stmt::ExprStmt(Box::new(ExprStmt { loc, expr })));
        }
    }

    fn parse_if(&mut self) -> ParserResult<Box<IfStmt>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::If))?;
        let test = self.parse_expression()?;
        let consequent = self.parse_statement()?;
        let alternate = if self.test(TokenKind::Keyword(Keywords::Else)) {
            self.expect(TokenKind::Keyword(Keywords::Else))?;
            Some(self.parse_statement()?)
        } else {
            None
        };
        Ok(Box::new(IfStmt {
            loc,
            test,
            consequent,
            alternate,
        }))
    }

    fn parse_while(&mut self) -> ParserResult<Box<WhileStmt>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::While))?;
        let condition = self.parse_expression()?;
        let consequent = self.parse_statement()?;
        Ok(Box::new(WhileStmt {
            loc,
            condition,
            consequent,
        }))
    }

    fn parse_block_statement(&mut self) -> ParserResult<Box<BlockStmt>> {
        let loc = self.source_loc();
        let mut stmts = Vec::new();
        self.expect(TokenKind::Punctuation(Punctuation::LeftBrace))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
            let stmt = self.parse_statement()?;
            stmts.push(stmt);
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
        Ok(Box::new(BlockStmt { loc, stmts }))
    }

    fn parse_var_decl_statement(&mut self) -> ParserResult<Box<VarDeclStmt>> {
        let loc = self.source_loc();
        let is_const = if self.test(TokenKind::Keyword(Keywords::Const)) {
            self.expect(TokenKind::Keyword(Keywords::Const))?;
            true
        } else {
            self.expect(TokenKind::Keyword(Keywords::Let))?;
            false
        };
        let id_token = self.expect(TokenKind::Identifier)?;
        let id = id_token.get_string();
        let type_annotation = if self.test(TokenKind::Punctuation(Punctuation::Colon)) {
            self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Punctuation(Punctuation::Equals))?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Punctuation(Punctuation::SemiColon))?;
        Ok(Box::new(VarDeclStmt {
            loc,
            is_const,
            id,
            type_annotation,
            value,
        }))
    }

    fn parse_return_statement(&mut self) -> ParserResult<Box<ReturnStmt>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Return))?;
        let value = if !self.test(TokenKind::Punctuation(Punctuation::SemiColon)) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(TokenKind::Punctuation(Punctuation::SemiColon))?;
        Ok(Box::new(ReturnStmt { loc, value }))
    }

    /////////////////////////////
    /// EXPRESSIONS
    /////////////////////////////

    fn parse_prec(token: Token) -> u8 {
        match token.kind {
            TokenKind::Punctuation(Punctuation::EqualsEquals)
            | TokenKind::Punctuation(Punctuation::ExclamationEquals)
            | TokenKind::Punctuation(Punctuation::LeftAngle)
            | TokenKind::Punctuation(Punctuation::RightAngle)
            | TokenKind::Punctuation(Punctuation::LeftAngleEquals)
            | TokenKind::Punctuation(Punctuation::RightAngleEquals) => 1,
            TokenKind::Punctuation(Punctuation::Plus)
            | TokenKind::Punctuation(Punctuation::Minus) => 2,
            TokenKind::Punctuation(Punctuation::Multiply)
            | TokenKind::Punctuation(Punctuation::ForwardSlash) => 3,
            _ => 0,
        }
    }

    fn parse_binary_op_kind(&mut self, token: Token) -> ParserResult<BinaryExprKind> {
        match token.kind {
            TokenKind::Punctuation(Punctuation::Plus) => Ok(BinaryExprKind::Add),
            TokenKind::Punctuation(Punctuation::Minus) => Ok(BinaryExprKind::Subtract),
            TokenKind::Punctuation(Punctuation::Multiply) => Ok(BinaryExprKind::Multiply),
            TokenKind::Punctuation(Punctuation::ForwardSlash) => Ok(BinaryExprKind::Divide),
            TokenKind::Punctuation(Punctuation::EqualsEquals) => Ok(BinaryExprKind::Equal),
            TokenKind::Punctuation(Punctuation::ExclamationEquals) => Ok(BinaryExprKind::NotEqual),
            TokenKind::Punctuation(Punctuation::LeftAngle) => Ok(BinaryExprKind::LessThan),
            TokenKind::Punctuation(Punctuation::RightAngle) => Ok(BinaryExprKind::GreaterThan),
            TokenKind::Punctuation(Punctuation::LeftAngleEquals) => {
                Ok(BinaryExprKind::LessThanEqual)
            }
            TokenKind::Punctuation(Punctuation::RightAngleEquals) => {
                Ok(BinaryExprKind::GreaterThanEqual)
            }
            _ => self.error(ParserErrorReason::UnknownBinaryOperator),
        }
    }

    fn parse_bin_expr(&mut self, prec: u8) -> ParserResult<Expr> {
        let mut lhs = self.parse_left_hand_side_expr()?;
        loop {
            let token = match self.tokeniser.peek(self.mode) {
                Some(token) => token,
                None => break,
            };
            let new_prec = Self::parse_prec(token.clone());
            if new_prec <= prec {
                break;
            }
            self.tokeniser.next(self.mode);
            let rhs = self.parse_bin_expr(new_prec)?;
            let kind = self.parse_binary_op_kind(token.clone())?;
            let expr = ExprKind::BinaryExpr(Box::new(BinaryExpr { lhs, rhs, kind }));
            lhs = self.expr(expr, token.loc);
        }
        Ok(lhs)
    }

    fn parse_left_hand_side_expr(&mut self) -> ParserResult<Expr> {
        let loc = self.source_loc();
        let mut expr = self.parse_primary_expr()?;
        loop {
            if self.test(TokenKind::Punctuation(Punctuation::LeftBracket)) {
                self.next()?;
                let index = self.parse_expression()?;
                expr = self.expr(
                    ExprKind::Lookup(Box::new(Lookup { value: expr, index })),
                    loc,
                );
                self.expect(TokenKind::Punctuation(Punctuation::RightBracket))?;
            } else if self.test(TokenKind::Punctuation(Punctuation::Equals)) {
                self.next()?;
                let value = self.parse_expression()?;
                expr = self.expr(
                    ExprKind::Assign(Box::new(Assign {
                        destination: expr,
                        value,
                    })),
                    loc,
                );
            } else if self.test(TokenKind::Punctuation(Punctuation::LeftParenthesis)) {
                self.next()?;
                let mut parameters = Vec::new();
                while !self.test(TokenKind::Punctuation(Punctuation::RightParenthesis)) {
                    parameters.push(self.parse_expression()?);
                    if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                        self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
                expr = self.expr(
                    ExprKind::Call(Box::new(Call { 
                        function: expr, 
                        parameters
                    })),
                    loc,
                );
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_primary_expr(&mut self) -> ParserResult<Expr> {
        if self.test(TokenKind::Punctuation(Punctuation::LeftParenthesis)) {
            self.next()?;
            let expr = self.parse_expression()?;
            self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
            return Ok(expr);
        } else if self.test(TokenKind::IntegerLiteral) {
            let token = self.next()?;
            let value = token.get_int();
            return Ok(Expr {
                kind: ExprKind::Integer(Box::new(Integer { value })),
                loc: token.loc,
                typ: Box::new(Type::Integer),
            });
        } else if self.test(TokenKind::NumberLiteral) {
            let token = self.next()?;
            let value = token.get_float();
            return Ok(Expr {
                kind: ExprKind::Number(Box::new(Number { value })),
                loc: token.loc,
                typ: Box::new(Type::Number),
            });
        } else if self.test(TokenKind::StringLiteral) {
            let token = self.next()?;
            let value = token.get_string();
            return Ok(Expr {
                kind: ExprKind::StringLiteral(Box::new(StringLiteral { value })),
                loc: token.loc,
                typ: Box::new(Type::String),
            });
        } else if self.test(TokenKind::Identifier) {
            let token = self.next()?;
            let id = token.get_string();
            return Ok(self.expr(ExprKind::Identifier(Box::new(Identifier { id })), token.loc));
        } else if self.test(TokenKind::Keyword(Keywords::True)) {
            let token = self.next()?;
            return Ok(Expr {
                kind: ExprKind::Boolean(Box::new(Bool { value: true })),
                loc: token.loc,
                typ: Box::new(Type::Bool),
            });
        } else if self.test(TokenKind::Keyword(Keywords::False)) {
            let token = self.next()?;
            return Ok(Expr {
                kind: ExprKind::Boolean(Box::new(Bool { value: false })),
                loc: token.loc,
                typ: Box::new(Type::Bool),
            });
        } else {
            return self.error(ParserErrorReason::ExpectedExpression);
        }
    }

    fn parse_expression(&mut self) -> ParserResult<Expr> {
        self.parse_bin_expr(0)
    }

    /////////////////////////////
    /// TYPES
    /////////////////////////////

    fn parse_type(&mut self) -> ParserResult<Box<Type>> {
        if self.test(TokenKind::Punctuation(Punctuation::LeftBracket)) {
            self.tokeniser.next(self.mode);
            self.expect(TokenKind::Punctuation(Punctuation::RightBracket))?;
            let element_type = self.parse_type()?;
            return Ok(Box::new(Type::Array(element_type)));
        }

        let string = self.expect(TokenKind::Identifier)?.get_string();

        if string == "string" {
            return Ok(Box::new(Type::String));
        } else if string == "bool" {
            return Ok(Box::new(Type::Bool));
        } else if string == "int" {
            return Ok(Box::new(Type::Integer));
        } else if string == "number" {
            return Ok(Box::new(Type::Number));
        } else {
            return Ok(Box::new(Type::Identifier(string)));
        }
    }
}

mod tests {
    #[test]
    fn test_parse_module() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("func test() {}");
        let module = parser.parse_module().unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].signature.id, "test");
    }

    #[test]
    fn test_parse_function() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("func test(param1: string, param2: int) {}");
        let func = parser.parse_function().unwrap();
        assert_eq!(func.signature.id, "test");
        assert_eq!(func.signature.params.len(), 2);
        assert_eq!(func.signature.params[0].id, "param1");
        assert_eq!(*func.signature.params[0].type_annotation, crate::types::Type::String);
        assert_eq!(func.signature.params[1].id, "param2");
        assert_eq!(*func.signature.params[1].type_annotation, crate::types::Type::Integer);
    }

    #[test]
    fn test_parse_if() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("if x < 10 {} else {}");
        let if_ = parser.parse_if().unwrap();
        //assert_eq!(if_.consequent.stmts.len(), 0);
        assert!(if_.alternate.is_some());
    }

    #[test]
    fn test_parse_while() {
        use crate::compiler::parser::Parser;
        let mut parser = Parser::new("while x < 10 {}");
        let _while = parser.parse_while().unwrap();
        //assert_eq!(while_.consequent.stmts.len(), 0);
    }

    #[test]
    fn test_parse_block() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("{ if x < 10 {} }");
        let block = parser.parse_block_statement().unwrap();
        assert_eq!(block.stmts.len(), 1);
        if let crate::compiler::ast::Stmt::If(_) = &block.stmts[0] {
            // pass
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_var_decl() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("let x: int = 10;");
        let var_decl = parser.parse_var_decl_statement().unwrap();
        assert_eq!(var_decl.id, "x");
        assert_eq!(
            *var_decl.type_annotation.unwrap(),
            crate::types::Type::Integer
        );
        if let crate::compiler::ast::ExprKind::Integer(int) = &var_decl.value.kind {
            assert_eq!(int.value, 10);
        } else {
            panic!("Expected integer literal");
        }
    }

    #[test]
    fn test_parse_return() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("return 42;");
        let return_stmt = parser.parse_return_statement().unwrap();
        if let Some(value) = &return_stmt.value {
            if let crate::compiler::ast::ExprKind::Integer(int) = &value.kind {
                assert_eq!(int.value, 42);
            } else {
                panic!("Expected integer literal");
            }
        } else {
            panic!("Expected return value");
        }
    }

    #[test]
    fn test_parse_type() {
        use crate::compiler::parser::Parser;
        use crate::types::Type;
        let mut parser = Parser::new("string bool int number []string myStruct");
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::String);
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::Bool);
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::Integer);
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::Number);
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::Array(Box::new(Type::String)));
        let ty = parser.parse_type().unwrap();
        assert_eq!(*ty, Type::Identifier("myStruct".into()));
    }
}

use crate::{
    compiler::{SourceLoc, ast::*, token::*, tokeniser::*},
    types::{self},
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
    ExpectedPattern,
    ExpectedTemplate,
}

#[derive(Debug)]
pub struct ParserError {
    loc: SourceLoc,
    reason: ParserErrorReason,
}

type ParserResult<X> = Result<X, ParserError>;

pub struct Parser<'a> {
    package: &'a str,
    tokeniser: Tokeniser<'a>,
    mode: TokeniserMode,
    nest_level: i32,
}

impl<'a> Parser<'a> {
    pub fn new(package: &'a str, contents: &'a str) -> Self {
        Self {
            package,
            tokeniser: Tokeniser::new(contents),
            mode: TokeniserMode::Regex,
            nest_level: 0,
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
            typ: types::Type::default(),
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

    fn skip(&mut self) {
        _ = self.tokeniser.next(self.mode);
    }

    fn source_loc(&self) -> SourceLoc {
        SourceLoc {
            line: self.tokeniser.line_no(),
            col: self.tokeniser.col_no(),
            len: 0,
        }
    }

    pub fn parse_file(&mut self) -> ParserResult<Box<File>> {
        let mut file = Box::new(File::default());

        while let Some(next) = self.tokeniser.peek(TokeniserMode::Regex) {
            match next.kind {
                TokenKind::Keyword(Keywords::Import) => {
                    file.imports.push(self.parse_import()?);
                }
                TokenKind::Keyword(Keywords::Func) => {
                    let func = self.parse_function()?;
                    file.functions.push(func);
                }
                TokenKind::Keyword(Keywords::Struct) => {
                    let struct_ = self.parse_struct()?;
                    file.structs.push(struct_);
                }
                TokenKind::Keyword(Keywords::Enum) => {
                    let enum_ = self.parse_enum()?;
                    file.enums.push(enum_);
                }
                TokenKind::Keyword(Keywords::Interface) => {
                    let interface = self.parse_interface()?;
                    file.interfaces.push(interface);
                }
                _ => return self.error(ParserErrorReason::ExpectedTopLevelDefinition),
            }
        }
        Ok(file)
    }

    fn parse_import(&mut self) -> ParserResult<Import> {
        self.expect(TokenKind::Keyword(Keywords::Import))?;
        let path = self.expect(TokenKind::StringLiteral)?.get_string();
        self.expect(TokenKind::Punctuation(Punctuation::SemiColon))?;

        if let Some((package, file)) = path.split_once(':') {
            Ok(Import {
                package: package.to_string(),
                file: file.to_string(),
            })
        } else {
            Ok(Import {
                package: self.package.into(),
                file: path,
            })
        }
    }

    pub fn parse_interface(&mut self) -> ParserResult<Box<Interface>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Interface))?;
        let id = self.expect(TokenKind::Identifier)?;
        let mut interface = Box::new(Interface {
            loc,
            id: id.get_string(),
            methods: Vec::new(),
            typ: types::Type::default(),
        });
        self.expect(TokenKind::Punctuation(Punctuation::LeftBrace))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
            let method = self.parse_function_signature()?;
            self.expect(TokenKind::Punctuation(Punctuation::SemiColon))?;
            interface.methods.push(method);
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
        Ok(interface)
    }

    pub fn parse_enum(&mut self) -> ParserResult<Box<Enum>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Enum))?;
        let id = self.expect(TokenKind::Identifier)?;
        let mut enum_ = Box::new(Enum {
            loc,
            id: id.get_string(),
            variants: Vec::new(),
            typ: types::Type::default(),
        });
        self.expect(TokenKind::Punctuation(Punctuation::LeftBrace))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
            let variant_loc = self.source_loc();
            let variant_id_token = self.expect(TokenKind::Identifier)?;
            let variant_id = variant_id_token.get_string();
            let mut variant_types = Vec::new();
            if self.test(TokenKind::Punctuation(Punctuation::LeftParenthesis)) {
                _ = self.next();
                while !self.test(TokenKind::Punctuation(Punctuation::RightParenthesis)) {
                    let ty = self.parse_type()?;
                    variant_types.push(ty);
                    if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                        self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
            }
            enum_.variants.push(EnumVariant {
                loc: variant_loc,
                id: variant_id,
                variant_types,
            });
            if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                _ = self.next();
            } else {
                break;
            }
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
        Ok(enum_)
    }

    pub fn parse_struct(&mut self) -> ParserResult<Box<Struct>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Struct))?;
        let id = self.expect(TokenKind::Identifier)?;
        let mut struct_ = Box::new(Struct {
            loc,
            id: id.get_string(),
            fields: Vec::new(),
            functions: Vec::new(),
            typ: types::Type::default(),
        });
        self.expect(TokenKind::Punctuation(Punctuation::LeftBrace))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
            let loc = self.source_loc();
            if self.test(TokenKind::Keyword(Keywords::Func)) {
                let func = self.parse_function()?;
                struct_.functions.push(func);
            } else {
                let field_id_token = self.expect(TokenKind::Identifier)?;
                let field_id = field_id_token.get_string();
                self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
                let field_type = self.parse_type()?;
                struct_.fields.push(StructField {
                    loc,
                    id: field_id,
                    type_annotation: field_type,
                });
                if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                    self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
        Ok(struct_)
    }

    pub fn parse_function_signature(&mut self) -> ParserResult<FuncSignature> {
        let mut signature = FuncSignature::default();

        self.expect(TokenKind::Keyword(Keywords::Func))?;
        let id = self.expect(TokenKind::Identifier)?;
        signature.id = id.get_string();
        self.expect(TokenKind::Punctuation(Punctuation::LeftParenthesis))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightParenthesis)) {
            let param_id = self.expect(TokenKind::Identifier)?;
            let param_id = param_id.get_string();
            self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
            let param_type = self.parse_type()?;
            signature.params.push(Param {
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
            signature.return_type = Some(return_type);
        }
        Ok(signature)
    }

    pub fn parse_function(&mut self) -> ParserResult<Box<Func>> {
        let mut function = Box::new(Func::default());
        function.signature = self.parse_function_signature()?;
        function.loc = self.source_loc();
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
            let while_ = self.parse_while()?;
            return Ok(Stmt::While(while_));
        } else if self.test(TokenKind::Keyword(Keywords::Switch)) {
            let switch = self.parse_switch()?;
            return Ok(Stmt::Switch(switch));
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
        let mut not = false;
        if self.test(TokenKind::Keyword(Keywords::Not)) {
            self.skip();
            not = true;
        }
        let old_nest_level = self.nest_level;
        self.nest_level = -1;
        let test = self.parse_expression()?;
        self.nest_level = old_nest_level;
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
            not,
        }))
    }

    fn parse_while(&mut self) -> ParserResult<Box<WhileStmt>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::While))?;
        let old_nest_level = self.nest_level;
        self.nest_level = -1;
        let condition = self.parse_expression()?;
        self.nest_level = old_nest_level;
        let consequent = self.parse_statement()?;
        Ok(Box::new(WhileStmt {
            loc,
            condition,
            consequent,
        }))
    }

    fn parse_switch(&mut self) -> ParserResult<Box<SwitchStmt>> {
        let loc = self.source_loc();
        self.expect(TokenKind::Keyword(Keywords::Switch))?;
        let old_nest_level = self.nest_level;
        self.nest_level = -1;
        let value = self.parse_expression()?;
        self.nest_level = old_nest_level;

        let mut cases = Vec::new();
        self.expect(TokenKind::Punctuation(Punctuation::LeftBrace))?;
        while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
            let block = self.parse_block_statement()?;
            if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                self.next()?;
            }
            cases.push(CaseStmt {
                pattern,
                block,
                case_idx: 0,
            });
        }
        self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;

        Ok(Box::new(SwitchStmt { loc, value, cases }))
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
            TokenKind::Punctuation(Punctuation::BarBar)
            | TokenKind::Punctuation(Punctuation::AndAnd) => 1,
            TokenKind::Punctuation(Punctuation::EqualsEquals)
            | TokenKind::Punctuation(Punctuation::ExclamationEquals)
            | TokenKind::Punctuation(Punctuation::LeftAngle)
            | TokenKind::Punctuation(Punctuation::RightAngle)
            | TokenKind::Punctuation(Punctuation::LeftAngleEquals)
            | TokenKind::Punctuation(Punctuation::RightAngleEquals) => 2,
            TokenKind::Punctuation(Punctuation::Plus)
            | TokenKind::Punctuation(Punctuation::Minus) => 3,
            TokenKind::Punctuation(Punctuation::Multiply)
            | TokenKind::Punctuation(Punctuation::ForwardSlash) => 4,
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
            TokenKind::Punctuation(Punctuation::AndAnd) => Ok(BinaryExprKind::LogicalAnd),
            TokenKind::Punctuation(Punctuation::BarBar) => Ok(BinaryExprKind::LogicalOr),
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
                if self.test(TokenKind::Punctuation(Punctuation::Colon)) {
                    self.next()?;
                    let index_end = if self.test(TokenKind::Punctuation(Punctuation::RightBracket)) {
                        None
                    } else {
                        Some(self.parse_expression()?)
                    };
                    expr = self.expr(
                        ExprKind::Subscript(Box::new(Subscript { value: expr, index: None, is_slice: true, index_end })),
                        loc,
                    );
                    self.expect(TokenKind::Punctuation(Punctuation::RightBracket))?;
                    continue;
                } else {
                    let index = self.parse_expression()?;
                    let mut index_end = None;
                    let mut is_slice = false;
                    if self.test(TokenKind::Punctuation(Punctuation::Colon)) {
                        self.next()?;
                        is_slice = true;
                        index_end = if self.test(TokenKind::Punctuation(Punctuation::RightBracket)) {
                            None
                        } else {
                            Some(self.parse_expression()?)
                        };
                    }
                    expr = self.expr(
                        ExprKind::Subscript(Box::new(Subscript { value: expr, index: Some(index), is_slice, index_end })),
                        loc,
                    );
                }
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
            } else if self.test(TokenKind::Punctuation(Punctuation::Dot)) {
                self.next()?;
                let id_token = self.expect(TokenKind::Identifier)?;
                let id = id_token.get_string();
                expr = self.expr(
                    ExprKind::Selector(Box::new(Selector {
                        value: expr,
                        selector: Identifier { id },
                        idx: 0,
                        enum_idx: None,
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
                let loc = expr.loc;
                expr = self.expr(
                    ExprKind::Call(Box::new(Call {
                        function: expr,
                        parameters,
                        symbol_name: None,
                        enum_idx: None,
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
            self.nest_level += 1;
            let expr = self.parse_expression()?;
            self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
            self.nest_level -= 1;
            return Ok(expr);
        } else if self.test(TokenKind::IntegerLiteral) {
            let token = self.next()?;
            let value = token.get_int();
            return Ok(Expr {
                kind: ExprKind::Integer(Box::new(Integer { value })),
                loc: token.loc,
                typ: types::integer(),
            });
        } else if self.test(TokenKind::NumberLiteral) {
            let token = self.next()?;
            let value = token.get_float();
            return Ok(Expr {
                kind: ExprKind::Number(Box::new(Number { value })),
                loc: token.loc,
                typ: types::number(),
            });
        } else if self.test(TokenKind::StringLiteral) {
            let token = self.next()?;
            let value = token.get_string();
            return Ok(Expr {
                kind: ExprKind::StringLiteral(Box::new(StringLiteral { value })),
                loc: token.loc,
                typ: types::string(),
            });
        } else if self.test(TokenKind::TemplateHead) {
            let mut literals = Vec::new();
            let mut expressions = Vec::new();

            let old_mode = self.mode;
            self.mode = TokeniserMode::TemplateTail;

            let token = self.next()?;
            literals.push(token.get_string());

            loop {
                let expr = self.parse_expression()?;
                expressions.push(expr);
                if self.test(TokenKind::TemplateTail) {
                    let token = self.next()?;
                    literals.push(token.get_string());
                    break;
                } else if self.test(TokenKind::TemplateMiddle) {
                    let token = self.next()?;
                    literals.push(token.get_string());
                } else {
                    return self.error(ParserErrorReason::ExpectedTemplate);
                }
            }

            self.mode = old_mode;

            return Ok(Expr {
                kind: ExprKind::Template(Box::new(Template {
                    literals,
                    expressions,
                })),
                loc: token.loc,
                typ: types::string(),
            });
        } else if self.test(TokenKind::Identifier) {
            let token = self.next()?;
            let id = token.get_string();

            // test for object literal
            if self.nest_level >= 0 && self.test(TokenKind::Punctuation(Punctuation::LeftBrace)) {
                self.next()?;
                let mut fields = Vec::new();
                while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
                    let field_loc = self.source_loc();
                    let field_id_token = self.expect(TokenKind::Identifier)?;
                    let field_id = field_id_token.get_string();
                    self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
                    let field_value = self.parse_expression()?;
                    fields.push(ObjectLiteralField {
                        loc: field_loc,
                        id: field_id,
                        value: field_value,
                    });
                    if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                        self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
                return Ok(self.expr(
                    ExprKind::ObjectLiteral(Box::new(ObjectLiteral {
                        id: Some(Identifier { id }),
                        fields,
                    })),
                    token.loc,
                ));
            } else {
                return Ok(self.expr(ExprKind::Identifier(Box::new(Identifier { id })), token.loc));
            }
        } else if self.test(TokenKind::Keyword(Keywords::True)) {
            let token = self.next()?;
            return Ok(Expr {
                kind: ExprKind::Boolean(Box::new(Bool { value: true })),
                loc: token.loc,
                typ: types::bool(),
            });
        } else if self.test(TokenKind::Keyword(Keywords::False)) {
            let token = self.next()?;
            return Ok(Expr {
                kind: ExprKind::Boolean(Box::new(Bool { value: false })),
                loc: token.loc,
                typ: types::bool(),
            });
        } else if self.test(TokenKind::Keyword(Keywords::_Self)) {
            let token = self.next()?;
            return Ok(Expr {
                kind: ExprKind::_Self,
                loc: token.loc,
                typ: types::bool(),
            });
        } else if self.test(TokenKind::Punctuation(Punctuation::LeftBracket)) {
            let token = self.next()?;
            let mut literals = Vec::new();
            while !self.test(TokenKind::Punctuation(Punctuation::RightBracket)) {
                let literal = self.parse_expression()?;
                literals.push(literal);
                if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                    self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                } else {
                    break;
                }
            }
            self.expect(TokenKind::Punctuation(Punctuation::RightBracket))?;
            return Ok(self.expr(
                ExprKind::ArrayLiteral(Box::new(ArrayLiteral { literals })),
                token.loc,
            ));
        }
        // else if self.test(TokenKind::Punctuation(Punctuation::LeftBrace)) {
        //     let token = self.next()?;
        //     let mut fields = Vec::new();
        //     while !self.test(TokenKind::Punctuation(Punctuation::RightBrace)) {
        //         let field_loc = self.source_loc();
        //         let field_id_token = self.expect(TokenKind::Identifier)?;
        //         let field_id = field_id_token.get_string();
        //         self.expect(TokenKind::Punctuation(Punctuation::Colon))?;
        //         let field_value = self.parse_expression()?;
        //         fields.push(ObjectLiteralField {

        //             loc: field_loc,
        //             id: field_id,
        //             value: field_value,
        //         });
        //         if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
        //             self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
        //         } else {
        //             break;
        //         }
        //     }
        //     self.expect(TokenKind::Punctuation(Punctuation::RightBrace))?;
        //     return Ok(self.expr(
        //         ExprKind::ObjectLiteral(Box::new(ObjectLiteral {
        //             id: None,
        //             fields,
        //         })),
        //         token.loc,
        //     ));
        // }
        else {
            return self.error(ParserErrorReason::ExpectedExpression);
        }
    }

    fn parse_expression(&mut self) -> ParserResult<Expr> {
        let old_tokeniser_mode = self.mode;
        self.mode = TokeniserMode::Div;
        let expr = self.parse_bin_expr(0)?;
        self.mode = old_tokeniser_mode;
        Ok(expr)
    }

    /////////////////////////////
    /// PATTERNS
    /////////////////////////////

    fn parse_pattern(&mut self) -> ParserResult<Box<Pattern>> {
        let loc = self.source_loc();

        let kind = if self.test(TokenKind::Punctuation(Punctuation::Underscore)) {
            self.next()?;
            PatternKind::CatchAll
        } else if self.test(TokenKind::IntegerLiteral) {
            let token = self.next()?;
            let value = token.get_int();
            if self.test(TokenKind::Punctuation(Punctuation::DotDot)) {
                self.next()?;
                let end_token = self.expect(TokenKind::IntegerLiteral)?;
                let end_value = end_token.get_int();
                PatternKind::IntegerRange(value, end_value)
            } else {
                PatternKind::Integer(value)
            }
        } else if self.test(TokenKind::StringLiteral) {
            let token = self.next()?;
            let value = token.get_string();
            PatternKind::String(value)
        } else if self.test(TokenKind::Punctuation(Punctuation::Dot)) {
            self.expect(TokenKind::Punctuation(Punctuation::Dot))?;
            let id_token = self.expect(TokenKind::Identifier)?;
            let id = id_token.get_string();

            let mut values = Vec::new();
            if self.test(TokenKind::Punctuation(Punctuation::LeftParenthesis)) {
                self.next()?;
                while !self.test(TokenKind::Punctuation(Punctuation::RightParenthesis)) {
                    let value_token = self.expect(TokenKind::Identifier)?;
                    values.push((value_token.get_string(), types::bad()));
                    if self.test(TokenKind::Punctuation(Punctuation::Comma)) {
                        self.next()?;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::Punctuation(Punctuation::RightParenthesis))?;
            }

            PatternKind::EnumVariant { id, values }
        } else {
            return self.error(ParserErrorReason::ExpectedPattern);
        };

        Ok(Box::new(Pattern { loc, kind }))
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
        } else if string == "byte" {
            return Ok(Box::new(Type::Byte));
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

        let mut parser = Parser::new("testing", "func test() {} struct MyStruct {}");
        let file = parser.parse_file().unwrap();
        assert_eq!(file.functions.len(), 1);
        assert_eq!(file.functions[0].signature.id, "test");
        assert_eq!(file.structs.len(), 1);
        assert_eq!(file.structs[0].id, "MyStruct");
    }

    #[test]
    fn test_parse_imports() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new(
            "testing",
            "import \"std:math/core.luna\"; import \"utils.luna\"; func main() {}",
        );
        let file = parser.parse_file().unwrap();
        assert_eq!(file.imports.len(), 2);
        assert_eq!(file.imports[0].package, "std");
        assert_eq!(file.imports[0].file, "math/core.luna");
        assert_eq!(file.imports[1].package, "testing");
        assert_eq!(file.imports[1].file, "utils.luna");
    }

    #[test]
    fn test_parse_function() {
        use crate::compiler::ast;
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "func test(param1: string, param2: int) {}");
        let func = parser.parse_function().unwrap();
        assert_eq!(func.signature.id, "test");
        assert_eq!(func.signature.params.len(), 2);
        assert_eq!(func.signature.params[0].id, "param1");
        assert_eq!(
            func.signature.params[0].type_annotation,
            Box::new(ast::Type::String)
        );
        assert_eq!(func.signature.params[1].id, "param2");
        assert_eq!(
            func.signature.params[1].type_annotation,
            Box::new(ast::Type::Integer)
        );
    }

    #[test]
    fn test_parse_struct() {
        use crate::compiler::ast;
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "struct MyStruct { field1: string, field2: int }");
        let struct_ = parser.parse_struct().unwrap();
        assert_eq!(struct_.id, "MyStruct");
        assert_eq!(struct_.fields.len(), 2);
        assert_eq!(struct_.fields[0].id, "field1");
        assert_eq!(
            struct_.fields[0].type_annotation,
            Box::new(ast::Type::String)
        );
        assert_eq!(struct_.fields[1].id, "field2");
        assert_eq!(
            struct_.fields[1].type_annotation,
            Box::new(ast::Type::Integer)
        );
    }

    #[test]
    fn test_parse_struct_with_function() {
        use crate::compiler::ast;
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new(
            "testing",
            "struct MyStruct { field1: string, field2: int, func method() {} }",
        );
        let struct_ = parser.parse_struct().unwrap();
        assert_eq!(struct_.id, "MyStruct");
        assert_eq!(struct_.fields.len(), 2);
        assert_eq!(struct_.fields[0].id, "field1");
        assert_eq!(
            struct_.fields[0].type_annotation,
            Box::new(ast::Type::String)
        );
        assert_eq!(struct_.fields[1].id, "field2");
        assert_eq!(
            struct_.fields[1].type_annotation,
            Box::new(ast::Type::Integer)
        );
        assert_eq!(struct_.functions.len(), 1);
        assert_eq!(struct_.functions[0].signature.id, "method");
    }

    #[test]
    fn test_parse_enum() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "enum MyEnum { Variant1, Variant2(int) }");
        let enum_ = parser.parse_enum().unwrap();
        assert_eq!(enum_.id, "MyEnum");
        assert_eq!(enum_.variants.len(), 2);
        assert_eq!(enum_.variants[0].id, "Variant1");
        assert_eq!(enum_.variants[0].variant_types.len(), 0);
        assert_eq!(enum_.variants[1].id, "Variant2");
        assert_eq!(enum_.variants[1].variant_types.len(), 1);
    }

    #[test]
    fn test_parse_interface() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new(
            "testing",
            "interface MyInterface { func method(param: string); }",
        );
        let interface = parser.parse_interface().unwrap();
        assert_eq!(interface.id, "MyInterface");
        assert_eq!(interface.methods.len(), 1);
        assert_eq!(interface.methods[0].id, "method");
        assert_eq!(interface.methods[0].params.len(), 1);
        assert_eq!(interface.methods[0].params[0].id, "param");
        assert_eq!(
            interface.methods[0].params[0].type_annotation,
            Box::new(crate::compiler::ast::Type::String)
        );
    }

    #[test]
    fn test_parse_if() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "if x < 10 {} else {}");
        let if_ = parser.parse_if().unwrap();
        //assert_eq!(if_.consequent.stmts.len(), 0);
        assert!(if_.alternate.is_some());
        assert!(if_.not == false);
    }

    #[test]
    fn test_parse_if_not() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "if not x < 10 {} else {}");
        let if_ = parser.parse_if().unwrap();
        //assert_eq!(if_.consequent.stmts.len(), 0);
        assert!(if_.alternate.is_some());
        assert!(if_.not == true);
    }

    #[test]
    fn test_parse_while() {
        use crate::compiler::parser::Parser;
        let mut parser = Parser::new("testing", "while x < 10 {}");
        let _while = parser.parse_while().unwrap();
        //assert_eq!(while_.consequent.stmts.len(), 0);
    }

    #[test]
    fn test_parse_pattern() {
        use crate::compiler::parser::Parser;

        // let mut parser = Parser::new(".Variant");
        // let pattern = parser.parse_pattern().unwrap();
        // assert_eq!(pattern.kind, crate::compiler::ast::PatternKind::EnumVariant{id: "Variant".into(), values: vec![]});

        // let mut parser = Parser::new(".Variant(a, b, c)");
        // let pattern = parser.parse_pattern().unwrap();
        // assert_eq!(pattern.kind, crate::compiler::ast::PatternKind::EnumVariant{id: "Variant".into(), values: vec!["a".into(), "b".into(), "c".into()]});

        let mut parser = Parser::new("testing", "_");
        let pattern = parser.parse_pattern().unwrap();
        assert_eq!(pattern.kind, crate::compiler::ast::PatternKind::CatchAll);

        // let mut parser = Parser::new("123");
        // let pattern = parser.parse_pattern().unwrap();
        // assert_eq!(pattern.kind, crate::compiler::ast::PatternKind::Integer(123));

        // let mut parser = Parser::new("\"Steven!\"");
        // let pattern = parser.parse_pattern().unwrap();
        // assert_eq!(pattern.kind, crate::compiler::ast::PatternKind::String("Steven!".into()))
    }

    #[test]
    fn test_parse_switch() {
        use crate::compiler::parser::Parser;
        use crate::types;

        let mut parser = Parser::new("testing", "switch x { .Variant1: {} .Variant2(value): {} }");
        let switch = parser.parse_switch().unwrap();
        assert_eq!(switch.cases.len(), 2);
        assert_eq!(
            switch.cases[0].pattern.kind,
            crate::compiler::ast::PatternKind::EnumVariant {
                id: "Variant1".into(),
                values: vec![]
            }
        );
        assert_eq!(
            switch.cases[1].pattern.kind,
            crate::compiler::ast::PatternKind::EnumVariant {
                id: "Variant2".into(),
                values: vec![("value".into(), types::bad())]
            }
        );
    }

    #[test]
    fn test_parse_block() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "{ if x < 10 {} }");
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
        use crate::compiler::ast;
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "let x: int = 10;");
        let var_decl = parser.parse_var_decl_statement().unwrap();
        assert_eq!(var_decl.id, "x");
        assert_eq!(
            var_decl.type_annotation.unwrap(),
            Box::new(ast::Type::Integer)
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

        let mut parser = Parser::new("testing", "return 42;");
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
        use crate::compiler::ast;
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new("testing", "string bool int number []string myStruct");
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::String));
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::Bool));
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::Integer));
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::Number));
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::Array(Box::new(ast::Type::String))));
        let ty = parser.parse_type().unwrap();
        assert_eq!(ty, Box::new(ast::Type::Identifier("myStruct".into())));
    }

    #[test]
    fn test_parse_template() {
        use crate::compiler::parser::Parser;

        let mut parser = Parser::new(
            "testing",
            "\"Hello, ${name}! You have ${messages.length} new messages.\"",
        );
        let template = parser.parse_primary_expr().unwrap();
        if let crate::compiler::ast::ExprKind::Template(t) = &template.kind {
            assert_eq!(t.literals.len(), 3);
            assert_eq!(t.literals[0], "Hello, ");
            assert_eq!(t.literals[1], "! You have ");
            assert_eq!(t.literals[2], " new messages.");
            assert_eq!(t.expressions.len(), 2);
        } else {
            panic!("Expected template literal");
        }
    }
}

pub use self::attr::*;
pub use self::class::*;
pub use self::expr::*;
pub use self::func::*;
pub use self::imp::*;
pub use self::node::*;
pub use self::path::*;
pub use self::stmt::*;
pub use self::struc::*;
pub use self::ty::*;

use crate::lexer::{
    AsmKeyword, AttributeName, ClassKeyword, FnKeyword, Identifier, IfKeyword, ImplKeyword,
    LetKeyword, Lexer, StructKeyword, SyntaxError, Token,
};
use std::path::PathBuf;
use thiserror::Error;

mod attr;
mod class;
mod expr;
mod func;
mod imp;
mod node;
mod path;
mod stmt;
mod struc;
mod ty;

///  A parsed source file.
pub struct SourceFile {
    path: PathBuf,
    ty: Option<TypeDefinition>,
    impls: Vec<TypeImpl>,
}

impl SourceFile {
    pub fn parse<P: Into<PathBuf>>(path: P) -> Result<SourceFile, ParseError> {
        // Read the file.
        let path = path.into();
        let data = match std::fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => return Err(ParseError::ReadFailed(e)),
        };

        // Parse source file.
        let mut file = Self {
            path,
            ty: None,
            impls: Vec::new(),
        };

        if let Err(e) = file.parse_top(data) {
            return Err(ParseError::ParseFailed(e));
        }

        Ok(file)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn ty(&self) -> Option<&TypeDefinition> {
        self.ty.as_ref()
    }

    pub fn impls(&self) -> &[TypeImpl] {
        self.impls.as_ref()
    }

    fn parse_top(&mut self, data: String) -> Result<(), SyntaxError> {
        let mut lex = Lexer::new(data);
        let mut attrs = None;

        loop {
            // Get next token.
            let tok = match lex.next()? {
                Some(v) => v,
                None => break,
            };

            // Check token.
            match tok {
                Token::AttributeName(name) => attrs = Some(Self::parse_attrs(&mut lex, name)?),
                Token::StructKeyword(def) => {
                    let name = lex.next_ident()?;
                    self.can_define_type(&name)?;
                    self.ty = Some(TypeDefinition::Struct(Self::parse_struct(
                        &mut lex,
                        attrs.take().unwrap_or_default(),
                        def,
                        name,
                    )?));
                }
                Token::ClassKeyword(def) => {
                    let name = lex.next_ident()?;
                    self.can_define_type(&name)?;
                    self.ty = Some(TypeDefinition::Class(Self::parse_class(
                        &mut lex,
                        attrs.take().unwrap_or_default(),
                        def,
                        name,
                    )?));
                }
                Token::ImplKeyword(def) => {
                    let ty = lex.next_ident()?;
                    let tok = match lex.next()? {
                        Some(v) => v,
                        None => {
                            return Err(SyntaxError::new(
                                ty.span().clone(),
                                "expect '{' after this",
                            ));
                        }
                    };

                    match tok {
                        Token::OpenCurly(_) => {
                            match &self.ty {
                                Some(v) => {
                                    if *v.name() != ty {
                                        return Err(SyntaxError::new(
                                            ty.span().clone(),
                                            "an implementation is not matched with type in the file"
                                        ));
                                    }
                                }
                                None => {
                                    return Err(SyntaxError::new(
                                        ty.span().clone(),
                                        "type must be defined before define an implementation",
                                    ));
                                }
                            }

                            self.impls.push(Self::parse_type_impl(&mut lex, def, ty)?);
                        }
                        t => return Err(SyntaxError::new(t.span().clone(), "expect '{'")),
                    }
                }
                t => {
                    return Err(SyntaxError::new(
                        t.span().clone(),
                        "this item is not allowed as a top-level",
                    ));
                }
            }
        }

        Ok(())
    }

    fn parse_attrs(lex: &mut Lexer, first: AttributeName) -> Result<Vec<Attribute>, SyntaxError> {
        let mut attrs: Vec<Attribute> = Vec::new();

        attrs.push(Self::parse_attr(lex, first)?);

        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expected an item after this",
                    ));
                }
            };

            match tok {
                Token::AttributeName(name) => attrs.push(Self::parse_attr(lex, name)?),
                _ => {
                    lex.undo();
                    break;
                }
            }
        }

        Ok(attrs)
    }

    fn parse_attr(lex: &mut Lexer, name: AttributeName) -> Result<Attribute, SyntaxError> {
        let args = match lex.next()? {
            Some(Token::OpenParenthesis(_)) => Some(Self::parse_args(lex)?),
            Some(Token::CloseParenthesis(v)) => {
                return Err(SyntaxError::new(v.span().clone(), "expect '('"));
            }
            _ => {
                lex.undo();
                None
            }
        };

        Ok(Attribute::new(name, args))
    }

    fn parse_struct(
        lex: &mut Lexer,
        attrs: Vec<Attribute>,
        def: StructKeyword,
        name: Identifier,
    ) -> Result<Struct, SyntaxError> {
        // Check if zero-sized struct. A zero-sized struct without a repr attribute is not allowed.
        let tok = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    name.span().clone(),
                    "expect either ';' or '{' after struct name",
                ));
            }
        };

        match tok {
            Token::Semicolon(_) => return Ok(Struct::new(attrs, def, name)),
            Token::OpenCurly(_) => {}
            v => {
                return Err(SyntaxError::new(
                    v.span().clone(),
                    "expect either ';' or '{'",
                ));
            }
        }

        // Parse fields.
        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an '}'",
                    ));
                }
            };

            match tok {
                Token::CloseCurly(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "expect an '}'")),
            }
        }

        Ok(Struct::new(attrs, def, name))
    }

    fn parse_class(
        lex: &mut Lexer,
        attrs: Vec<Attribute>,
        def: ClassKeyword,
        name: Identifier,
    ) -> Result<Class, SyntaxError> {
        // Check if zero-sized class. A zero-sized class cannot be instantiate. They act as a
        // container for static methods.
        let tok = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    name.span().clone(),
                    "expect either ';' or '{' after the class name",
                ));
            }
        };

        match tok {
            Token::Semicolon(_) => return Ok(Class::new(attrs, def, name)),
            Token::OpenCurly(_) => {}
            v => {
                return Err(SyntaxError::new(
                    v.span().clone(),
                    "expect either ';' or '{'",
                ));
            }
        }

        // Parse fields.
        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an '}'",
                    ));
                }
            };

            match tok {
                Token::CloseCurly(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
            }
        }

        Ok(Class::new(attrs, def, name))
    }

    fn parse_type_impl(
        lex: &mut Lexer,
        def: ImplKeyword,
        ty: Identifier,
    ) -> Result<TypeImpl, SyntaxError> {
        let mut attrs = None;
        let mut functions = Vec::new();

        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an '}'",
                    ));
                }
            };

            match tok {
                Token::AttributeName(name) => attrs = Some(Self::parse_attrs(lex, name)?),
                Token::FnKeyword(def) => {
                    functions.push(Self::parse_fn(lex, attrs.take().unwrap_or_default(), def)?);
                }
                Token::CloseCurly(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
            }
        }

        Ok(TypeImpl::new(def, ty, functions))
    }

    fn parse_fn(
        lex: &mut Lexer,
        attrs: Vec<Attribute>,
        def: FnKeyword,
    ) -> Result<Function, SyntaxError> {
        let name = lex.next_ident()?;

        // Parse parameters.
        let mut params = Vec::new();

        lex.next_op()?;

        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an ')'",
                    ));
                }
            };

            match tok {
                Token::Identifier(name) => {
                    // Parse the parameter.
                    lex.next_colon()?;
                    params.push(FunctionParam::new(name, Self::parse_type(lex)?));

                    // Check for a ','.
                    let tok = match lex.next()? {
                        Some(v) => v,
                        None => {
                            return Err(SyntaxError::new(
                                lex.last().unwrap().clone(),
                                "expect an ')'",
                            ));
                        }
                    };

                    match tok {
                        Token::Comma(_) => {}
                        Token::CloseParenthesis(_) => break,
                        t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
                    }
                }
                Token::CloseParenthesis(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
            }
        }

        // Parse return type.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect either '{' or ';' after this",
                ));
            }
        };

        let ret = match next {
            Token::Semicolon(_) => return Ok(Function::new(attrs, def, name, params, None, None)),
            Token::OpenCurly(_) => None,
            Token::Colon(_) => {
                let ret = Self::parse_type(lex)?;
                let next = match lex.next()? {
                    Some(v) => v,
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect either '{' or ';' after this",
                        ));
                    }
                };

                match next {
                    Token::Semicolon(_) => {
                        return Ok(Function::new(attrs, def, name, params, Some(ret), None));
                    }
                    Token::OpenCurly(_) => {}
                    t => {
                        return Err(SyntaxError::new(
                            t.span().clone(),
                            "expect either '{' or ';'",
                        ));
                    }
                }

                Some(ret)
            }
            t => {
                return Err(SyntaxError::new(
                    t.span().clone(),
                    "expect either '{' or ';'",
                ));
            }
        };

        // Parse body.
        let body = Self::parse_block(lex)?;

        Ok(Function::new(attrs, def, name, params, ret, Some(body)))
    }

    fn parse_stmt(lex: &mut Lexer) -> Result<Option<Statement>, SyntaxError> {
        // Check for attribute.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an '}' after this",
                ));
            }
        };

        let attrs = match next {
            Token::AttributeName(name) => {
                let attrs = Self::parse_attrs(lex, name)?;
                let next = match lex.next()? {
                    Some(v) => v,
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect a statement after this",
                        ));
                    }
                };

                match next {
                    Token::CloseCurly(v) => {
                        return Err(SyntaxError::new(v.span().clone(), "expect a statement"));
                    }
                    _ => lex.undo(),
                }

                attrs
            }
            _ => {
                lex.undo();
                Vec::new()
            }
        };

        // Parse statement.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an '}' after this",
                ));
            }
        };

        let stmt = match next {
            Token::LetKeyword(v) => Statement::Let(Self::parse_let(lex, attrs, v)?),
            Token::CloseCurly(_) => return Ok(None),
            _ => {
                lex.undo();

                let exprs = Self::parse_exprs(lex)?;
                let next = match lex.next()? {
                    Some(v) => v,
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect an '}' after this",
                        ));
                    }
                };

                match next {
                    Token::Semicolon(_) => Statement::Unit(exprs),
                    Token::CloseCurly(_) => {
                        lex.undo();
                        Statement::Value(exprs)
                    }
                    t => return Err(SyntaxError::new(t.span().clone(), "expect ';'")),
                }
            }
        };

        Ok(Some(stmt))
    }

    fn parse_args(lex: &mut Lexer) -> Result<Vec<Vec<Expression>>, SyntaxError> {
        let mut args = Vec::new();

        loop {
            // Check for ')'.
            match lex.next()? {
                Some(Token::CloseParenthesis(_)) => break,
                Some(_) => lex.undo(),
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect ')' after this",
                    ));
                }
            }

            // Parse expression.
            args.push(Self::parse_exprs(lex)?);

            // Check for ','.
            let next = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect ')' after this",
                    ));
                }
            };

            match next {
                Token::Comma(_) => continue,
                Token::CloseParenthesis(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
            }
        }

        Ok(args)
    }

    fn parse_let(
        lex: &mut Lexer,
        attrs: Vec<Attribute>,
        def: LetKeyword,
    ) -> Result<Let, SyntaxError> {
        let var = lex.next_ident()?;
        lex.next_equals()?;

        let exprs = Self::parse_exprs(lex)?;
        lex.next_semicolon()?;

        Ok(Let::new(attrs, def, var, exprs))
    }

    fn parse_if(lex: &mut Lexer, def: IfKeyword) -> Result<If, SyntaxError> {
        // Parse condition.
        let exprs = Self::parse_exprs(lex)?;
        lex.next_oc()?;

        // Parse the body.
        let body = Self::parse_block(lex)?;

        Ok(If::new(def, exprs, body))
    }

    fn parse_block(lex: &mut Lexer) -> Result<Vec<Statement>, SyntaxError> {
        let mut block = Vec::new();

        while let Some(stmt) = Self::parse_stmt(lex)? {
            block.push(stmt);
        }

        Ok(block)
    }

    fn parse_exprs(lex: &mut Lexer) -> Result<Vec<Expression>, SyntaxError> {
        let mut exprs = Vec::new();

        loop {
            // Check the first item.
            let next = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an expression after this",
                    ));
                }
            };

            let ident = match next {
                Token::Identifier(v) => v,
                Token::UnsignedLiteral(v) => {
                    exprs.push(Expression::Unsigned(v));
                    continue;
                }
                Token::StringLiteral(v) => {
                    exprs.push(Expression::String(v));
                    continue;
                }
                Token::NullKeyword(v) => {
                    exprs.push(Expression::Null(v));
                    break;
                }
                Token::AsmKeyword(v) => {
                    exprs.push(Expression::Asm(Self::parse_asm(lex, v)?));
                    continue;
                }
                Token::IfKeyword(v) => {
                    exprs.push(Expression::If(Self::parse_if(lex, v)?));
                    continue;
                }
                _ => {
                    lex.undo();
                    break;
                }
            };

            // Check the token after the identifier.
            let second = match lex.next()? {
                Some(v) => v,
                None => {
                    exprs.push(Expression::Value(ident));
                    break;
                }
            };

            match second {
                Token::ExclamationMark(ex) => {
                    let eq = lex.next_equals()?;

                    exprs.push(Expression::Value(ident));
                    exprs.push(Expression::NotEqual(ex, eq));

                    continue;
                }
                Token::Equals(eq1) => {
                    let eq2 = lex.next_equals()?;

                    exprs.push(Expression::Value(ident));
                    exprs.push(Expression::Equal(eq1, eq2));

                    continue;
                }
                Token::OpenParenthesis(_) => {
                    let args = Self::parse_args(lex)?;

                    exprs.push(Expression::Call(Call::new(Vec::new(), ident, args)));

                    continue;
                }
                _ => {
                    lex.undo();
                    exprs.push(Expression::Value(ident));
                    break;
                }
            }
        }

        Ok(exprs)
    }

    fn parse_asm(lex: &mut Lexer, def: AsmKeyword) -> Result<Asm, SyntaxError> {
        lex.next_op()?;

        // Get the instruction.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an instruction after this",
                ));
            }
        };

        let inst = match next {
            Token::StringLiteral(v) => v,
            t => return Err(SyntaxError::new(t.span().clone(), "expect an instruction")),
        };

        // Parse the arguments.
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        match lex.next()? {
            Some(Token::Comma(_)) => loop {
                let next = match lex.next()? {
                    Some(v) => v,
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect ')' after this",
                        ));
                    }
                };

                match next {
                    Token::Identifier(v) => {
                        match v.value() {
                            "in" => inputs.push(Self::parse_asm_in(lex)?),
                            "out" => outputs.push(Self::parse_asm_out(lex)?),
                            _ => {
                                return Err(SyntaxError::new(v.span().clone(), "unknown argument"));
                            }
                        }

                        // Check for comma.
                        let next = match lex.next()? {
                            Some(v) => v,
                            None => {
                                return Err(SyntaxError::new(
                                    lex.last().unwrap().clone(),
                                    "expect ')' after this",
                                ));
                            }
                        };

                        match next {
                            Token::Comma(_) => {}
                            Token::CloseParenthesis(_) => break,
                            t => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
                        }
                    }
                    Token::CloseParenthesis(_) => break,
                    t => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
                }
            },
            Some(Token::CloseParenthesis(_)) => {}
            Some(t) => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect ')' after this",
                ));
            }
        }

        Ok(Asm::new(def, inst, inputs, outputs))
    }

    fn parse_asm_in(lex: &mut Lexer) -> Result<(AsmIn, Vec<Expression>), SyntaxError> {
        // Load target register.
        lex.next_op()?;

        let reg = match lex.next()? {
            Some(v) => match v {
                Token::StringLiteral(v) => AsmIn::Register(v),
                t => return Err(SyntaxError::new(t.span().clone(), "invalid input")),
            },
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an item after this",
                ));
            }
        };

        // Load the value.
        lex.next_cp()?;

        Ok((reg, Self::parse_exprs(lex)?))
    }

    fn parse_asm_out(lex: &mut Lexer) -> Result<(AsmOut, Identifier), SyntaxError> {
        // Load output register.
        lex.next_op()?;

        let reg = match lex.next()? {
            Some(v) => match v {
                Token::ExclamationMark(v) => AsmOut::Never(v),
                t => return Err(SyntaxError::new(t.span().clone(), "invalid output")),
            },
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an item after this",
                ));
            }
        };

        // Load the target variable.
        let cp = lex.next_cp()?;
        let var = match lex.next()? {
            Some(Token::Identifier(v)) => v,
            Some(t) => return Err(SyntaxError::new(t.span().clone(), "expect an identifier")),
            None => {
                return Err(SyntaxError::new(
                    cp.span().clone(),
                    "expect an identifier after this",
                ));
            }
        };

        Ok((reg, var))
    }

    fn parse_type(lex: &mut Lexer) -> Result<Type, SyntaxError> {
        // Parse pointer prefix.
        let mut prefixes = Vec::new();

        loop {
            match lex.next()? {
                Some(Token::Asterisk(v)) => prefixes.push(v),
                Some(_) => {
                    lex.undo();
                    break;
                }
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an identifier after this",
                    ));
                }
            }
        }

        // Parse type.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an identifier after this",
                ));
            }
        };

        let name = match next {
            Token::ExclamationMark(v) => TypeName::Never(v),
            Token::OpenParenthesis(o) => TypeName::Unit(o, lex.next_cp()?),
            Token::Identifier(mut ident) => {
                let mut fqtn = Vec::new();

                loop {
                    match lex.next()? {
                        Some(Token::FullStop(v)) => {
                            fqtn.push(Token::FullStop(v));
                            fqtn.push(Token::Identifier(ident));
                        }
                        Some(_) => {
                            lex.undo();
                            break;
                        }
                        None => break,
                    }

                    ident = match lex.next()? {
                        Some(Token::Identifier(v)) => v,
                        Some(t) => {
                            return Err(SyntaxError::new(t.span().clone(), "expect an identifier"))
                        }
                        None => {
                            return Err(SyntaxError::new(
                                lex.last().unwrap().clone(),
                                "expect an identifier after this",
                            ));
                        }
                    };
                }

                fqtn.push(Token::Identifier(ident));

                TypeName::Ident(Path::new(fqtn))
            }
            t => return Err(SyntaxError::new(t.span().clone(), "invalid type")),
        };

        Ok(Type::new(prefixes, name))
    }

    fn can_define_type(&self, name: &Identifier) -> Result<(), SyntaxError> {
        if self.ty.is_some() {
            return Err(SyntaxError::new(
                name.span().clone(),
                "multiple type definition in a source file",
            ));
        } else if name.value() != self.path.file_stem().unwrap() {
            return Err(SyntaxError::new(
                name.span().clone(),
                "type name and file name must be matched",
            ));
        }

        Ok(())
    }
}

/// A type definition in a source file.
pub enum TypeDefinition {
    Struct(Struct),
    Class(Class),
}

impl TypeDefinition {
    pub fn name(&self) -> &Identifier {
        match self {
            Self::Struct(v) => v.name(),
            Self::Class(v) => v.name(),
        }
    }
}

/// Represents an error when [`SourceFile::parse()`] is failed.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("cannot read source file")]
    ReadFailed(#[source] std::io::Error),

    #[error("cannot parse source file")]
    ParseFailed(#[source] SyntaxError),
}

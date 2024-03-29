use self::attr::Attributes;
use self::bt::BasicType;
use self::func::{Function, FunctionParam};
use self::imp::TypeImpl;
use self::path::Path;
use self::stmt::Statement;
use self::ty::{Type, TypeName};
use self::using::Use;
use crate::codegen::Codegen;
use crate::lexer::{Identifier, ImplKeyword, Lexer, SyntaxError, Token};
use crate::pkg::{Public, TypeDeclaration};
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

mod attr;
mod bt;
mod expr;
mod func;
mod imp;
mod path;
mod stmt;
mod ty;
mod using;

///  A parsed source file.
pub struct SourceFile {
    path: PathBuf,
    uses: Vec<Use>,
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
            uses: Vec::new(),
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

    pub fn has_type(&self) -> bool {
        self.ty.is_some()
    }

    fn ty(&self) -> Option<&TypeDefinition> {
        self.ty.as_ref()
    }

    pub fn build<'a, 'b: 'a>(
        &self,
        cg: &'a mut Codegen<'b>,
    ) -> Result<Option<TypeDeclaration>, SyntaxError> {
        // Get fully qualified type name.
        let ty = self.ty.as_ref().unwrap();
        let fqtn = if cg.namespace().is_empty() {
            Cow::Borrowed(ty.name().value())
        } else {
            Cow::Owned(format!("{}.{}", cg.namespace(), ty.name().value()))
        };

        // Build the type.
        let pkg = match ty {
            TypeDefinition::Basic(ty) => {
                let mut funcs = HashSet::new();

                for im in &self.impls {
                    for func in im.functions() {
                        let exp = match func.build(cg, &fqtn, &self.uses)? {
                            Some(v) => v,
                            None => continue,
                        };

                        if func
                            .attrs()
                            .public()
                            .filter(|v| v.1 == Public::External)
                            .is_some()
                        {
                            funcs.insert(exp);
                        }
                    }
                }

                TypeDeclaration::Basic(crate::pkg::BasicType::new(
                    ty.is_ref(),
                    ty.attrs().to_external(),
                    fqtn.into_owned(),
                    funcs,
                ))
            }
        };

        if ty
            .attrs()
            .public()
            .filter(|p| p.1 == Public::External)
            .is_some()
        {
            Ok(Some(pkg))
        } else {
            Ok(None)
        }
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
                Token::AttributeName(name) => attrs = Some(Attributes::parse(&mut lex, name)?),
                Token::UseKeyword(def) => {
                    self.uses
                        .push(Use::parse(&mut lex, attrs.take().unwrap_or_default(), def)?)
                }
                Token::StructKeyword(_) => {
                    let name = lex.next_ident()?;
                    self.can_define_type(&name)?;
                    self.ty = Some(TypeDefinition::Basic(Self::parse_basic(
                        &mut lex,
                        attrs.take().unwrap_or_default(),
                        false,
                        name,
                    )?));
                }
                Token::ClassKeyword(_) => {
                    let name = lex.next_ident()?;
                    self.can_define_type(&name)?;
                    self.ty = Some(TypeDefinition::Basic(Self::parse_basic(
                        &mut lex,
                        attrs.take().unwrap_or_default(),
                        true,
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

    fn parse_basic(
        lex: &mut Lexer,
        attrs: Attributes,
        class: bool,
        name: Identifier,
    ) -> Result<BasicType, SyntaxError> {
        // Check if body available.
        match lex.next()? {
            Some(Token::Semicolon(_)) => {
                if !class && attrs.repr().is_none() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "primitive struct without repr attribute is not allowed",
                    ));
                }

                return Ok(BasicType::new(attrs, class, name));
            }
            Some(Token::OpenCurly(_)) => {}
            Some(t) => return Err(SyntaxError::new(t.span(), "expect either ';' or '}'")),
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap(),
                    "expect either ';' or '}' after this",
                ));
            }
        }

        // Parse fields.
        loop {
            let tok = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap(),
                        "expect '}' after this",
                    ));
                }
            };

            match tok {
                Token::CloseCurly(_) => break,
                t => return Err(SyntaxError::new(t.span(), "expect '}'")),
            }
        }

        Ok(BasicType::new(attrs, class, name))
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
                Token::AttributeName(name) => attrs = Some(Attributes::parse(lex, name)?),
                Token::FnKeyword(_) => {
                    functions.push(Self::parse_fn(lex, attrs.take().unwrap_or_default())?);
                }
                Token::CloseCurly(_) => break,
                t => return Err(SyntaxError::new(t.span().clone(), "syntax error")),
            }
        }

        Ok(TypeImpl::new(def, ty, functions))
    }

    fn parse_fn(lex: &mut Lexer, attrs: Attributes) -> Result<Function, SyntaxError> {
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

                    let ty = Self::parse_type(lex)?;

                    match ty.name() {
                        TypeName::Unit(o, c) => {
                            return Err(SyntaxError::new(
                                o.span() + c.span(),
                                "unit type cannot be a function parameter",
                            ));
                        }
                        TypeName::Never(t) => {
                            return Err(SyntaxError::new(
                                t.span().clone(),
                                "never type cannot be a function parameter",
                            ));
                        }
                        TypeName::Ident(_) => params.push(FunctionParam::new(name, ty)),
                    }

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
            Token::Semicolon(_) => return Ok(Function::new(attrs, name, params, None, None)),
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
                        return Ok(Function::new(attrs, name, params, Some(ret), None));
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
        let body = Statement::parse_block(lex)?;

        Ok(Function::new(attrs, name, params, ret, Some(body)))
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
            Token::ExclamationMark(v) => {
                if prefixes.is_empty() {
                    TypeName::Never(v)
                } else {
                    return Err(SyntaxError::new(
                        v.span().clone(),
                        "never type cannot be a pointer",
                    ));
                }
            }
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
enum TypeDefinition {
    Basic(BasicType),
}

impl TypeDefinition {
    pub fn attrs(&self) -> &Attributes {
        match self {
            Self::Basic(v) => v.attrs(),
        }
    }

    pub fn name(&self) -> &Identifier {
        match self {
            Self::Basic(v) => v.name(),
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

use super::bt::BasicType;
use super::{Path, SourceFile, TypeDefinition, Use};
use crate::codegen::{
    Codegen, LlvmI32, LlvmPtr, LlvmType, LlvmU64, LlvmU8, LlvmVoid, ResolvedType,
};
use crate::lexer::{
    Asterisk, CloseParenthesis, ExclamationMark, OpenParenthesis, Span, SyntaxError,
};
use crate::pkg::{Representation, TypeDeclaration};

/// A type of something (e.g. variable).
pub(super) struct Type {
    prefixes: Vec<Asterisk>,
    name: TypeName,
}

impl Type {
    pub fn new(prefixes: Vec<Asterisk>, name: TypeName) -> Self {
        Self { prefixes, name }
    }

    pub fn name(&self) -> &TypeName {
        &self.name
    }

    pub fn build<'a, 'b: 'a, U: IntoIterator<Item = &'a Use>>(
        &self,
        cx: &'a Codegen<'b>,
        uses: U,
    ) -> Result<Option<LlvmType<'a, 'b>>, SyntaxError> {
        // Resolve base type.
        let mut ty = match &self.name {
            TypeName::Unit(_, _) => LlvmType::Void(LlvmVoid::new(cx)),
            TypeName::Never(_) => return Ok(None),
            TypeName::Ident(n) => match Self::resolve(cx, uses, n) {
                Some((n, t)) => match t {
                    ResolvedType::Internal(v) => Self::build_internal_type(cx, &n, v),
                    ResolvedType::External((_, t)) => Self::build_external_type(cx, &n, t),
                },
                None => return Err(SyntaxError::new(n.span(), "type is undefined")),
            },
        };

        // Resolve pointers.
        for _ in self.prefixes.iter().rev() {
            ty = LlvmType::Ptr(LlvmPtr::new(cx, ty));
        }

        Ok(Some(ty))
    }

    pub fn to_external<'a, 'b: 'a, U: IntoIterator<Item = &'a Use>>(
        &self,
        cx: &'a Codegen<'b>,
        uses: U,
    ) -> Option<crate::pkg::Type> {
        use crate::pkg::Type;

        let ptr = self.prefixes.len();
        let ty = match &self.name {
            TypeName::Unit(_, _) => Type::Unit { ptr },
            TypeName::Never(_) => Type::Never,
            TypeName::Ident(n) => {
                let (n, t) = Self::resolve(cx, uses, n)?;

                match t {
                    ResolvedType::Internal(s) => {
                        // Strip "self.".
                        let name = n[5..].to_owned();
                        let pkg = None;

                        match s.ty.as_ref().unwrap() {
                            TypeDefinition::Basic(t) => {
                                if t.is_ref() {
                                    Type::Class { ptr, pkg, name }
                                } else {
                                    Type::Struct { ptr, pkg, name }
                                }
                            }
                        }
                    }
                    ResolvedType::External((p, t)) => {
                        let pkg = Some((p.name().as_str().to_owned(), p.version().major()));
                        let name = t.name().to_owned();

                        match t {
                            TypeDeclaration::Basic(t) => {
                                if t.is_class() {
                                    Type::Class { ptr, pkg, name }
                                } else {
                                    Type::Struct { ptr, pkg, name }
                                }
                            }
                        }
                    }
                }
            }
        };

        Some(ty)
    }

    fn build_internal_type<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        name: &str,
        ty: &SourceFile,
    ) -> LlvmType<'a, 'b> {
        match ty.ty().unwrap() {
            TypeDefinition::Basic(v) => {
                if v.is_ref() {
                    todo!()
                } else {
                    Self::build_internal_struct(cx, name, v)
                }
            }
        }
    }

    fn build_external_type<'a, 'b: 'a>(
        cg: &'a Codegen<'b>,
        name: &str,
        ty: &TypeDeclaration,
    ) -> LlvmType<'a, 'b> {
        match ty {
            TypeDeclaration::Basic(bt) => {
                if bt.is_class() {
                    todo!()
                } else {
                    Self::build_external_struct(cg, name, bt)
                }
            }
        }
    }

    fn build_internal_struct<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        name: &str,
        ty: &BasicType,
    ) -> LlvmType<'a, 'b> {
        assert!(!ty.is_ref());

        match ty.attrs().repr() {
            Some(v) => Self::build_primitive_struct(cx, v.1),
            None => todo!(),
        }
    }

    fn build_external_struct<'a, 'b: 'a>(
        cg: &'a Codegen<'b>,
        name: &str,
        ty: &crate::pkg::BasicType,
    ) -> LlvmType<'a, 'b> {
        assert!(!ty.is_class());

        match ty.attrs().repr() {
            Some(v) => Self::build_primitive_struct(cg, v),
            None => todo!(),
        }
    }

    fn build_primitive_struct<'a, 'b: 'a>(
        cg: &'a Codegen<'b>,
        repr: Representation,
    ) -> LlvmType<'a, 'b> {
        match repr {
            Representation::I32 => LlvmType::I32(LlvmI32::new(cg)),
            Representation::U8 => LlvmType::U8(LlvmU8::new(cg)),
            Representation::Un => match cg.pointer_size() {
                8 => LlvmType::U64(LlvmU64::new(cg)),
                _ => todo!(),
            },
        }
    }

    fn resolve<'a, 'b: 'a, U: IntoIterator<Item = &'a Use>>(
        cg: &'a Codegen<'b>,
        uses: U,
        name: &Path,
    ) -> Option<(String, &'b ResolvedType<'b>)> {
        // Resolve full name.
        let name = match name.as_local() {
            Some(name) => {
                // Search from use declarations first to allow overrides.
                let mut found = None;

                for u in uses {
                    match u.rename() {
                        Some(v) => {
                            if v == name {
                                found = Some(u);
                            }
                        }
                        None => {
                            if u.name().last() == name {
                                found = Some(u);
                            }
                        }
                    }
                }

                match found {
                    Some(v) => v.name().to_string(),
                    None => {
                        if cg.namespace().is_empty() {
                            format!("self.{}", name)
                        } else {
                            format!("self.{}.{}", cg.namespace(), name)
                        }
                    }
                }
            }
            None => name.to_string(),
        };

        // Resolve type.
        let ty = cg.resolver().resolve(&name)?;

        Some((name, ty))
    }
}

/// Name of a [`Type`].
pub(super) enum TypeName {
    Unit(OpenParenthesis, CloseParenthesis),
    Never(ExclamationMark),
    Ident(Path),
}

impl TypeName {
    pub fn span(&self) -> Span {
        match self {
            TypeName::Unit(o, c) => o.span() + c.span(),
            TypeName::Never(v) => v.span().clone(),
            TypeName::Ident(v) => v.span(),
        }
    }
}

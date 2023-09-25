use super::{Path, Representation, SourceFile, Struct, TypeDefinition, Use};
use crate::codegen::{
    Codegen, LlvmI32, LlvmPtr, LlvmType, LlvmU64, LlvmU8, LlvmVoid, ResolvedType,
};
use crate::lexer::{
    Asterisk, CloseParenthesis, ExclamationMark, OpenParenthesis, Span, SyntaxError,
};

/// A type of something (e.g. variable).
pub struct Type {
    prefixes: Vec<Asterisk>,
    name: TypeName,
}

impl Type {
    pub fn new(prefixes: Vec<Asterisk>, name: TypeName) -> Self {
        Self { prefixes, name }
    }

    pub fn prefixes(&self) -> &[Asterisk] {
        self.prefixes.as_ref()
    }

    pub fn name(&self) -> &TypeName {
        &self.name
    }

    pub fn build<'a, 'b: 'a>(
        &self,
        cx: &'a Codegen<'b>,
        uses: &[Use],
    ) -> Result<Option<LlvmType<'a, 'b>>, SyntaxError> {
        // Resolve base type.
        let mut ty = match &self.name {
            TypeName::Unit(_, _) => LlvmType::Void(LlvmVoid::new(cx)),
            TypeName::Never(_) => return Ok(None),
            TypeName::Ident(n) => match cx.resolve(uses, n) {
                Some((n, t)) => match t {
                    ResolvedType::Project(v) => Self::build_project_type(cx, &n, v),
                    ResolvedType::External(_) => todo!(),
                },
                None => return Err(SyntaxError::new(n.span(), "type is undefined")),
            },
        };

        // Resolve pointers.
        for p in self.prefixes.iter().rev() {
            ty = LlvmType::Ptr(LlvmPtr::new(cx, ty));
        }

        Ok(Some(ty))
    }

    fn build_project_type<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        name: &str,
        ty: &SourceFile,
    ) -> LlvmType<'a, 'b> {
        match ty.ty().unwrap() {
            TypeDefinition::Struct(v) => Self::build_project_struct(cx, name, v),
            TypeDefinition::Class(_) => todo!(),
        }
    }

    fn build_project_struct<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        name: &str,
        ty: &Struct,
    ) -> LlvmType<'a, 'b> {
        match ty {
            Struct::Primitive(_, r, _, _) => match r {
                Representation::I32 => LlvmType::I32(LlvmI32::new(cx)),
                Representation::U8 => LlvmType::U8(LlvmU8::new(cx)),
                Representation::Un => match cx.pointer_size() {
                    8 => LlvmType::U64(LlvmU64::new(cx)),
                    _ => todo!(),
                },
            },
            Struct::Composite(_, _, _) => todo!(),
        }
    }
}

/// Name of a [`Type`].
pub enum TypeName {
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

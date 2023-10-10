use super::{Path, SourceFile, TypeDefinition, Use};
use crate::codegen::{
    Codegen, LlvmI32, LlvmPtr, LlvmType, LlvmU64, LlvmU8, LlvmVoid, ResolvedType,
};
use crate::lexer::{
    Asterisk, CloseParenthesis, ExclamationMark, OpenParenthesis, Span, SyntaxError,
};
use crate::ty::{BasicType, Representation};

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

    pub fn build<'a, 'b: 'a, U: IntoIterator<Item = &'a Use>>(
        &self,
        cx: &'a Codegen<'b>,
        uses: U,
    ) -> Result<Option<LlvmType<'a, 'b>>, SyntaxError> {
        // Resolve base type.
        let mut ty = match &self.name {
            TypeName::Unit(_, _) => LlvmType::Void(LlvmVoid::new(cx)),
            TypeName::Never(_) => return Ok(None),
            TypeName::Ident(n) => match cx.resolve(uses, n) {
                Some((n, t)) => match t {
                    ResolvedType::Internal(v) => Self::build_internal_type(cx, &n, v),
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

    pub fn to_export<'a, 'b: 'a>(&self, cx: &'a Codegen<'b>, uses: &[Use]) -> crate::pkg::Type {
        use crate::pkg::Type;

        let ptr = self.prefixes.len();

        match &self.name {
            TypeName::Unit(_, _) => Type::Unit(ptr),
            TypeName::Never(_) => Type::Never,
            TypeName::Ident(n) => {
                let (n, t) = cx.resolve(uses, n).unwrap();

                match t {
                    ResolvedType::Internal(f) => {
                        Type::Local(ptr, n.strip_prefix("self.").unwrap().to_owned())
                    }
                    ResolvedType::External((p, t)) => Type::External(
                        ptr,
                        p.name().to_string(),
                        p.version().major(),
                        t.name().to_owned(),
                    ),
                }
            }
        }
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
                    Self::build_struct(cx, name, v)
                }
            }
        }
    }

    fn build_struct<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        name: &str,
        ty: &dyn BasicType,
    ) -> LlvmType<'a, 'b> {
        assert!(!ty.is_ref());

        match ty.attrs().repr() {
            Some(v) => match v {
                Representation::I32 => LlvmType::I32(LlvmI32::new(cx)),
                Representation::U8 => LlvmType::U8(LlvmU8::new(cx)),
                Representation::Un => match cx.pointer_size() {
                    8 => LlvmType::U64(LlvmU64::new(cx)),
                    _ => todo!(),
                },
            },
            None => todo!(),
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

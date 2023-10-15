use super::{Attributes, Statement, Type, Use};
use crate::codegen::{BasicBlock, Builder, Codegen, LlvmFunc, LlvmType, LlvmVoid};
use crate::lexer::{Identifier, SyntaxError};
use crate::pkg::Extern;
use std::borrow::Cow;
use std::ffi::CString;

/// A function in the source file.
pub(super) struct Function {
    attrs: Attributes,
    name: Identifier,
    params: Vec<FunctionParam>,
    ret: Option<Type>,
    body: Option<Vec<Statement>>,
}

impl Function {
    pub fn new(
        attrs: Attributes,
        name: Identifier,
        params: Vec<FunctionParam>,
        ret: Option<Type>,
        body: Option<Vec<Statement>>,
    ) -> Self {
        Self {
            attrs,
            name,
            params,
            ret,
            body,
        }
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn build<'a, 'b: 'a, U: IntoIterator<Item = &'a Use> + Clone>(
        &self,
        cx: &'a Codegen<'b>,
        container: &str,
        uses: U,
    ) -> Result<Option<crate::pkg::Function>, SyntaxError> {
        // Check condition.
        if !self.attrs.run_condition(cx)? {
            return Ok(None);
        }

        // Get public type.
        let ext = crate::pkg::Function::new(
            self.name.value().to_owned(),
            {
                let mut params = Vec::with_capacity(self.params.len());

                for p in &self.params {
                    let t = match p.ty.to_external(cx, uses.clone()) {
                        Some(v) => v,
                        None => return Err(SyntaxError::new(p.ty.name().span(), "undefined type")),
                    };

                    params.push(crate::pkg::FunctionParam::new(p.name.value().to_owned(), t));
                }

                params
            },
            match &self.ret {
                Some(v) => match v.to_external(cx, uses.clone()) {
                    Some(v) => v,
                    None => return Err(SyntaxError::new(v.name().span(), "undefined type")),
                },
                None => crate::pkg::Type::Unit { ptr: 0 },
            },
        );

        // Build function name.
        let name = match self.attrs.ext() {
            Some((_, Extern::C)) => Cow::Borrowed(self.name.value()),
            None => Cow::Owned(ext.mangle(
                if cx.executable() {
                    None
                } else {
                    Some((cx.pkg().as_str(), cx.version().major()))
                },
                container,
            )),
        };

        // Check if function already exists.
        let name = CString::new(name.as_ref()).unwrap();

        if LlvmFunc::get(cx, &name).is_some() {
            return Err(SyntaxError::new(
                self.name.span(),
                "multiple definition of the same name",
            ));
        }

        // Get params.
        let mut params = Vec::<LlvmType<'a, 'b>>::new();

        for p in &self.params {
            let ty = match p.ty.build(cx, uses.clone())? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        p.ty.name().span(),
                        "function parameter cannot be a never type",
                    ));
                }
            };

            params.push(ty);
        }

        // Get return type.
        let mut never = false;
        let ret = match &self.ret {
            Some(v) => match v.build(cx, uses)? {
                Some(v) => v,
                None => {
                    never = true;
                    LlvmType::Void(LlvmVoid::new(cx))
                }
            },
            None => LlvmType::Void(LlvmVoid::new(cx)),
        };

        // Create a function.
        let mut func = LlvmFunc::new(cx, name, &params, ret);

        match &self.body {
            Some(v) => Self::build_body(cx, &mut func, v),
            None => {
                if self.attrs.ext().is_none() {
                    return Err(SyntaxError::new(
                        self.name.span(),
                        "a body is required for non-extern or non-abstract",
                    ));
                }
            }
        }

        Ok(Some(ext))
    }

    fn build_body<'a, 'b: 'a>(
        cx: &'a Codegen<'b>,
        func: &mut LlvmFunc<'a, 'b>,
        stmts: &[Statement],
    ) {
        let mut bb = BasicBlock::new(cx);
        let mut b = Builder::new(cx, &mut bb);

        b.ret_void();

        func.append(bb);
    }
}

/// A parameter of a function in the source file.
pub(super) struct FunctionParam {
    name: Identifier,
    ty: Type,
}

impl FunctionParam {
    pub fn new(name: Identifier, ty: Type) -> Self {
        Self { name, ty }
    }
}

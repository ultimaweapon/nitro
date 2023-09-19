use super::{Attributes, Statement, Type};
use crate::codegen::{Codegen, LlvmFunc, LlvmType, LlvmVoid};
use crate::lexer::{FnKeyword, Identifier, SyntaxError};
use std::ffi::CString;

/// A function.
pub struct Function {
    attrs: Attributes,
    def: FnKeyword,
    name: Identifier,
    params: Vec<FunctionParam>,
    ret: Option<Type>,
    body: Option<Vec<Statement>>,
}

impl Function {
    pub fn new(
        attrs: Attributes,
        def: FnKeyword,
        name: Identifier,
        params: Vec<FunctionParam>,
        ret: Option<Type>,
        body: Option<Vec<Statement>>,
    ) -> Self {
        Self {
            attrs,
            def,
            name,
            params,
            ret,
            body,
        }
    }

    pub fn build<'a, 'b: 'a>(
        &self,
        cx: &'a Codegen<'b>,
        container: &str,
    ) -> Result<Option<LlvmFunc<'a, 'b>>, SyntaxError> {
        // Check cfg attribute.
        if let Some((_, cfg)) = self.attrs.config() {
            if !cx.run_cfg(cfg)? {
                return Ok(None);
            }
        }

        // Check if function already exists.
        let name = CString::new(cx.encode_name(container, self.name.value())).unwrap();

        if LlvmFunc::get(cx, &name).is_some() {
            return Err(SyntaxError::new(
                self.name.span().clone(),
                "multiple definition of the same name",
            ));
        }

        // Get params.
        let mut params = Vec::<LlvmType<'a, 'b>>::new();

        for p in &self.params {
            let ty = match p.ty.build(cx, &[])? {
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
            Some(v) => match v.build(cx, &[])? {
                Some(v) => v,
                None => {
                    never = true;
                    LlvmType::Void(LlvmVoid::new(cx))
                }
            },
            None => LlvmType::Void(LlvmVoid::new(cx)),
        };

        // Create a function.
        let func = LlvmFunc::new(cx, name, &params, ret);

        // TODO: Build function body.
        Ok(Some(func))
    }
}

/// A function parameter.
pub struct FunctionParam {
    name: Identifier,
    ty: Type,
}

impl FunctionParam {
    pub fn new(name: Identifier, ty: Type) -> Self {
        Self { name, ty }
    }
}

use crate::ast::SourceFile;
use crate::pkg::ExportedType;
use std::collections::HashMap;

/// An object to resolve types.
pub struct Resolver<'a> {
    types: HashMap<String, ResolvedType<'a>>,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn resolve(&self, name: &str) -> Option<ResolvedType<'a>> {
        todo!()
    }
}

/// A type that was resolved by [`Resolver`].
pub enum ResolvedType<'a> {
    Project(&'a SourceFile),
    External(&'a ExportedType),
}

use crate::ast::SourceFile;
use crate::pkg::{ExportedType, PackageMeta};
use std::collections::HashMap;

/// An object to resolve types.
pub struct TypeResolver<'a> {
    types: HashMap<String, ResolvedType<'a>>,
}

impl<'a> TypeResolver<'a> {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&ResolvedType<'a>> {
        self.types.get(name)
    }

    pub fn populate_internal_types<S>(&mut self, set: S)
    where
        S: IntoIterator<Item = (&'a String, &'a SourceFile)>,
    {
        for (name, ty) in set {
            let mut key = String::from("self.");

            key.push_str(&name);

            assert!(self.types.insert(key, ResolvedType::Internal(ty)).is_none());
        }
    }

    pub fn populate_external_types<S>(&mut self, pkg: &'a PackageMeta, types: S)
    where
        S: IntoIterator<Item = &'a ExportedType>,
    {
        for ty in types {
            let mut key = pkg.name().as_str().to_owned();

            key.push('.');
            key.push_str(ty.name());

            assert!(self
                .types
                .insert(key, ResolvedType::External((pkg, ty)))
                .is_none());
        }
    }
}

/// A type that was resolved by [`TypeResolver`].
pub enum ResolvedType<'a> {
    Internal(&'a SourceFile),
    External((&'a PackageMeta, &'a ExportedType)),
}

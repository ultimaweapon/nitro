use crate::ast::SourceFile;
use crate::pkg::ExportedType;
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

    pub fn populate_project_types<S>(&mut self, set: S)
    where
        S: IntoIterator<Item = (&'a String, &'a SourceFile)>,
    {
        for (name, ty) in set {
            let mut key = String::from("self.");

            key.push_str(&name);

            assert!(self.types.insert(key, ResolvedType::Project(ty)).is_none());
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&ResolvedType<'a>> {
        self.types.get(name)
    }
}

/// A type that was resolved by [`Resolver`].
pub enum ResolvedType<'a> {
    Project(&'a SourceFile),
    External(&'a ExportedType),
}

use std::hash::{Hash, Hasher};

/// A type that was exported from a package.
pub struct ExportedType {
    name: String,
}

impl ExportedType {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl PartialEq for ExportedType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ExportedType {}

impl Hash for ExportedType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

use super::Path;
use crate::lexer::{Identifier, UseKeyword};

/// A `use` declaration.
pub struct Use {
    def: UseKeyword,
    name: Path,
    rename: Option<Identifier>,
}

impl Use {
    pub fn new(def: UseKeyword, name: Path, rename: Option<Identifier>) -> Self {
        assert!(name.as_local().is_none());

        Self { def, name, rename }
    }

    pub fn name(&self) -> &Path {
        &self.name
    }

    pub fn rename(&self) -> Option<&Identifier> {
        self.rename.as_ref()
    }
}

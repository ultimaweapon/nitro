/// Output target of the code.
pub struct Target<'a> {
    triple: &'a str,
}

impl<'a> Target<'a> {
    pub fn new(triple: &'a str) -> Self {
        Self { triple }
    }

    pub fn triple(&self) -> &'a str {
        self.triple
    }
}

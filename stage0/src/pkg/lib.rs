/// Contains information about a Nitro library.
///
/// A Nitro library is always a shared library. Nitro can consume a static library but cannot
/// produce it. The reason is because it will cause a surprising behavior to the user in the
/// following scenario:
///
/// - Alice publish a static library named `foo`.
/// - Bob publish a shared library named `bar` that link to `foo`.
/// - Carol publish a shared library named `baz` that also link to `foo`.
/// - Carlos build a binary that link to both `bar` and `baz`.
///
/// There will be two states of `foo` here, which likely to cause a headache to Alice to figure out
/// what wrong with `foo` when Carlos report something is not working.
pub struct Library {}

impl Library {
    pub fn new() -> Self {
        Self {}
    }
}

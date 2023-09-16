use super::Span;

/// A token in the source file.
pub enum Token {
    ExclamationMark(ExclamationMark),
    Equals(Equals),
    Asterisk(Asterisk),
    FullStop(FullStop),
    Comma(Comma),
    Colon(Colon),
    Semicolon(Semicolon),
    OpenParenthesis(OpenParenthesis),
    CloseParenthesis(CloseParenthesis),
    OpenCurly(OpenCurly),
    CloseCurly(CloseCurly),
    AttributeName(AttributeName),
    UnsignedLiteral(UnsignedLiteral),
    FloatLiteral(FloatLiteral),
    StringLiteral(StringLiteral),
    StructKeyword(StructKeyword),
    ClassKeyword(ClassKeyword),
    ImplKeyword(ImplKeyword),
    FnKeyword(FnKeyword),
    LetKeyword(LetKeyword),
    IfKeyword(IfKeyword),
    IsKeyword(IsKeyword),
    AsmKeyword(AsmKeyword),
    NullKeyword(NullKeyword),
    Identifier(Identifier),
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Self::ExclamationMark(v) => &v.0,
            Self::Equals(v) => &v.0,
            Self::Asterisk(v) => &v.0,
            Self::FullStop(v) => &v.0,
            Self::Comma(v) => &v.0,
            Self::Colon(v) => &v.0,
            Self::Semicolon(v) => &v.0,
            Self::OpenParenthesis(v) => &v.0,
            Self::CloseParenthesis(v) => &v.0,
            Self::OpenCurly(v) => &v.0,
            Self::CloseCurly(v) => &v.0,
            Self::AttributeName(v) => &v.span,
            Self::UnsignedLiteral(v) => &v.span,
            Self::FloatLiteral(v) => &v.span,
            Self::StringLiteral(v) => &v.span,
            Self::StructKeyword(v) => &v.0,
            Self::ClassKeyword(v) => &v.0,
            Self::ImplKeyword(v) => &v.0,
            Self::FnKeyword(v) => &v.0,
            Self::LetKeyword(v) => &v.0,
            Self::IfKeyword(v) => &v.0,
            Self::IsKeyword(v) => &v.0,
            Self::AsmKeyword(v) => &v.0,
            Self::NullKeyword(v) => &v.0,
            Self::Identifier(v) => &v.span,
        }
    }
}

impl From<ExclamationMark> for Token {
    fn from(value: ExclamationMark) -> Self {
        Self::ExclamationMark(value)
    }
}

impl From<Equals> for Token {
    fn from(value: Equals) -> Self {
        Self::Equals(value)
    }
}

impl From<Asterisk> for Token {
    fn from(value: Asterisk) -> Self {
        Self::Asterisk(value)
    }
}

impl From<FullStop> for Token {
    fn from(value: FullStop) -> Self {
        Self::FullStop(value)
    }
}

impl From<Comma> for Token {
    fn from(value: Comma) -> Self {
        Self::Comma(value)
    }
}

impl From<Colon> for Token {
    fn from(value: Colon) -> Self {
        Self::Colon(value)
    }
}

impl From<Semicolon> for Token {
    fn from(value: Semicolon) -> Self {
        Self::Semicolon(value)
    }
}

impl From<OpenParenthesis> for Token {
    fn from(value: OpenParenthesis) -> Self {
        Self::OpenParenthesis(value)
    }
}

impl From<CloseParenthesis> for Token {
    fn from(value: CloseParenthesis) -> Self {
        Self::CloseParenthesis(value)
    }
}

impl From<OpenCurly> for Token {
    fn from(value: OpenCurly) -> Self {
        Self::OpenCurly(value)
    }
}

impl From<CloseCurly> for Token {
    fn from(value: CloseCurly) -> Self {
        Self::CloseCurly(value)
    }
}

impl From<AttributeName> for Token {
    fn from(value: AttributeName) -> Self {
        Self::AttributeName(value)
    }
}

impl From<UnsignedLiteral> for Token {
    fn from(value: UnsignedLiteral) -> Self {
        Self::UnsignedLiteral(value)
    }
}

impl From<FloatLiteral> for Token {
    fn from(value: FloatLiteral) -> Self {
        Self::FloatLiteral(value)
    }
}

impl From<StringLiteral> for Token {
    fn from(value: StringLiteral) -> Self {
        Self::StringLiteral(value)
    }
}

impl From<StructKeyword> for Token {
    fn from(value: StructKeyword) -> Self {
        Self::StructKeyword(value)
    }
}

impl From<ClassKeyword> for Token {
    fn from(value: ClassKeyword) -> Self {
        Self::ClassKeyword(value)
    }
}

impl From<ImplKeyword> for Token {
    fn from(value: ImplKeyword) -> Self {
        Self::ImplKeyword(value)
    }
}

impl From<FnKeyword> for Token {
    fn from(value: FnKeyword) -> Self {
        Self::FnKeyword(value)
    }
}

impl From<LetKeyword> for Token {
    fn from(value: LetKeyword) -> Self {
        Self::LetKeyword(value)
    }
}

impl From<IfKeyword> for Token {
    fn from(value: IfKeyword) -> Self {
        Self::IfKeyword(value)
    }
}

impl From<IsKeyword> for Token {
    fn from(value: IsKeyword) -> Self {
        Self::IsKeyword(value)
    }
}

impl From<AsmKeyword> for Token {
    fn from(value: AsmKeyword) -> Self {
        Self::AsmKeyword(value)
    }
}

impl From<NullKeyword> for Token {
    fn from(value: NullKeyword) -> Self {
        Self::NullKeyword(value)
    }
}

impl From<Identifier> for Token {
    fn from(value: Identifier) -> Self {
        Self::Identifier(value)
    }
}

/// An `!` token.
pub struct ExclamationMark(Span);

impl ExclamationMark {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `=` token.
pub struct Equals(Span);

impl Equals {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `*` token.
pub struct Asterisk(Span);

impl Asterisk {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `.` token.
pub struct FullStop(Span);

impl FullStop {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `,` token.
pub struct Comma(Span);

impl Comma {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `:` token.
pub struct Colon(Span);

impl Colon {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `;` token.
pub struct Semicolon(Span);

impl Semicolon {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `(` token.
pub struct OpenParenthesis(Span);

impl OpenParenthesis {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `)` token.
pub struct CloseParenthesis(Span);

impl CloseParenthesis {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `{` token.
pub struct OpenCurly(Span);

impl OpenCurly {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `}` token.
pub struct CloseCurly(Span);

impl CloseCurly {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `@foo`.
pub struct AttributeName {
    span: Span,
    value: String,
}

impl AttributeName {
    pub fn new(span: Span, value: String) -> Self {
        Self { span, value }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// An unsigned integer literal (e.g. `123`).
pub struct UnsignedLiteral {
    span: Span,
    value: u64,
}

impl UnsignedLiteral {
    pub fn new(span: Span, value: u64) -> Self {
        Self { span, value }
    }
}

/// A floating point literal (e.g. `1.234`).
pub struct FloatLiteral {
    span: Span,
    value: f64,
}

impl FloatLiteral {
    pub fn new(span: Span, value: f64) -> Self {
        Self { span, value }
    }
}

/// A string literal (e.g. `"abc"`).
pub struct StringLiteral {
    span: Span,
    value: String,
}

impl StringLiteral {
    pub fn new(span: Span, value: String) -> Self {
        Self { span, value }
    }
}

/// An `struct` keyword.
pub struct StructKeyword(Span);

impl StructKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `class` keyword.
pub struct ClassKeyword(Span);

impl ClassKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `impl` keyword.
pub struct ImplKeyword(Span);

impl ImplKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }
}

/// An `fn` keyword.
pub struct FnKeyword(Span);

impl FnKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `let` keyword.
pub struct LetKeyword(Span);

impl LetKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `if` keyword.
pub struct IfKeyword(Span);

impl IfKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `is` keyword.
pub struct IsKeyword(Span);

impl IsKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `asm` keyword.
pub struct AsmKeyword(Span);

impl AsmKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An `null` keyword.
pub struct NullKeyword(Span);

impl NullKeyword {
    pub fn new(span: Span) -> Self {
        Self(span)
    }
}

/// An identifier.
pub struct Identifier {
    span: Span,
    value: String,
}

impl Identifier {
    pub fn new(span: Span, value: String) -> Self {
        Self { span, value }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

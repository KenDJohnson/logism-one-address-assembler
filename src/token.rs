use logos::Logos;
use std::fmt;

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Text => write!(f, ".text"),
            Self::Data => write!(f, ".data"),
            Self::Label => write!(f, ".label"),
            Self::Number => write!(f, ".number"),
            Self::NumLiteral(i) => write!(f, "{}", i),
            Self::LabelIdent(label) => write!(f, "{}", label),
            Self::Add => write!(f, "add"),
            Self::AddImmediate => write!(f, "addi"),
            Self::Subtract => write!(f, "sub"),
            Self::SubtractImmediate => write!(f, "subi"),
            Self::Multiply => write!(f, "mul"),
            Self::MultiplyImmediate => write!(f, "muli"),
            Self::Divide => write!(f, "div"),
            Self::DivideImmediate => write!(f, "divi"),
            Self::Remainder => write!(f, "rem"),
            Self::RemainderImmediate => write!(f, "remi"),
            Self::Shift => write!(f, "shift"),
            Self::And => write!(f, "and"),
            Self::AndImmediate => write!(f, "andi"),
            Self::BranchZero => write!(f, "beqz"),
            Self::Branch => write!(f, "br"),
            Self::ClearAc => write!(f, "clac"),
            Self::Store => write!(f, "stor"),
            Self::NoOp => write!(f, "noop"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token<'a> {
    // Sections
    #[token(".text")]
    Text,
    #[token(".data")]
    Data,
    #[token(".label")]
    Label,
    #[token(".number")]
    Number,

    #[regex("[0-9]+", |lex| i16::from_str_radix(lex.slice(), 10).ok(), priority=2)]
    #[regex("0x[0-9a-f]+", |lex| i16::from_str_radix(&lex.slice()[2..], 16).ok())]
    NumLiteral(i16),

    #[regex("[_a-zA-Z0-9]+")]
    LabelIdent(&'a str),

    // mnemonics
    #[token("add")]
    Add,
    #[token("addi")]
    AddImmediate,
    #[token("sub")]
    Subtract,
    #[token("subi")]
    SubtractImmediate,
    #[token("mul")]
    Multiply,
    #[token("muli")]
    MultiplyImmediate,
    #[token("div")]
    Divide,
    #[token("divi")]
    DivideImmediate,
    #[token("rem")]
    Remainder,
    #[token("remi")]
    RemainderImmediate,
    #[token("shift")]
    Shift,
    #[token("and")]
    And,
    #[token("andi")]
    AndImmediate,

    #[token("beqz")]
    BranchZero,
    #[token("br")]
    Branch,
    #[token("clac")]
    ClearAc,
    #[token("stor")]
    Store,
    #[token("noop")]
    NoOp,

    #[error]
    #[regex("[ \t\n\r]+", logos::skip)]
    #[regex("#.*", logos::skip)]
    Error,
}

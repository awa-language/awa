use ecow::EcoString;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Name { value: EcoString },
    IntLiteral { value: EcoString },
    FloatLiteral { value: EcoString },
    StringLiteral { value: EcoString },
    CharLiteral { value: EcoString },
    DiscardName { value: EcoString },
    LeftParenthesis,    // '('
    RightParenthesis,   // ')'
    LeftSquare,         // '['
    RightSquare,        // ']'
    LeftBrace,          // '{'
    RightBrace,         // '}'
    Plus,               // '+'
    Minus,              // '-'
    Asterisk,           // '*'
    Slash,              // '/'
    Less,               // '<'
    Greater,            // '>'
    LessEqual,          // '<='
    GreaterEqual,       // '>='
    Percent,            // '%'
    PlusFloat,          // '+.'
    MinusFloat,         // '-.'
    AsteriskFloat,      // '*.'
    SlashFloat,         // '/.'
    LessFloat,          // '<.'
    GreaterFloat,       // '>.'
    LessEqualFloat,     // '<=.'
    GreaterEqualFloat,  // '>=.'
    Concat,             // '<>'
    Colon,              // ':'
    Comma,              // ','
    Bang,               // '!'
    Equal,              // '='
    EqualEqual,         // '=='
    NotEqual,           // '!='
    Pipe,               // '|'
    PipePipe,           // '||'
    Ampersand,          // '&'
    AmpersandAmpersand, // '&&'
    LessLess,           // '<<'
    GreaterGreater,     // '>>'
    Dot,                // '.'
    Comment,            // '//'
    EndOfFile,          // 'EOF'
    NewLine,            // 'NEWLINE'
    Int,                // 'int'
    Float,              // 'float'
    Char,               // 'char'
    String,             // 'string'
    Var,                // 'var'
    Struct,             // 'struct'
    If,                 // 'if'
    Else,               // 'else'
    Func,               // 'func'
    For,                // 'for'
    While,              // 'while'
    Return,             // 'return'
    Exit,               // 'exit'
    Panic,              // 'panic'
    Todo,               // 'todo'
}

impl Token {}

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Name { value }
            | Self::IntLiteral { value }
            | Self::FloatLiteral { value }
            | Self::CharLiteral { value }
            | Self::DiscardName { value }
            | Self::StringLiteral { value } => value.as_str(),
            Self::Ampersand => "&",
            Self::AmpersandAmpersand => "&&",
            Self::Bang => "!",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Comment => "//",
            Self::Dot => ".",
            Self::If => "if",
            Self::Else => "else",
            Self::NewLine => "NEWLINE",
            Self::EndOfFile => "EOF",
            Self::Equal => "=",
            Self::EqualEqual => "==",
            Self::Func => "func",
            Self::Greater => ">",
            Self::GreaterFloat => ">.",
            Self::GreaterEqual => ">=",
            Self::GreaterEqualFloat => ">=.",
            Self::GreaterGreater => ">>",
            Self::LeftBrace => "{",
            Self::LeftParenthesis => "(",
            Self::LeftSquare => "[",
            Self::Less => "<",
            Self::LessFloat => "<.",
            Self::LessEqual => "<=",
            Self::LessEqualFloat => "<=.",
            Self::Var => "var",
            Self::Struct => "var",
            Self::Concat => "<>",
            Self::LessLess => "<<",
            Self::Minus => "-",
            Self::MinusFloat => "-.",
            Self::NotEqual => "!=",
            Self::Panic => "panic",
            Self::Exit => "exit",
            Self::Return => "return",
            Self::For => "for",
            Self::While => "while",
            Self::Percent => "%",
            Self::Plus => "+",
            Self::PlusFloat => "+.",
            Self::RightBrace => "}",
            Self::RightParenthesis => ")",
            Self::RightSquare => "]",
            Self::Slash => "/",
            Self::SlashFloat => "/.",
            Self::Asterisk => "*",
            Self::AsteriskFloat => "*.",
            Self::Todo => "todo",
            Self::Pipe => "|",
            Self::PipePipe => "||",
            Self::Int => "int",
            Self::Float => "float",
            Self::Char => "char",
            Self::String => "string",
        };

        write!(formatter, "`{str}`")
    }
}

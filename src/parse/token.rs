use ecow::EcoString;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Name { value: EcoString },
    IntLiteral { value: EcoString },
    FloatLiteral { value: EcoString },
    StringLiteral { value: EcoString },
    CharLiteral { value: EcoString },
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
    PlusPlus,           // '++'
    MinusMinus,         // '--'
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
    Int32,              // 'int32'
    Int64,              // 'int64'
    Char,               // 'char'
    String,             // 'string'
    Var,                // 'var'
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Token::Name { value }
            | Token::IntLiteral { value }
            | Token::FloatLiteral { value }
            | Token::CharLiteral { value }
            | Token::StringLiteral { value } => value.as_str(),
            Token::Ampersand => "&",
            Token::AmpersandAmpersand => "&&",
            Token::Bang => "!",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Comment => "//",
            Token::Dot => ".",
            Token::If => "if",
            Token::Else => "else",
            Token::NewLine => "NEWLINE",
            Token::EndOfFile => "EOF",
            Token::Equal => "=",
            Token::EqualEqual => "==",
            Token::Func => "func",
            Token::Greater => ">",
            Token::GreaterFloat => ">.",
            Token::GreaterEqual => ">=",
            Token::GreaterEqualFloat => ">=.",
            Token::GreaterGreater => ">>",
            Token::LeftBrace => "{",
            Token::LeftParenthesis => "(",
            Token::LeftSquare => "[",
            Token::Less => "<",
            Token::LessFloat => "<.",
            Token::LessEqual => "<=",
            Token::LessEqualFloat => "<=.",
            Token::Var => "var",
            Token::Concat => "<>",
            Token::LessLess => "<<",
            Token::Minus => "-",
            Token::MinusFloat => "-.",
            Token::MinusMinus => "--",
            Token::NotEqual => "!=",
            Token::Panic => "panic",
            Token::Exit => "exit",
            Token::Return => "return",
            Token::For => "for",
            Token::While => "while",
            Token::Percent => "%",
            Token::Plus => "+",
            Token::PlusFloat => "+.",
            Token::PlusPlus => "++",
            Token::RightBrace => "}",
            Token::RightParenthesis => ")",
            Token::RightSquare => "]",
            Token::Slash => "/",
            Token::SlashFloat => "/.",
            Token::Asterisk => "*",
            Token::AsteriskFloat => "*.",
            Token::Todo => "todo",
            Token::Pipe => "|",
            Token::PipePipe => "||",
            Token::Int32 => "int32",
            Token::Int64 => "int64",
            Token::Char => "char",
            Token::String => "string",
        };
        write!(f, "`{s}`")
    }
}

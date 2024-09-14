use crate::parse::error::{LexicalError, LexicalErrorType};
use crate::parse::token::Token;
use std::char;

#[derive(Debug)]
pub struct Lexer<T: Iterator<Item = (u32, char)>> {
    input_chars: T,
    pending_tokens: Vec<TokenSpan>,
    current_char: Option<char>,
    current_location: u32,
}

#[derive(Debug)]
pub struct TokenSpan {
    _start: u32,
    token: Token,
    _end: u32,
}

pub type LexResult = Result<TokenSpan, LexicalError>;

impl<T> Lexer<T>
where
    T: Iterator<Item = (u32, char)>,
{
    pub fn new(input: T) -> Self {
        let mut lexer = Lexer {
            input_chars: input,
            pending_tokens: Vec::new(),
            current_char: None,
            current_location: 0,
        };

        let _ = lexer.advance_char();
        let _ = lexer.advance_char();

        lexer
    }

    fn advance_char(&mut self) -> Option<char> {
        let char = self.current_char;

        match self.input_chars.next() {
            Some((location, ch)) => {
                self.current_location = location;
                self.current_char = Some(ch);
            }
            None => {
                self.current_char = None;
            }
        }

        char
    }

    fn inner_next(&mut self) -> LexResult {
        while self.pending_tokens.is_empty() {
            self.consume()?;
        }

        Ok(self.pending_tokens.remove(0))
    }

    fn emit(&mut self, token_span: TokenSpan) {
        self.pending_tokens.push(token_span);
    }

    // TODO: add negative numbers support
    fn consume(&mut self) -> Result<(), LexicalError> {
        if let Some(ch) = self.current_char {
            if self.is_name_start(ch) {
                self.lex_name()?;
            } else if self.is_number_start(ch) {
                self.lex_number()?;
            } else {
                self.consume_character(ch)?;
            }
        } else {
            self.emit(TokenSpan {
                _start: self.current_location,
                token: Token::EndOfFile,
                _end: self.current_location,
            });
        }

        Ok(())
    }

    fn consume_character(&mut self, ch: char) -> Result<(), LexicalError> {
        match ch {
            '"' => self.lex_string(),
            '\'' => self.lex_char(),
            '=' => self.lex_equal(),
            '+' => self.lex_plus(),
            '-' => self.lex_minus(),
            '*' => self.lex_asterisk(),
            '/' => self.lex_slash(),
            '%' => self.lex_single_char(Token::Percent),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            '!' => self.lex_bang(),
            '(' => self.lex_single_char(Token::LeftParenthesis),
            ')' => self.lex_single_char(Token::RightParenthesis),
            '{' => self.lex_single_char(Token::LeftBrace),
            '}' => self.lex_single_char(Token::RightBrace),
            '[' => self.lex_single_char(Token::LeftSquare),
            ']' => self.lex_single_char(Token::RightSquare),
            ':' => self.lex_single_char(Token::Colon),
            '<' => self.lex_less(),
            '>' => self.lex_greater(),
            '.' => self.lex_single_char(Token::Dot),
            ',' => self.lex_single_char(Token::Comma),
            '\n' | ' ' | '\t' | '\x0c' => self.lex_newline(ch),
            ch => {
                return Err(LexicalError {
                    error: LexicalErrorType::UnrecognizedToken { tok: ch },
                });
            }
        }
    }

    fn lex_name(&mut self) -> Result<(), LexicalError> {
        let mut name = String::new();
        let start_location = self.current_location;

        while self.is_name_continuation() {
            name.push(self.advance_char().expect("lex_name continuation"))
        }

        let end_location = self.current_location;

        let token = match to_keyword(&name) {
            Some(tok) => tok,
            None => Token::Name { name: name.into() },
        };

        self.emit(TokenSpan {
            _start: start_location,
            token,
            _end: end_location,
        });

        Ok(())
    }

    fn lex_number(&mut self) -> Result<(), LexicalError> {
        let start_location = self.current_location;
        let mut number = String::new();

        let mut has_floating_point = false;

        loop {
            match self.advance_char() {
                Some(ch) if ch.is_digit(10) => number.push(ch),
                Some('.') => {
                    if number.len() == 0 || has_floating_point {
                        return Err(LexicalError {
                            error: LexicalErrorType::InvalidNumberFormat,
                        });
                    }

                    has_floating_point = true;
                    number.push('.');
                }
                Some(_) => break,
                None => {
                    return Err(LexicalError {
                        error: LexicalErrorType::UnexpectedNumberEnd,
                    });
                }
            };
        }

        if number.is_empty() {
            return Err(LexicalError {
                error: LexicalErrorType::InvalidNumberFormat,
            });
        }

        let token = if has_floating_point {
            Token::IntLiteral {
                value: number.into(),
            }
        } else {
            Token::FloatLiteral {
                value: number.into(),
            }
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            _start: start_location,
            token,
            _end: end_location,
        });

        Ok(())
    }

    fn lex_char(&mut self) -> Result<(), LexicalError> {
        let start_location = self.current_location;

        let _ = self.advance_char();
        let mut string = String::new();

        loop {
            match self.advance_char() {
                Some('\'') => break,
                Some('\\') => {
                    self.lex_escape_character(&mut string)?;
                }
                Some(ch) => string.push(ch),
                None => {
                    return Err(LexicalError {
                        error: LexicalErrorType::UnexpectedCharEnd,
                    });
                }
            }
        }

        let token = Token::CharLiteral {
            value: string.into(),
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            _start: start_location,
            token,
            _end: end_location,
        });

        Ok(())
    }

    fn lex_string(&mut self) -> Result<(), LexicalError> {
        let start_location = self.current_location;

        let _ = self.advance_char();
        let mut string = String::new();

        loop {
            match self.advance_char() {
                Some('"') => break,
                Some('\\') => {
                    self.lex_escape_character(&mut string)?;
                }
                Some(ch) => string.push(ch),
                None => {
                    return Err(LexicalError {
                        error: LexicalErrorType::UnexpectedStringEnd,
                    });
                }
            }
        }

        let token = Token::StringLiteral {
            value: string.into(),
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            _start: start_location,
            token,
            _end: end_location,
        });

        Ok(())
    }

    fn lex_escape_character(&mut self, string: &mut String) -> Result<(), LexicalError> {
        if let Some(ch) = self.current_char {
            match ch {
                'f' | 'n' | 'r' | 't' | '"' | '\\' => {
                    let _ = self.advance_char();
                    string.push('\\');
                    string.push(ch);
                }
                'u' => self.lex_unicode_escape(string)?,
                _ => {
                    return Err(LexicalError {
                        error: LexicalErrorType::BadEscapeCharacter,
                    })
                }
            }
        } else {
            return Err(LexicalError {
                error: LexicalErrorType::BadEscapeCharacter,
            });
        }

        Ok(())
    }

    fn lex_unicode_escape(&mut self, string: &mut String) -> Result<(), LexicalError> {
        let _ = self.advance_char();

        if Some('{') != self.current_char {
            return Err(LexicalError {
                error: LexicalErrorType::InvalidUnicodeEscape,
            });
        }

        let hex_digits = self.read_hex_digits()?;

        if Some('}') != self.current_char {
            return Err(LexicalError {
                error: LexicalErrorType::InvalidUnicodeEscape,
            });
        }

        let _ = self.advance_char();

        if !(1..=6).contains(&hex_digits.len()) {
            return Err(LexicalError {
                error: LexicalErrorType::InvalidUnicodeEscape,
            });
        }

        if char::from_u32(
            u32::from_str_radix(&hex_digits, 16)
                .expect("Cannot parse codepoint number in Unicode escape"),
        )
        .is_none()
        {
            return Err(LexicalError {
                error: LexicalErrorType::InvalidUnicodeEscape,
            });
        }

        string.push_str("\\u{");
        string.push_str(&hex_digits);
        string.push('}');

        Ok(())
    }

    fn read_hex_digits(&mut self) -> Result<String, LexicalError> {
        let mut hex_digits = String::new();

        loop {
            self.advance_char();

            let Some(chr) = self.current_char else {
                break;
            };

            if chr == '}' {
                break;
            }

            hex_digits.push(chr);

            if !chr.is_ascii_hexdigit() {
                return Err(LexicalError {
                    error: LexicalErrorType::InvalidUnicodeEscape,
                });
            }
        }

        Ok(hex_digits)
    }

    fn lex_equal(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('=') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                if let Some('=') = self.current_char {
                    return Err(LexicalError {
                        error: LexicalErrorType::InvalidTripleEqual,
                    });
                };

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::EqualEqual,
                    _end: token_end,
                });

                Ok(())
            }
            _ => {
                let token_end = self.current_location;
                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Equal,
                    _end: token_end,
                });

                Ok(())
            }
        }
    }

    fn lex_plus(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('+') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::PlusPlus,
                    _end: token_end,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::PlusFloat,
                    _end: token_end,
                });
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Plus,
                    _end: token_end,
                });
            }
        }

        Ok(())
    }

    fn lex_minus(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('-') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::MinusMinus,
                    _end: token_end,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::MinusFloat,
                    _end: token_end,
                });
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Minus,
                    _end: token_end,
                });
            }
        }

        Ok(())
    }

    fn lex_asterisk(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('.') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::AsteriskFloat,
                _end: token_end,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::Asterisk,
                _end: token_end,
            });
        }

        Ok(())
    }

    fn lex_slash(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::SlashFloat,
                    _end: token_end,
                });
            }
            Some('/') => {
                let _ = self.advance_char();

                self.lex_comment();
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Slash,
                    _end: token_end,
                });
            }
        }

        Ok(())
    }

    fn lex_comment(&mut self) {
        let start_location = self.current_location;

        while Some('\n') != self.current_char {
            match self.current_char {
                Some(_) => {
                    let _ = self.advance_char();
                }
                None => break,
            }
        }

        let end_location = self.current_location;

        self.emit(TokenSpan {
            _start: start_location,
            token: Token::Comment,
            _end: end_location,
        });
    }

    fn lex_ampersand(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('&') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::AmpersandAmpersand,
                _end: token_end,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::Ampersand,
                _end: token_end,
            });
        }

        Ok(())
    }

    fn lex_pipe(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('|') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::PipePipe,
                _end: token_end,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::Pipe,
                _end: token_end,
            });
        }

        Ok(())
    }

    fn lex_bang(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('=') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::NotEqual,
                _end: token_end,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                _start: token_start,
                token: Token::Bang,
                _end: token_end,
            });
        }

        Ok(())
    }

    fn lex_less(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('>') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Concat,
                    _end: token_end,
                });
            }
            Some('<') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::LessLess,
                    _end: token_end,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::LessFloat,
                    _end: token_end,
                });
            }
            Some('=') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                if let Some('.') = self.current_char {
                    self.emit(TokenSpan {
                        _start: token_start,
                        token: Token::LessEqualFloat,
                        _end: token_end,
                    });
                } else {
                    self.emit(TokenSpan {
                        _start: token_start,
                        token: Token::LessEqual,
                        _end: token_end,
                    });
                }
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Less,
                    _end: token_end,
                });
            }
        }

        Ok(())
    }

    fn lex_greater(&mut self) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('>') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::GreaterGreater,
                    _end: token_end,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::GreaterFloat,
                    _end: token_end,
                });
            }
            Some('=') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                if let Some('.') = self.current_char {
                    self.emit(TokenSpan {
                        _start: token_start,
                        token: Token::GreaterEqualFloat,
                        _end: token_end,
                    });
                } else {
                    self.emit(TokenSpan {
                        _start: token_start,
                        token: Token::GreaterEqual,
                        _end: token_end,
                    });
                }
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    _start: token_start,
                    token: Token::Greater,
                    _end: token_end,
                });
            }
        }

        Ok(())
    }

    fn lex_single_char(&mut self, t: Token) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char().expect("lex_single_char");
        let token_end = self.current_location;

        self.emit(TokenSpan {
            _start: token_start,
            token: t,
            _end: token_end,
        });

        Ok(())
    }

    fn lex_newline(&mut self, ch: char) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        let _ = self.advance_char();

        let token_end = self.current_location;

        if ch == '\n' {
            self.emit(TokenSpan {
                _start: token_start,
                token: Token::NewLine,
                _end: token_end,
            });
        }

        Ok(())
    }

    fn is_name_start(&mut self, ch: char) -> bool {
        matches!(ch, '_' | 'a'..='z' | 'A'..='Z')
    }

    fn is_name_continuation(&mut self) -> bool {
        match self.current_char {
            Some(ch) => matches!(ch, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'),
            None => false,
        }
    }

    fn is_number_start(&mut self, ch: char) -> bool {
        matches!(ch, '0'..='9')
    }
}

fn to_keyword(word: &str) -> Option<Token> {
    match word {
        "var" => Some(Token::Var),
        "for" => Some(Token::For),
        "while" => Some(Token::While),
        "func" => Some(Token::Func),
        "if" => Some(Token::If),
        "else" => Some(Token::Else),
        "return" => Some(Token::Return),
        "exit" => Some(Token::Exit),
        "panic" => Some(Token::Panic),
        "todo" => Some(Token::Todo),
        "int32" => Some(Token::Int32),
        "int64" => Some(Token::Int64),
        "char" => Some(Token::Char),
        "string" => Some(Token::String),
        _ => None,
    }
}

impl<T> Iterator for Lexer<T>
where
    T: Iterator<Item = (u32, char)>,
{
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner_next();

        match token {
            Ok(TokenSpan {
                _start: _,
                token: Token::EndOfFile,
                _end: _,
            }) => None,
            r => Some(r),
        }
    }
}

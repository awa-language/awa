use crate::lex::error::{LexicalError, Type};
use crate::lex::token::Token;
use itertools::{peek_nth, PeekNth};
use std::char;

use super::location::Location;
use super::newline_handler::NewlineHandler;

/// Lexes the input string into tokens.
///
/// # Panics
///
/// This function will panic if the input string length exceeds the maximum value
/// of `u32`, as it is the largest supported character count. This would only occur
/// if a single source file is approximately 4 GB in size, which is highly unlikely.
pub fn lex(input: &str) -> impl Iterator<Item = LexResult> + '_ {
    let chars = input
        .char_indices()
        .map(|(byte_index, char)| {
            let index = u32::try_from(byte_index).expect("Lex input string is too long");
            (index, char)
        })
        .collect::<Vec<_>>();

    let newline_handler = NewlineHandler::new(chars.into_iter());

    Lexer::new(newline_handler)
}

#[derive(Debug)]
pub struct Lexer<T: Iterator<Item = (u32, char)>> {
    input_chars: PeekNth<T>,
    pending_tokens: Vec<TokenSpan>,
    current_char: Option<char>,
    current_location: u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TokenSpan {
    pub start: u32,
    pub end: u32,
    pub token: Token,
}

pub type LexResult = Result<TokenSpan, LexicalError>;

impl<T> Lexer<T>
where
    T: Iterator<Item = (u32, char)>,
{
    pub fn new(input: T) -> Self {
        let mut lexer = Lexer {
            input_chars: peek_nth(input),
            pending_tokens: Vec::new(),
            current_char: None,
            current_location: 0,
        };

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

    fn peek_char(&mut self) -> Option<char> {
        if let Some((_, ch)) = self.input_chars.peek_nth(0) {
            Some(*ch)
        } else {
            None
        }
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

    fn consume(&mut self) -> Result<(), LexicalError> {
        if let Some(ch) = self.current_char {
            let mut check_for_binary_minus = false;

            if Self::is_name_start(ch) {
                self.lex_name();
                check_for_binary_minus = true;
            } else if self.is_number_start(ch) {
                self.lex_number()?;
                check_for_binary_minus = true;
            } else {
                self.consume_character(ch)?;
            }

            if check_for_binary_minus && Some('-') == self.current_char && self.is_number_start('-')
            {
                self.lex_single_char(Token::Minus);
            }
        } else {
            self.emit(TokenSpan {
                start: self.current_location,
                end: self.current_location,
                token: Token::EndOfFile,
            });
        }

        Ok(())
    }

    fn consume_character(&mut self, ch: char) -> Result<(), LexicalError> {
        let token_start = self.current_location;
        match ch {
            '"' => self.lex_string(),
            '\'' => self.lex_char(),
            '+' => {
                self.lex_plus();
                Ok(())
            }
            '-' => {
                self.lex_minus();
                Ok(())
            }
            '*' => {
                self.lex_asterisk();
                Ok(())
            }
            '/' => {
                self.lex_slash();
                Ok(())
            }
            '=' => self.lex_equal(),
            '&' => {
                self.lex_ampersand();
                Ok(())
            }
            '|' => {
                self.lex_pipe();
                Ok(())
            }
            '!' => {
                self.lex_bang();
                Ok(())
            }
            '<' => {
                self.lex_less();
                Ok(())
            }
            '>' => {
                self.lex_greater();
                Ok(())
            }
            '.' => {
                self.lex_single_char(Token::Dot);
                Ok(())
            }
            ',' => {
                self.lex_single_char(Token::Comma);
                Ok(())
            }
            '%' => {
                self.lex_single_char(Token::Percent);
                Ok(())
            }
            '(' => {
                self.lex_single_char(Token::LeftParenthesis);
                Ok(())
            }
            ')' => {
                self.lex_single_char(Token::RightParenthesis);
                Ok(())
            }
            '{' => {
                self.lex_single_char(Token::LeftBrace);
                Ok(())
            }
            '}' => {
                self.lex_single_char(Token::RightBrace);
                Ok(())
            }
            '[' => {
                self.lex_single_char(Token::LeftSquare);
                Ok(())
            }
            ']' => {
                self.lex_single_char(Token::RightSquare);
                Ok(())
            }
            ':' => {
                self.lex_single_char(Token::Colon);
                Ok(())
            }
            '\n' | ' ' | '\t' | '\x0c' => {
                self.lex_newline(ch);
                Ok(())
            }
            ch => Err(LexicalError {
                error: Type::UnrecognizedToken { token: ch },
                location: Location {
                    start: token_start,
                    end: token_start,
                },
            }),
        }
    }

    fn lex_name(&mut self) {
        let mut name = String::new();
        let start_location = self.current_location;

        while self.is_name_continuation() {
            name.push(self.advance_char().expect("lex_name continuation"));
        }

        let end_location = self.current_location;

        let token = match to_keyword(&name) {
            Some(tok) => tok,
            None => Token::Name { value: name.into() },
        };

        self.emit(TokenSpan {
            start: start_location,
            end: end_location,
            token,
        });
    }

    fn lex_number(&mut self) -> Result<(), LexicalError> {
        let start_location = self.current_location;
        let mut number = String::new();

        let mut has_floating_point = false;
        let mut last_is_digit = true;

        loop {
            match self.advance_char() {
                Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                    number.push(ch);
                    last_is_digit = true;
                }
                Some('.') => {
                    let end_location = self.current_location;
                    if number.is_empty() || has_floating_point {
                        return Err(LexicalError {
                            error: Type::InvalidNumberFormat,
                            location: Location {
                                start: start_location,
                                end: end_location,
                            },
                        });
                    }

                    has_floating_point = true;
                    last_is_digit = false;
                    number.push('.');
                }
                Some(_) => {
                    let end_location = self.current_location;
                    return Err(LexicalError {
                        error: Type::UnexpectedNumberEnd,
                        location: Location {
                            start: end_location,
                            end: end_location,
                        },
                    });
                }
                None => break,
            };
        }

        if number.is_empty() || !last_is_digit {
            let start_location = self.current_location;
            return Err(LexicalError {
                error: Type::InvalidNumberFormat,
                location: Location {
                    start: start_location,
                    end: start_location,
                },
            });
        }

        let token = if has_floating_point {
            Token::FloatLiteral {
                value: number.into(),
            }
        } else {
            Token::IntLiteral {
                value: number.into(),
            }
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            start: start_location,
            end: end_location,
            token,
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
                        error: Type::UnexpectedCharEnd,
                        location: Location {
                            start: start_location,
                            end: start_location,
                        },
                    });
                }
            }
        }

        let token = Token::CharLiteral {
            value: string.into(),
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            start: start_location,
            end: end_location,
            token,
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
                        error: Type::UnexpectedStringEnd,
                        location: Location {
                            start: start_location,
                            end: start_location,
                        },
                    });
                }
            }
        }

        let token = Token::StringLiteral {
            value: string.into(),
        };

        let end_location = self.current_location;

        self.emit(TokenSpan {
            start: start_location,
            end: end_location,
            token,
        });

        Ok(())
    }

    fn lex_escape_character(&mut self, string: &mut String) -> Result<(), LexicalError> {
        let start_location = self.current_location;
        if let Some(ch) = self.current_char {
            let end_location = self.current_location;
            match ch {
                'f' | 'n' | 'r' | 't' | '"' | '\\' => {
                    let _ = self.advance_char();
                    string.push('\\');
                    string.push(ch);
                }
                'u' => self.lex_unicode_escape(string)?,
                _ => {
                    return Err(LexicalError {
                        error: Type::BadEscapeCharacter,
                        location: Location {
                            start: start_location,
                            end: end_location,
                        },
                    })
                }
            }
        } else {
            return Err(LexicalError {
                error: Type::BadEscapeCharacter,
                location: Location {
                    start: start_location,
                    end: start_location,
                },
            });
        }

        Ok(())
    }

    fn lex_unicode_escape(&mut self, string: &mut String) -> Result<(), LexicalError> {
        let _ = self.advance_char();
        let start_location = self.current_location;

        if Some('{') != self.current_char {
            return Err(LexicalError {
                error: Type::InvalidUnicodeEscape,
                location: Location {
                    start: start_location,
                    end: start_location,
                },
            });
        }

        let hex_digits = self.read_hex_digits()?;

        if Some('}') != self.current_char {
            let end_location = self.current_location;
            return Err(LexicalError {
                error: Type::InvalidUnicodeEscape,
                location: Location {
                    start: start_location,
                    end: end_location,
                },
            });
        }

        let _ = self.advance_char();

        if !(1..=6).contains(&hex_digits.len()) {
            let end_location = self.current_location;

            return Err(LexicalError {
                error: Type::InvalidUnicodeEscape,
                location: Location {
                    start: start_location,
                    end: end_location,
                },
            });
        }

        if char::from_u32(
            u32::from_str_radix(&hex_digits, 16)
                .expect("Cannot parse codepoint number in Unicode escape"),
        )
        .is_none()
        {
            let end_location = self.current_location;

            return Err(LexicalError {
                error: Type::InvalidUnicodeEscape,
                location: Location {
                    start: start_location,
                    end: end_location,
                },
            });
        }

        string.push_str("\\u{");
        string.push_str(&hex_digits);
        string.push('}');

        Ok(())
    }

    fn read_hex_digits(&mut self) -> Result<String, LexicalError> {
        let mut hex_digits = String::new();
        let start_location = self.current_location;

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
                let end_location = self.current_location;
                return Err(LexicalError {
                    error: Type::InvalidUnicodeEscape,
                    location: Location {
                        start: start_location,
                        end: end_location,
                    },
                });
            }
        }

        Ok(hex_digits)
    }

    fn lex_equal(&mut self) -> Result<(), LexicalError> {
        let start_location = self.current_location;
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('=') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            if let Some('=') = self.current_char {
                let end_location = self.current_location;

                return Err(LexicalError {
                    error: Type::InvalidTripleEqual,
                    location: Location {
                        start: start_location,
                        end: end_location,
                    },
                });
            };

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::EqualEqual,
            });

            Ok(())
        } else {
            let token_end = self.current_location;
            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::Equal,
            });

            Ok(())
        }
    }

    fn lex_plus(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('+') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::PlusPlus,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::PlusFloat,
                });
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Plus,
                });
            }
        }
    }

    fn lex_minus(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('-') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::MinusMinus,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::MinusFloat,
                });
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Minus,
                });
            }
        }
    }

    fn lex_asterisk(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('.') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::AsteriskFloat,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::Asterisk,
            });
        }
    }

    fn lex_slash(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::SlashFloat,
                });
            }
            Some('/') => {
                let _ = self.advance_char();

                self.lex_comment();
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Slash,
                });
            }
        }
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
            start: start_location,
            end: end_location,
            token: Token::Comment,
        });
    }

    fn lex_ampersand(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('&') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::AmpersandAmpersand,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::Ampersand,
            });
        }
    }

    fn lex_pipe(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('|') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::PipePipe,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::Pipe,
            });
        }
    }

    fn lex_bang(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        if let Some('=') = self.current_char {
            let _ = self.advance_char();
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::NotEqual,
            });
        } else {
            let token_end = self.current_location;

            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::Bang,
            });
        }
    }

    fn lex_less(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('>') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Concat,
                });
            }
            Some('<') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::LessLess,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::LessFloat,
                });
            }
            Some('=') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                if let Some('.') = self.current_char {
                    self.emit(TokenSpan {
                        start: token_start,
                        end: token_end,
                        token: Token::LessEqualFloat,
                    });
                } else {
                    self.emit(TokenSpan {
                        start: token_start,
                        end: token_end,
                        token: Token::LessEqual,
                    });
                }
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Less,
                });
            }
        }
    }

    fn lex_greater(&mut self) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        match self.current_char {
            Some('>') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::GreaterGreater,
                });
            }
            Some('.') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::GreaterFloat,
                });
            }
            Some('=') => {
                let _ = self.advance_char();
                let token_end = self.current_location;

                if let Some('.') = self.current_char {
                    self.emit(TokenSpan {
                        start: token_start,
                        end: token_end,
                        token: Token::GreaterEqualFloat,
                    });
                } else {
                    self.emit(TokenSpan {
                        start: token_start,
                        end: token_end,
                        token: Token::GreaterEqual,
                    });
                }
            }
            _ => {
                let token_end = self.current_location;

                self.emit(TokenSpan {
                    start: token_start,
                    end: token_end,
                    token: Token::Greater,
                });
            }
        }
    }

    fn lex_single_char(&mut self, token: Token) {
        let token_start = self.current_location;
        let _ = self.advance_char().expect("lex_single_char");
        let token_end = self.current_location;

        self.emit(TokenSpan {
            start: token_start,
            end: token_end,
            token,
        });
    }

    fn lex_newline(&mut self, ch: char) {
        let token_start = self.current_location;
        let _ = self.advance_char();

        let token_end = self.current_location;

        if ch == '\n' {
            self.emit(TokenSpan {
                start: token_start,
                end: token_end,
                token: Token::NewLine,
            });
        }
    }

    fn is_name_start(ch: char) -> bool {
        matches!(ch, '_' | 'a'..='z' | 'A'..='Z')
    }

    fn is_name_continuation(&mut self) -> bool {
        match self.current_char {
            Some(ch) => matches!(ch, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'),
            None => false,
        }
    }

    fn is_number_start(&mut self, ch: char) -> bool {
        if ch.is_ascii_digit() {
            true
        } else if ch == '-' {
            match self.peek_char() {
                Some(ch) => ch.is_ascii_digit(),
                None => false,
            }
        } else {
            false
        }
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
        "int" => Some(Token::Int),
        "float" => Some(Token::Float),
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
                start: _,
                end: _,
                token: Token::EndOfFile,
            }) => None,
            result => Some(result),
        }
    }
}

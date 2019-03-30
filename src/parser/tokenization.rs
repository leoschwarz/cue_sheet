// cue_sheet
// Copyright (C) 2017  Leonardo Schwarz <mail@leoschwarz.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use errors::Error;
use parser::Time;

/// Any token as it can appear in a cue sheet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    /// A two digit long integer.
    Number(u32),

    /// Any string, notice commands and long numbers are all treated as String for the sake of this
    /// parser's implementation.
    String(String),

    /// A time (usually relative to the start of the file).
    Time(Time),
}

struct Reader {
    chars: Vec<char>,
    position: usize,
}

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

impl Reader {
    fn new(source: &str) -> Self {
        Reader {
            chars: source.chars().collect(),
            position: 0,
        }
    }

    /// True if there are still chars available to be read.
    fn available(&self) -> bool {
        self.chars.len() > self.position
    }

    fn peek(&self, n: usize) -> Result<String, Error> {
        if self.position + n <= self.chars.len() {
            Ok(self.chars[self.position..self.position + n]
                .into_iter()
                .collect())
        } else {
            Err("Tried to read out of bounds of reader.".into())
        }
    }

    fn take(&mut self, n: usize) -> Result<String, Error> {
        self.peek(n).map(|s| {
            self.position += n;
            s
        })
    }

    fn try_take_time(&mut self) -> Option<Time> {
        self.peek(8).ok().and_then(|s| s.parse().ok()).map(|time| {
            self.position += 8;
            time
        })
    }

    // notice that numbers can only be two digits long
    fn try_take_number(&mut self) -> Option<u32> {
        // Check if the next two chars are digits.
        let s = match self.peek(2) {
            Ok(s) => s,
            Err(_) => return None,
        };

        if s.chars()
            .map(|c| DIGITS.contains(&c))
            .fold(true, |old, new| old && new)
        {
            // Return a number if the third character is either whitespace or EOF.
            if let Ok(s3) = self.peek(3) {
                if !is_whitespace(s3.chars().nth(2).unwrap()) {
                    return None;
                }
            }

            // Parse the number.
            self.position += 3;
            Some(s.parse().unwrap())
        } else {
            None
        }
    }

    fn take_string(&mut self) -> Result<String, Error> {
        let mut result = Vec::new();

        // Check if string is quoted.
        let first = self.take(1)?.chars().next().unwrap();
        let is_quoted = first == '"';
        if !is_quoted {
            result.push(first);
        }

        // Now read as many chars as possible.
        while let Ok(next) = self.take(1) {
            let next = next.chars().next().unwrap();
            if next == '"' {
                if is_quoted {
                    return Ok(result.into_iter().collect());
                } else {
                    return Err("The `\"` char is not allowed in strings.".into());
                }
            } else if !is_quoted && is_whitespace(next) {
                break;
            } else {
                result.push(next);
            }
        }

        if is_quoted {
            Err("Opened string not closed until EOF.".into())
        } else {
            Ok(result.into_iter().collect())
        }
    }

    fn try_skip_whitespace(&mut self) {
        while let Ok(next) = self.peek(1) {
            let next = next.chars().next().unwrap();
            if is_whitespace(next) {
                self.position += 1;
            } else {
                return;
            }
        }
    }
}

/// Converts a string into a vector of tokens.
pub fn tokenize(source: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut reader = Reader::new(source);

    reader.try_skip_whitespace();
    while reader.available() {
        if let Some(time) = reader.try_take_time() {
            tokens.push(Token::Time(time));
        } else if let Some(num) = reader.try_take_number() {
            tokens.push(Token::Number(num));
        } else {
            tokens.push(Token::String(reader.take_string()?));
        }
        reader.try_skip_whitespace();
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_take_time() {
        let mut r1 = Reader::new("10:11:12");
        assert_eq!(r1.try_take_time(), Some(Time::new(10, 11, 12)));

        let mut r2 = Reader::new("10");
        assert_eq!(r2.try_take_time(), None);

        let mut r3 = Reader::new(" ");
        assert_eq!(r3.try_take_time(), None);
    }

    #[test]
    fn try_take_number() {
        let mut r1 = Reader::new("12");
        assert_eq!(r1.try_take_number(), Some(12));

        let mut r2 = Reader::new("xyz");
        assert_eq!(r2.try_take_number(), None);

        let mut r3 = Reader::new(" ");
        assert_eq!(r3.try_take_number(), None);
    }

    #[test]
    fn string_starting_with_num() {
        let mut r1 = Reader::new("860B640B");
        assert_eq!(r1.try_take_number(), None);
        assert_eq!(r1.take_string().unwrap(), "860B640B".to_string());
    }

    #[test]
    fn take_string() {
        let mut r1 = Reader::new("abc");
        assert_eq!(r1.take_string().unwrap(), "abc".to_string());

        let mut r2 = Reader::new("\"abc\"");
        assert_eq!(r2.take_string().unwrap(), "abc".to_string());
    }

    #[test]
    fn basic_types() {
        let source = r#"ABC 12 10:10:30 Abc"#;
        let tokens = tokenize(source).unwrap();

        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::String("ABC".to_string()));
        assert_eq!(tokens[1], Token::Number(12));
        assert_eq!(tokens[2], Token::Time(Time::new(10, 10, 30)));
        assert_eq!(tokens[3], Token::String("Abc".to_string()));
    }

    #[test]
    fn test_strings() {
        let source = r#"ABC "xyz xyz 12 10:10:30" " abc ""#;
        let tokens = tokenize(source).unwrap();

        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::String("ABC".to_string()));
        assert_eq!(tokens[1], Token::String("xyz xyz 12 10:10:30".to_string()));
        assert_eq!(tokens[2], Token::String(" abc ".to_string()));
    }
}

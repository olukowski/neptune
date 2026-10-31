//! Decoded code points and incremental newline preprocessing (HTML § parsing).

use std::collections::VecDeque;

/// A Unicode code point, including surrogates (unlike `char`). Not a byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodePoint(u32);

impl CodePoint {
    pub const fn new(value: u32) -> Option<Self> {
        if value <= 0x10ffff {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

impl From<char> for CodePoint {
    fn from(value: char) -> Self {
        Self(value as u32)
    }
}

/// One-based coordinates in the newline-normalized stream, counting code points.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub code: &'static str,
    pub position: Position,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Input {
    CodePoint(CodePoint),
    Eof,
    NeedMoreInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorError {
    InputAlreadyFinished,
    NoCurrentInput,
    AlreadyReconsuming,
}

/// Append-only decoded input. EOF is explicit; an empty chunk is not EOF.
///
/// CR becomes LF immediately; a following raw LF is ignored, even across chunks.
/// Diagnostics are generated once on first consumption, never on reconsumption.
#[derive(Debug)]
pub struct Cursor {
    pending: VecDeque<CodePoint>,
    finished: bool,
    skip_lf: bool,
    current: Option<(Input, Position)>,
    reconsume: bool,
    next_position: Position,
    errors: Vec<ParseError>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            finished: false,
            skip_lf: false,
            current: None,
            reconsume: false,
            next_position: Position { line: 1, column: 1 },
            errors: Vec::new(),
        }
    }
}

impl Cursor {
    pub fn push(&mut self, points: impl IntoIterator<Item = CodePoint>) -> Result<(), CursorError> {
        if self.finished {
            return Err(CursorError::InputAlreadyFinished);
        }
        self.pending.extend(points);
        Ok(())
    }

    /// Idempotent. No more code points may be appended afterwards.
    pub fn finish(&mut self) {
        self.finished = true;
    }

    pub fn current(&self) -> Option<(Input, Position)> {
        self.current
    }

    /// At EOF the position is just after the final normalized code point.
    pub fn position(&self) -> Position {
        self.current
            .map_or(self.next_position, |(_, position)| position)
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Reconsume the current code point or conceptual EOF, without advancing.
    pub fn reconsume(&mut self) -> Result<(), CursorError> {
        if self.current.is_none() {
            return Err(CursorError::NoCurrentInput);
        }
        if self.reconsume {
            return Err(CursorError::AlreadyReconsuming);
        }
        self.reconsume = true;
        Ok(())
    }

    pub fn consume(&mut self) -> Input {
        if self.reconsume {
            self.reconsume = false;
            // reconsume() establishes this invariant, with no mutation seam.
            return self.current.expect("reconsumption has current input").0;
        }
        loop {
            let Some(point) = self.pending.pop_front() else {
                if self.finished {
                    self.current = Some((Input::Eof, self.next_position));
                    return Input::Eof;
                }
                // A suspended consume retains the previous current character.
                return Input::NeedMoreInput;
            };
            if self.skip_lf {
                self.skip_lf = false;
                if point.value() == 0x0a {
                    continue;
                }
            }
            let point = if point.value() == 0x0d {
                self.skip_lf = true;
                CodePoint::from('\n')
            } else {
                point
            };
            let position = self.next_position;
            let value = point.value();
            let code = if (0xd800..=0xdfff).contains(&value) {
                Some("surrogate-in-input-stream")
            } else if (0xfdd0..=0xfdef).contains(&value) || value & 0xfffe == 0xfffe {
                Some("noncharacter-in-input-stream")
            } else if matches!(value, 0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f) {
                Some("control-character-in-input-stream")
            } else {
                None
            };
            if let Some(code) = code {
                self.errors.push(ParseError { code, position });
            }
            if value == 0x0a {
                self.next_position.line += 1;
                self.next_position.column = 1;
            } else {
                self.next_position.column += 1;
            }
            let input = Input::CodePoint(point);
            self.current = Some((input, position));
            return input;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspension_reconsumption_and_explicit_eof() {
        let mut cursor = Cursor::default();
        assert_eq!(cursor.reconsume(), Err(CursorError::NoCurrentInput));
        assert_eq!(cursor.consume(), Input::NeedMoreInput);
        assert_eq!(cursor.current(), None);
        cursor.push(['a'.into()]).unwrap();
        let a = Input::CodePoint('a'.into());
        assert_eq!(cursor.consume(), a);
        let current = cursor.current();
        assert_eq!(cursor.consume(), Input::NeedMoreInput);
        assert_eq!(cursor.current(), current);
        cursor.reconsume().unwrap();
        assert_eq!(cursor.reconsume(), Err(CursorError::AlreadyReconsuming));
        cursor.push(['b'.into()]).unwrap();
        assert_eq!(cursor.consume(), a);
        assert_eq!(cursor.current(), current);
        assert_eq!(cursor.consume(), Input::CodePoint('b'.into()));
        cursor.finish();
        cursor.finish();
        assert_eq!(cursor.consume(), Input::Eof);
        assert_eq!(cursor.position(), Position { line: 1, column: 3 });
        cursor.reconsume().unwrap();
        assert_eq!(cursor.consume(), Input::Eof);
        assert_eq!(cursor.consume(), Input::Eof);
        assert_eq!(cursor.push([]), Err(CursorError::InputAlreadyFinished));
    }

    #[test]
    fn all_chunk_boundaries_preserve_preprocessing_positions_and_diagnostics() {
        let points: Vec<_> = [
            0x61, 0x0d, 0x0a, 0x0d, 0x62, 0xd800, 0x1fffe, 0x0b, 0, 0x1f600, 0x0d,
        ]
        .map(|value| CodePoint::new(value).unwrap())
        .into();
        let collect = |chunks: Vec<&[CodePoint]>| {
            let mut cursor = Cursor::default();
            let mut consumed = Vec::new();
            for chunk in chunks {
                cursor.push(chunk.iter().copied()).unwrap();
                loop {
                    let input = cursor.consume();
                    if input == Input::NeedMoreInput {
                        break;
                    }
                    consumed.push(cursor.current().unwrap());
                    cursor.reconsume().unwrap();
                    assert_eq!(cursor.consume(), input);
                }
            }
            cursor.finish();
            assert_eq!(cursor.consume(), Input::Eof);
            (consumed, cursor.errors, cursor.next_position)
        };
        let expected = collect(vec![&points]);
        for split in 0..=points.len() {
            assert_eq!(collect(vec![&points[..split], &points[split..]]), expected);
        }
        assert_eq!(collect(points.chunks(1).collect()), expected);
        assert_eq!(
            expected
                .0
                .iter()
                .map(|(input, _)| match input {
                    Input::CodePoint(point) => point.value(),
                    _ => panic!("expected code point"),
                })
                .collect::<Vec<_>>(),
            [
                0x61, 0x0a, 0x0a, 0x62, 0xd800, 0x1fffe, 0x0b, 0, 0x1f600, 0x0a
            ]
        );
        assert_eq!(
            expected.1,
            vec![
                ParseError {
                    code: "surrogate-in-input-stream",
                    position: Position { line: 3, column: 2 }
                },
                ParseError {
                    code: "noncharacter-in-input-stream",
                    position: Position { line: 3, column: 3 }
                },
                ParseError {
                    code: "control-character-in-input-stream",
                    position: Position { line: 3, column: 4 }
                },
            ]
        );
        assert_eq!(
            expected.0.last().unwrap().1,
            Position { line: 3, column: 7 }
        );
        assert_eq!(expected.2, Position { line: 4, column: 1 });
        assert!(CodePoint::new(0x110000).is_none());
    }
}

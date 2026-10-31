//! Test-only html5lib adapter. Unsupported capabilities are explicit outcomes.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, ParseError, Position, Progress, State, Token, UnimplementedState,
};
use serde_json::Value;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub code: String,
}

impl From<&ParseError> for Diagnostic {
    fn from(error: &ParseError) -> Self {
        Self {
            line: error.position.line,
            column: error.position.column,
            code: error.code.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blocked {
    XmlCoercion,
    PendingSpecReview,
    TokenVariants,
    InitialState,
    FixtureCapability,
    Unimplemented(State),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Outcome {
    Passed,
    Failed(String),
    Blocked(Blocked),
}

pub struct Run {
    pub initial: State,
    pub context: Context,
}

pub enum Completion {
    Complete {
        tokens: Vec<Token>,
        errors: Vec<Diagnostic>,
    },
    Blocked(UnimplementedState),
    Incomplete,
}

/// Decode each literal \uHHHH independently: surrogate pairs are NOT combined.
/// This is the fixture convention, not JSON/UTF-16 decoding.
pub fn decode(text: &str, double_escaped: bool) -> Vec<CodePoint> {
    let mut result = Vec::new();
    let chars: Vec<_> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if double_escaped
            && chars[i] == '\\'
            && chars.get(i + 1) == Some(&'u')
            && let Some(digits) = chars.get(i + 2..i + 6)
            && digits.iter().all(|digit| digit.is_ascii_hexdigit())
        {
            let value = digits
                .iter()
                .fold(0, |value, digit| value * 16 + digit.to_digit(16).unwrap());
            result.push(CodePoint::new(value).unwrap());
            i += 6;
            continue;
        }
        result.push(chars[i].into());
        i += 1;
    }
    result
}

/// One run per initialStates entry, or Data when the field is absent.
pub fn initial_states(case: &Value) -> Vec<String> {
    match case.get("initialStates") {
        None => vec!["Data state".into()],
        Some(states) => {
            let states = states.as_array().expect("initialStates array");
            assert!(
                !states.is_empty(),
                "empty initialStates must not hide a case"
            );
            states
                .iter()
                .map(|state| state.as_str().expect("state string").into())
                .collect()
        }
    }
}

struct Prepared {
    run: Run,
    tokens: Vec<Token>,
    errors: Vec<Diagnostic>,
}

fn prepare(case: &Value, state: &str) -> Result<Prepared, Blocked> {
    // Unknown fields are a capability limitation, not silently ignored behavior.
    if case.as_object().expect("fixture object").keys().any(|key| {
        !matches!(
            key.as_str(),
            "description"
                | "input"
                | "output"
                | "initialStates"
                | "lastStartTag"
                | "doubleEscaped"
                | "errors"
        )
    }) {
        return Err(Blocked::FixtureCapability);
    }
    let initial = [
        State::Data,
        State::Plaintext,
        State::Rcdata,
        State::Rawtext,
        State::ScriptData,
        State::CdataSection,
    ]
    .into_iter()
    .find(|candidate| candidate.name() == state)
    .ok_or(Blocked::InitialState)?;
    let double_escaped = case
        .get("doubleEscaped")
        .map(|value| value.as_bool().expect("doubleEscaped bool"))
        .unwrap_or(false);
    let mut expected = Vec::new();
    for token in case["output"].as_array().expect("output array") {
        let fields = token.as_array().expect("token array");
        let kind = decode(fields[0].as_str().expect("token kind"), double_escaped);
        if kind == decode("Character", false) {
            assert_eq!(fields.len(), 2, "Character token shape");
            let data = decode(fields[1].as_str().expect("character data"), double_escaped);
            expected.extend(data.into_iter().map(Token::Character));
        } else if kind == decode("Comment", false) {
            assert_eq!(fields.len(), 2, "Comment token shape");
            expected.push(Token::Comment {
                data: decode(fields[1].as_str().expect("comment data"), double_escaped),
            });
        } else if kind == decode("StartTag", false) {
            assert!(matches!(fields.len(), 3 | 4), "StartTag token shape");
            let attributes = fields[2].as_object().expect("attributes object");
            let self_closing = fields
                .get(3)
                .map(|flag| flag.as_bool().expect("flag bool"))
                .unwrap_or(false);
            // Attributes remain unsupported. Compare the representable flag exactly.
            if !attributes.is_empty() {
                return Err(Blocked::TokenVariants);
            }
            expected.push(Token::StartTag {
                name: decode(fields[1].as_str().expect("tag name"), double_escaped),
                self_closing,
            });
        } else if kind == decode("EndTag", false) {
            assert_eq!(fields.len(), 2, "EndTag token shape");
            // Upstream EndTag output omits its flag; the trailing-solidus error
            // remains compared exactly. Preserve the flag in production storage.
            expected.push(Token::EndTag {
                name: decode(fields[1].as_str().expect("tag name"), double_escaped),
                self_closing: false,
            });
        } else {
            return Err(Blocked::TokenVariants);
        }
    }
    let errors = case
        .get("errors")
        .map(|errors| {
            errors
                .as_array()
                .expect("errors array")
                .iter()
                .map(|error| Diagnostic {
                    code: error["code"].as_str().expect("error code").into(),
                    line: error["line"]
                        .as_u64()
                        .filter(|value| *value > 0)
                        .expect("one-based line") as usize,
                    column: error["col"]
                        .as_u64()
                        .filter(|value| *value > 0)
                        .expect("one-based column") as usize,
                })
                .collect()
        })
        .unwrap_or_default();
    let mut context = Context {
        // lastStartTag is not part of the fixture's doubleEscaped convention.
        last_start_tag: case
            .get("lastStartTag")
            .map(|tag| decode(tag.as_str().expect("lastStartTag string"), false)),
        ..Context::default()
    };
    context
        .input
        .push(decode(
            case["input"].as_str().expect("input string"),
            double_escaped,
        ))
        .unwrap();
    context.input.finish();
    Ok(Prepared {
        run: Run { initial, context },
        tokens: expected,
        errors,
    })
}

/// Expected character strings expand to code points, equivalent to coalescing
/// adjacent character tokens without crossing tags or comments. Require exactly one terminal
/// EOF, omitted from fixture output. No truncated prefix can pass.
fn output(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut result = Vec::new();
    let mut eof = false;
    for token in tokens {
        if eof {
            return Err("output after EOF".into());
        }
        match token {
            Token::Eof => eof = true,
            // The upstream EndTag shape has no self-closing field.
            Token::EndTag { name, .. } => result.push(Token::EndTag {
                name,
                self_closing: false,
            }),
            other => result.push(other),
        }
    }
    if !eof {
        return Err("missing EOF token".into());
    }
    Ok(result)
}

pub fn run_with(
    case: &Value,
    state: &str,
    policy: Option<Blocked>,
    executor: impl FnOnce(Run) -> Completion,
) -> Outcome {
    if let Some(reason) = policy {
        return Outcome::Blocked(reason);
    }
    let Prepared {
        run,
        tokens: expected_tokens,
        errors: mut expected_errors,
    } = match prepare(case, state) {
        Ok(prepared) => prepared,
        Err(reason) => return Outcome::Blocked(reason),
    };
    match executor(run) {
        Completion::Blocked(error) => Outcome::Blocked(Blocked::Unimplemented(error.state)),
        Completion::Incomplete => {
            Outcome::Failed("need-more-input after closed fixture input".into())
        }
        Completion::Complete { tokens, mut errors } => {
            let tokens = match output(tokens) {
                Ok(tokens) => tokens,
                Err(error) => return Outcome::Failed(error),
            };
            // Compare diagnostic multisets: exact code/location/multiplicity,
            // without imposing an emission order (including at equal positions).
            errors.sort();
            expected_errors.sort();
            if tokens == expected_tokens && errors == expected_errors {
                Outcome::Passed
            } else {
                Outcome::Failed(format!(
                    "tokens: {tokens:?} != {expected_tokens:?}; errors: {errors:?} != {expected_errors:?}"
                ))
            }
        }
    }
}

pub fn execute(run: Run) -> Completion {
    let mut execution = Execution::new(run.initial, run.context);
    loop {
        match execution.step() {
            Err(error) => return Completion::Blocked(error),
            Ok(Progress::NeedMoreInput) => return Completion::Incomplete,
            Ok(Progress::Continue(_)) => {}
            Ok(Progress::Finished) => {
                let context = execution.context();
                return Completion::Complete {
                    tokens: context.tokens.clone(),
                    errors: context
                        .input
                        .errors()
                        .iter()
                        .chain(&context.errors)
                        .map(Diagnostic::from)
                        .collect(),
                };
            }
        }
    }
}

pub fn diagnostic(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

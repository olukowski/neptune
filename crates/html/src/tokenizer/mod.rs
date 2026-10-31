//! Partial tokenizer with explicit stubs for unsupported states.
//! See docs/tokenizer.md for current scope and pinned specification provenance.

mod before_attribute_name;
mod bogus_comment;
mod data;
mod end_tag_open;
mod input;
mod plaintext;
mod rawtext;
mod self_closing_start_tag;
mod state;
mod tag;
mod tag_name;
mod tag_open;

#[cfg(test)]
mod support_inventory;

pub use input::{CodePoint, Cursor, CursorError, Input, ParseError, Position};
pub use state::State;

/// Only emitted token variants needed by the implemented states.
/// Reachable emission paths have empty attributes; attribute creation leads to stubs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Character(CodePoint),
    StartTag {
        name: Vec<CodePoint>,
        self_closing: bool,
    },
    EndTag {
        name: Vec<CodePoint>,
        self_closing: bool,
    },
    Comment {
        data: Vec<CodePoint>,
    },
    Eof,
}

/// Unfinished start tag created by Tag open, not an emitted token.
///
/// Creation unsets the self-closing flag. Attributes start empty; the last
/// entry is the current attribute. No attribute-name/value algorithms run yet.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PendingStartTag {
    pub name: Vec<CodePoint>,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
}

/// Unfinished end tag created by End tag open, not an emitted token.
///
/// As with `PendingStartTag`, creation unsets the self-closing flag and the
/// attribute list starts empty; its last entry is the current attribute.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PendingEndTag {
    pub name: Vec<CodePoint>,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
}

/// A pending tag attribute, retaining code points (including non-scalars).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Attribute {
    pub name: Vec<CodePoint>,
    pub value: Vec<CodePoint>,
}

/// Selects the current tag independently of other retained pending storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TagKind {
    Start,
    End,
}

/// Unfinished comment created by End tag open's recovery branch, not emitted.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PendingComment {
    pub data: Vec<CodePoint>,
}

/// Minimal handler context. No byte decoding or tree-builder API is implied.
#[derive(Debug, Default)]
pub struct Context {
    pub input: Cursor,
    pub tokens: Vec<Token>,
    /// State-generated diagnostics; preprocessing diagnostics live on the cursor.
    pub errors: Vec<ParseError>,
    pub last_start_tag: Option<Vec<CodePoint>>,
    /// Set by states invoking Character reference; absent before any invocation.
    pub return_state: Option<State>,
    /// Selected by tag creation; cleared on emission or abandonment at EOF.
    /// Direct Tag name / Before attribute name / Self-closing start tag entry
    /// requires this and its matching pending tag.
    pub current_tag: Option<TagKind>,
    /// Created, not emitted; absent until Tag open's ASCII-alpha branch.
    pub pending_start_tag: Option<PendingStartTag>,
    /// Created, not emitted; absent until End tag open's ASCII-alpha branch.
    pub pending_end_tag: Option<PendingEndTag>,
    /// Created by End tag open's recovery; taken on Bogus comment emission.
    /// Direct Bogus comment entry requires an initialized pending comment.
    pub pending_comment: Option<PendingComment>,
    /// Code-point scratch storage; Tag open clears it on `?`.
    pub temporary_buffer: Vec<CodePoint>,
}

/// A source step completes before the destination is attempted on the next step.
/// Reconsumption, when required, is scheduled on `Context::input` by the source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Progress {
    Continue(State),
    NeedMoreInput,
    Finished,
}

/// A development limitation, never EOF or a recoverable HTML parse error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnimplementedState {
    pub state: State,
}

impl std::fmt::Display for UnimplementedState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unimplemented HTML tokenizer state: {}",
            self.state.name()
        )
    }
}

impl std::error::Error for UnimplementedState {}

/// The only production dispatch path. The support inventory is tested explicitly.
fn dispatch(state: State, context: &mut Context) -> Result<Progress, UnimplementedState> {
    match state {
        State::Data => Ok(data::step(context)),
        State::Plaintext => Ok(plaintext::step(context)),
        State::Rawtext => Ok(rawtext::step(context)),
        State::TagOpen => Ok(tag_open::step(context)),
        State::EndTagOpen => Ok(end_tag_open::step(context)),
        State::TagName => Ok(tag_name::step(context)),
        State::BeforeAttributeName => Ok(before_attribute_name::step(context)),
        State::SelfClosingStartTag => Ok(self_closing_start_tag::step(context)),
        State::BogusComment => Ok(bogus_comment::step(context)),
        _ => Err(UnimplementedState { state }),
    }
}

/// A stub is terminal: subsequent steps return the same error without executing.
/// Context remains available for inspection, but resuming a blocked execution
/// (even by reconstructing it) has no supported semantics.
#[derive(Debug)]
pub struct Execution {
    state: State,
    context: Context,
    terminal: Option<Result<Progress, UnimplementedState>>,
}

impl Execution {
    pub fn new(state: State, context: Context) -> Self {
        Self {
            state,
            context,
            terminal: None,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Feed incremental input through the cursor. Does not clear terminal status.
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn step(&mut self) -> Result<Progress, UnimplementedState> {
        self.step_with(dispatch)
    }

    fn step_with(
        &mut self,
        handler: impl FnOnce(State, &mut Context) -> Result<Progress, UnimplementedState>,
    ) -> Result<Progress, UnimplementedState> {
        if let Some(result) = self.terminal {
            return result;
        }
        let result = handler(self.state, &mut self.context);
        match result {
            Ok(Progress::Continue(target)) => self.state = target,
            Ok(Progress::NeedMoreInput) => {}
            Ok(Progress::Finished) | Err(_) => self.terminal = Some(result),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_source_effects_and_transition_are_inspectable_before_target_stub() {
        let mut context = Context::default();
        context.input.push(['x'.into()]).unwrap();
        let mut execution = Execution::new(State::Data, context);
        // Test-only executor, deliberately not a Data algorithm.
        let result = execution.step_with(|source, context| {
            assert_eq!(source, State::Data);
            assert_eq!(context.input.consume(), Input::CodePoint('x'.into()));
            context.tokens.push(Token::Character('!'.into()));
            context.errors.push(ParseError {
                code: "synthetic",
                position: context.input.position(),
            });
            context.input.reconsume().unwrap();
            Ok(Progress::Continue(State::AttributeName))
        });
        assert_eq!(result, Ok(Progress::Continue(State::AttributeName)));
        assert_eq!(execution.state(), State::AttributeName);
        assert_eq!(execution.context().tokens, [Token::Character('!'.into())]);
        assert_eq!(execution.context().errors.len(), 1);
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::AttributeName
            })
        );
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('x'.into())
        );
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::AttributeName
            })
        );
    }

    #[test]
    fn synthetic_suspension_can_resume_but_completion_is_terminal() {
        let mut execution = Execution::new(State::Data, Context::default());
        assert_eq!(
            execution.step_with(|_, context| {
                assert_eq!(context.input.consume(), Input::NeedMoreInput);
                Ok(Progress::NeedMoreInput)
            }),
            Ok(Progress::NeedMoreInput)
        );
        execution.context_mut().input.finish();
        assert_eq!(
            execution.step_with(|_, context| {
                assert_eq!(context.input.consume(), Input::Eof);
                context.tokens.push(Token::Eof);
                Ok(Progress::Finished)
            }),
            Ok(Progress::Finished)
        );
        assert_eq!(
            execution.step_with(|_, _| panic!("finished")),
            Ok(Progress::Finished)
        );
    }
}

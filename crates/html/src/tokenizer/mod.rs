//! HTML tokenizer state registry and explicit implementation boundary.
//!
//! This module deliberately has no input cursor, tokens, or parsing behavior
//! yet. Those interfaces should be introduced alongside their first tests.

mod state;

pub use state::State;

/// A development stub was reached, not an HTML parse error.
///
/// Callers must not turn this into EOF, an empty token sequence, or a successful
/// conformance result. Malformed HTML will eventually be handled by the
/// specification's recovery rules, separately from this error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnimplementedState {
    /// The state whose algorithm has not yet been implemented.
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

/// Placeholder for state dispatch; currently every state is an explicit stub.
///
/// This is a temporary development seam, not a functioning tokenizer step.
/// When an implementation is added, replace its dispatch arm with a handler.
/// Keep the fallback error for all remaining states.
pub fn dispatch(state: State) -> Result<(), UnimplementedState> {
    Err(UnimplementedState { state })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_is_explicitly_unimplemented() {
        for &state in State::ALL {
            assert_eq!(dispatch(state), Err(UnimplementedState { state }));
        }
    }

    #[test]
    fn stub_errors_identify_the_target_state() {
        let error = dispatch(State::TagOpen).unwrap_err();
        assert_eq!(
            error.to_string(),
            "unimplemented HTML tokenizer state: Tag open state"
        );
        let _: &dyn std::error::Error = &error;
    }
}

//! RCDATA less-than sign state only. Pinned HTML Standard revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source lines 143486–143502.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143486-L143502>

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::NeedMoreInput => Progress::NeedMoreInput,
        Input::CodePoint(point) if point.value() == 0x2f => {
            context.temporary_buffer.clear();
            Progress::Continue(State::RcdataEndTagOpen)
        }
        // Anything else includes conceptual EOF. Only the destination handles it.
        Input::CodePoint(_) | Input::Eof => {
            context.tokens.push(Token::Character('<'.into()));
            context
                .input
                .reconsume()
                .expect("RCDATA less-than sign consumed a current input item");
            Progress::Continue(State::Rcdata)
        }
    }
}

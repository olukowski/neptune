//! State names from the pinned HTML Standard; see docs/sources.toml.
//!
//! Listing a state here does not mean its algorithm is implemented.

macro_rules! states {
    ($($variant:ident => ($name:literal, $fragment:literal),)*) => {
        /// Tokenizer states in specification order at the pinned revision.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum State {
            $(
                #[doc = $name]
                $variant,
            )*
        }

        impl State {
            /// All states, including unimplemented ones, in specification order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The specification's human-readable state name.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }

            /// The fragment identifier on the HTML Standard's parsing page.
            pub const fn spec_fragment(self) -> &'static str {
                match self {
                    $(Self::$variant => $fragment,)*
                }
            }
        }
    };
}

states! {
    Data => ("Data state", "data-state"),
    Rcdata => ("RCDATA state", "rcdata-state"),
    Rawtext => ("RAWTEXT state", "rawtext-state"),
    ScriptData => ("Script data state", "script-data-state"),
    Plaintext => ("PLAINTEXT state", "plaintext-state"),
    TagOpen => ("Tag open state", "tag-open-state"),
    EndTagOpen => ("End tag open state", "end-tag-open-state"),
    TagName => ("Tag name state", "tag-name-state"),
    RcdataLessThanSign => ("RCDATA less-than sign state", "rcdata-less-than-sign-state"),
    RcdataEndTagOpen => ("RCDATA end tag open state", "rcdata-end-tag-open-state"),
    RcdataEndTagName => ("RCDATA end tag name state", "rcdata-end-tag-name-state"),
    RawtextLessThanSign => ("RAWTEXT less-than sign state", "rawtext-less-than-sign-state"),
    RawtextEndTagOpen => ("RAWTEXT end tag open state", "rawtext-end-tag-open-state"),
    RawtextEndTagName => ("RAWTEXT end tag name state", "rawtext-end-tag-name-state"),
    ScriptDataLessThanSign => ("Script data less-than sign state", "script-data-less-than-sign-state"),
    ScriptDataEndTagOpen => ("Script data end tag open state", "script-data-end-tag-open-state"),
    ScriptDataEndTagName => ("Script data end tag name state", "script-data-end-tag-name-state"),
    ScriptDataEscapeStart => ("Script data escape start state", "script-data-escape-start-state"),
    ScriptDataEscapeStartDash => ("Script data escape start dash state", "script-data-escape-start-dash-state"),
    ScriptDataEscaped => ("Script data escaped state", "script-data-escaped-state"),
    ScriptDataEscapedDash => ("Script data escaped dash state", "script-data-escaped-dash-state"),
    ScriptDataEscapedDashDash => ("Script data escaped dash dash state", "script-data-escaped-dash-dash-state"),
    ScriptDataEscapedLessThanSign => ("Script data escaped less-than sign state", "script-data-escaped-less-than-sign-state"),
    ScriptDataEscapedEndTagOpen => ("Script data escaped end tag open state", "script-data-escaped-end-tag-open-state"),
    ScriptDataEscapedEndTagName => ("Script data escaped end tag name state", "script-data-escaped-end-tag-name-state"),
    ScriptDataDoubleEscapeStart => ("Script data double escape start state", "script-data-double-escape-start-state"),
    ScriptDataDoubleEscaped => ("Script data double escaped state", "script-data-double-escaped-state"),
    ScriptDataDoubleEscapedDash => ("Script data double escaped dash state", "script-data-double-escaped-dash-state"),
    ScriptDataDoubleEscapedDashDash => ("Script data double escaped dash dash state", "script-data-double-escaped-dash-dash-state"),
    ScriptDataDoubleEscapedLessThanSign => ("Script data double escaped less-than sign state", "script-data-double-escaped-less-than-sign-state"),
    ScriptDataDoubleEscapeEnd => ("Script data double escape end state", "script-data-double-escape-end-state"),
    BeforeAttributeName => ("Before attribute name state", "before-attribute-name-state"),
    AttributeName => ("Attribute name state", "attribute-name-state"),
    AfterAttributeName => ("After attribute name state", "after-attribute-name-state"),
    BeforeAttributeValue => ("Before attribute value state", "before-attribute-value-state"),
    AttributeValueDoubleQuoted => ("Attribute value (double-quoted) state", "attribute-value-(double-quoted)-state"),
    AttributeValueSingleQuoted => ("Attribute value (single-quoted) state", "attribute-value-(single-quoted)-state"),
    AttributeValueUnquoted => ("Attribute value (unquoted) state", "attribute-value-(unquoted)-state"),
    AfterAttributeValueQuoted => ("After attribute value (quoted) state", "after-attribute-value-(quoted)-state"),
    SelfClosingStartTag => ("Self-closing start tag state", "self-closing-start-tag-state"),
    BogusComment => ("Bogus comment state", "bogus-comment-state"),
    MarkupDeclarationOpen => ("Markup declaration open state", "markup-declaration-open-state"),
    CommentStart => ("Comment start state", "comment-start-state"),
    CommentStartDash => ("Comment start dash state", "comment-start-dash-state"),
    Comment => ("Comment state", "comment-state"),
    CommentLessThanSign => ("Comment less-than sign state", "comment-less-than-sign-state"),
    CommentLessThanSignBang => ("Comment less-than sign bang state", "comment-less-than-sign-bang-state"),
    CommentLessThanSignBangDash => ("Comment less-than sign bang dash state", "comment-less-than-sign-bang-dash-state"),
    CommentLessThanSignBangDashDash => ("Comment less-than sign bang dash dash state", "comment-less-than-sign-bang-dash-dash-state"),
    CommentEndDash => ("Comment end dash state", "comment-end-dash-state"),
    CommentEnd => ("Comment end state", "comment-end-state"),
    CommentEndBang => ("Comment end bang state", "comment-end-bang-state"),
    Doctype => ("DOCTYPE state", "doctype-state"),
    BeforeDoctypeName => ("Before DOCTYPE name state", "before-doctype-name-state"),
    DoctypeName => ("DOCTYPE name state", "doctype-name-state"),
    AfterDoctypeName => ("After DOCTYPE name state", "after-doctype-name-state"),
    AfterDoctypePublicKeyword => ("After DOCTYPE public keyword state", "after-doctype-public-keyword-state"),
    BeforeDoctypePublicIdentifier => ("Before DOCTYPE public identifier state", "before-doctype-public-identifier-state"),
    DoctypePublicIdentifierDoubleQuoted => ("DOCTYPE public identifier (double-quoted) state", "doctype-public-identifier-(double-quoted)-state"),
    DoctypePublicIdentifierSingleQuoted => ("DOCTYPE public identifier (single-quoted) state", "doctype-public-identifier-(single-quoted)-state"),
    AfterDoctypePublicIdentifier => ("After DOCTYPE public identifier state", "after-doctype-public-identifier-state"),
    BetweenDoctypePublicAndSystemIdentifiers => ("Between DOCTYPE public and system identifiers state", "between-doctype-public-and-system-identifiers-state"),
    AfterDoctypeSystemKeyword => ("After DOCTYPE system keyword state", "after-doctype-system-keyword-state"),
    BeforeDoctypeSystemIdentifier => ("Before DOCTYPE system identifier state", "before-doctype-system-identifier-state"),
    DoctypeSystemIdentifierDoubleQuoted => ("DOCTYPE system identifier (double-quoted) state", "doctype-system-identifier-(double-quoted)-state"),
    DoctypeSystemIdentifierSingleQuoted => ("DOCTYPE system identifier (single-quoted) state", "doctype-system-identifier-(single-quoted)-state"),
    AfterDoctypeSystemIdentifier => ("After DOCTYPE system identifier state", "after-doctype-system-identifier-state"),
    BogusDoctype => ("Bogus DOCTYPE state", "bogus-doctype-state"),
    CdataSection => ("CDATA section state", "cdata-section-state"),
    CdataSectionBracket => ("CDATA section bracket state", "cdata-section-bracket-state"),
    CdataSectionEnd => ("CDATA section end state", "cdata-section-end-state"),
    ProcessingInstructionOpen => ("Processing instruction open state", "processing-instruction-open-state"),
    ProcessingInstructionTarget => ("Processing instruction target state", "processing-instruction-target-state"),
    AfterProcessingInstructionTarget => ("After processing instruction target state", "after-processing-instruction-target-state"),
    ProcessingInstructionData => ("Processing instruction data state", "processing-instruction-data-state"),
    ProcessingInstructionQuestionable => ("Processing instruction questionable state", "processing-instruction-questionable-state"),
    CharacterReference => ("Character reference state", "character-reference-state"),
    NamedCharacterReference => ("Named character reference state", "named-character-reference-state"),
    AmbiguousAmpersand => ("Ambiguous ampersand state", "ambiguous-ampersand-state"),
    NumericCharacterReference => ("Numeric character reference state", "numeric-character-reference-state"),
    HexadecimalCharacterReferenceStart => ("Hexadecimal character reference start state", "hexadecimal-character-reference-start-state"),
    HexadecimalCharacterReference => ("Hexadecimal character reference state", "hexadecimal-character-reference-state"),
    DecimalCharacterReference => ("Decimal character reference state", "decimal-character-reference-state"),
    NumericCharacterReferenceEnd => ("Numeric character reference end state", "numeric-character-reference-end-state"),
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn registry_matches_the_pinned_state_inventory() {
        // Update deliberately when reviewing a new specification baseline.
        assert_eq!(State::ALL.len(), 84);
        assert_eq!(State::ALL.first(), Some(&State::Data));
        assert_eq!(
            State::ALL.last(),
            Some(&State::NumericCharacterReferenceEnd)
        );

        let names: HashSet<_> = State::ALL.iter().map(|state| state.name()).collect();
        let fragments: HashSet<_> = State::ALL
            .iter()
            .map(|state| state.spec_fragment())
            .collect();
        assert_eq!(names.len(), State::ALL.len());
        assert_eq!(fragments.len(), State::ALL.len());
    }
}

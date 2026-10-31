//! Previously passing upstream runs, independent of dispatch and current results.

use std::collections::{BTreeMap, BTreeSet};

use crate::support::Outcome;

/// Pinned file, root group, zero-based case index, description, initial state,
/// zero-based initialStates entry (0 for implicit Data). Indices disambiguate
/// repeated descriptions and repeated initial states; descriptions aid review.
pub type RunId = (String, String, usize, String, String, usize);

pub fn passing_baseline() -> BTreeSet<RunId> {
    let entries: Vec<RunId> = [
        include_str!("../baselines/passing.json"),
        include_str!("../baselines/rawtext.json"),
        include_str!("../baselines/script_data.json"),
        include_str!("../baselines/script_data_less_than_sign.json"),
        include_str!("../baselines/script_data_end_tag_open.json"),
        include_str!("../baselines/script_data_end_tag_name.json"),
        include_str!("../baselines/script_data_escape_start.json"),
        include_str!("../baselines/rawtext_end_tag_open.json"),
        include_str!("../baselines/rawtext_end_tag_name.json"),
        include_str!("../baselines/rcdata.json"),
        include_str!("../baselines/rcdata_end_tag_open.json"),
        include_str!("../baselines/rcdata_end_tag_name.json"),
        include_str!("../baselines/after_attribute_name.json"),
        include_str!("../baselines/before_attribute_value.json"),
        include_str!("../baselines/integrated_attribute_values.json"),
    ]
    .into_iter()
    .flat_map(|source| serde_json::from_str::<Vec<RunId>>(source).expect("passing run identities"))
    .collect();
    let count = entries.len();
    let baseline: BTreeSet<_> = entries.into_iter().collect();
    assert!(!baseline.is_empty(), "empty passing baseline");
    assert_eq!(baseline.len(), count, "duplicate passing run identity");
    baseline
}

pub fn check_previous_passes(
    baseline: &BTreeSet<RunId>,
    outcomes: &BTreeMap<RunId, Outcome>,
) -> Result<(), String> {
    let regressions: Vec<_> = baseline
        .iter()
        .filter_map(|id| match outcomes.get(id) {
            Some(Outcome::Passed) => None,
            other => Some(format!("{id:?}: previously passed, now {other:?}")),
        })
        .collect();
    if regressions.is_empty() {
        Ok(())
    } else {
        Err(regressions.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::Blocked;
    use neptune_html::tokenizer::State;

    fn id(index: usize) -> RunId {
        (
            "synthetic.test".into(),
            "tests".into(),
            index,
            "same description".into(),
            "Data state".into(),
            0,
        )
    }

    #[test]
    fn synthetic_regressions_reject_blocked_failed_missing_and_substituted_passes() {
        let baseline = BTreeSet::from([id(0)]);
        for outcome in [
            Outcome::Blocked(Blocked::Unimplemented(State::AttributeName)),
            Outcome::Blocked(Blocked::TokenVariants),
            Outcome::Blocked(Blocked::XmlCoercion),
            Outcome::Failed("deliberate mismatch".into()),
        ] {
            let outcomes = BTreeMap::from([(id(0), outcome), (id(1), Outcome::Passed)]);
            assert!(check_previous_passes(&baseline, &outcomes).is_err());
        }
        let mut outcomes = BTreeMap::from([(id(1), Outcome::Passed)]);
        assert!(check_previous_passes(&baseline, &outcomes).is_err());
        outcomes.insert(id(0), Outcome::Passed);
        assert_eq!(check_previous_passes(&baseline, &outcomes), Ok(()));
        // A different initial state/occurrence cannot stand in for a prior pass.
        for field in [4, 5] {
            let mut different = id(0);
            if field == 4 {
                different.4 = "PLAINTEXT state".into();
            } else {
                different.5 = 1;
            }
            assert!(
                check_previous_passes(&baseline, &BTreeMap::from([(different, Outcome::Passed)]))
                    .is_err()
            );
        }
    }
}

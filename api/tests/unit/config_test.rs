use done_with_debt_api::config::hours_to_max_age_secs;

#[test]
fn hours_to_max_age_secs_computes_correctly() {
    assert_eq!(hours_to_max_age_secs(168), Some(604800));
    assert_eq!(hours_to_max_age_secs(0), Some(0));
    assert_eq!(hours_to_max_age_secs(1), Some(3600));
}

#[test]
fn hours_to_max_age_secs_returns_none_on_overflow() {
    // u64::MAX / 3600 = 5124095576030431; one more overflows
    assert_eq!(hours_to_max_age_secs(u64::MAX / 3600 + 1), None);
    assert_eq!(hours_to_max_age_secs(u64::MAX), None);
}

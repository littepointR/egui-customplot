use std::collections::BTreeSet;

#[test]
fn matlab_checklist_retains_every_frozen_id_and_official_function() {
    let checklist = include_str!("../../../docs/MATLAB_PLOT_CHECKLIST.md");
    for (prefix, count) in [("ML", 82), ("EX", 35), ("LG", 16)] {
        assert_eq!(
            checklist
                .lines()
                .filter(|line| line.starts_with(&format!("| {prefix}-")))
                .count(),
            count,
            "duplicate or missing {prefix} row"
        );
        let ids: BTreeSet<_> = checklist
            .lines()
            .filter_map(|line| {
                line.strip_prefix(&format!("| {prefix}-"))
                    .and_then(|s| s.split(" | ").next())
            })
            .collect();
        assert_eq!(ids.len(), count);
        for id in 1..=count {
            assert!(ids.contains(format!("{id:03}").as_str()));
        }
    }
    let snapshot = include_str!("../../../docs/MATLAB_PLOT_BASELINE.json");
    // Read the frozen, one-name-per-line array without adding a runtime JSON dependency.
    let names: BTreeSet<_> = snapshot
        .split("\"functions\": [")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .lines()
        .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
        .filter(|name| !name.is_empty())
        .collect();
    let actual: BTreeSet<_> = checklist
        .lines()
        .filter(|line| line.starts_with("| ML-"))
        .map(|line| line.split('`').nth(1).unwrap())
        .collect();
    assert_eq!(names.len(), 82);
    assert_eq!(actual, names);
}

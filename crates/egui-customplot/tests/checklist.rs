use std::collections::{BTreeMap, BTreeSet};

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

#[test]
fn matlab_status_summary_matches_all_133_rows_without_counting_toolboxes() {
    let checklist = include_str!("../../../docs/MATLAB_PLOT_CHECKLIST.md");
    let mut counts = BTreeMap::new();
    for row in checklist.lines().filter(|line| {
        ["| ML-", "| EX-", "| LG-"]
            .iter()
            .any(|prefix| line.starts_with(prefix))
    }) {
        let status = row.split('|').nth(2).unwrap().trim();
        assert!(
            ["未实现", "开发中", "待验收", "已交付"].contains(&status),
            "unknown status: {row}"
        );
        *counts.entry(status).or_insert(0usize) += 1;
    }
    assert_eq!(counts.values().sum::<usize>(), 133);
    let developing = counts.get("开发中").copied().unwrap_or(0);
    let unimplemented = counts.get("未实现").copied().unwrap_or(0);
    assert!(checklist.contains(&format!("共 {developing} 项“开发中”")));
    assert!(checklist.contains(&format!("其余 {unimplemented} 项未实现")));
    if counts.get("已交付").copied().unwrap_or(0) == 0 {
        assert!(checklist.contains("没有图类型通过完整交付门槛"));
    }
}

use egui::{FullOutput, Pos2, Shape};

pub fn paths(output: &FullOutput) -> Vec<Vec<Pos2>> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(path) => Some(path.points.clone()),
            _ => None,
        })
        .collect()
}

pub fn assert_axis_aligned(paths: &[Vec<Pos2>]) {
    for path in paths {
        for pair in path.windows(2) {
            assert!(
                pair[0].x == pair[1].x || pair[0].y == pair[1].y,
                "diagonal step: {pair:?}"
            );
        }
    }
}

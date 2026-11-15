use super::*;

fn total_area(vertices: &[PlotPoint], tris: &[[usize; 3]]) -> f64 {
    tris.iter()
        .map(|&[i, j, k]| cross(vertices[i], vertices[j], vertices[k]).abs() * 0.5)
        .sum()
}

#[test]
fn triangulates_convex_concave_and_cw_rings_with_exact_area() {
    let l_shape = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 1.0],
        [1.0, 1.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ];
    let outer: Vec<_> = (0..l_shape.len()).collect();
    let tris = triangulate_indexed_polygon(&l_shape, &outer, &[], [AxisScale::Linear; 2]).unwrap();
    assert_eq!(tris.len(), 4);
    assert!((total_area(&l_shape, &tris) - 7.0).abs() < 1e-12);

    let cw: Vec<_> = l_shape.iter().copied().rev().collect();
    let tris_cw = triangulate_indexed_polygon(&cw, &outer, &[], [AxisScale::Linear; 2]).unwrap();
    assert!((total_area(&cw, &tris_cw) - 7.0).abs() < 1e-12);
}

#[test]
fn triangulates_polygon_with_multiple_holes_preserving_net_area() {
    let mut vertices = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    let hole1 = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
    let hole2 = vec![[6.0, 5.0], [8.0, 5.0], [8.0, 8.0], [6.0, 8.0]];
    let h1_idx: Vec<_> = (4..8).collect();
    let h2_idx: Vec<_> = (8..12).collect();
    vertices.extend(hole1);
    vertices.extend(hole2);

    let tris = triangulate_indexed_polygon(
        &vertices,
        &[0, 1, 2, 3],
        &[&h1_idx, &h2_idx],
        [AxisScale::Linear; 2],
    )
    .unwrap();
    // Net area = 100 - 4 - 6 = 90
    assert!((total_area(&vertices, &tris) - 90.0).abs() < 1e-10);
    // No triangle may cover the center of either hole.
    for hole_center in [[2.0, 2.0], [7.0, 6.5]] {
        assert!(tris.iter().all(|&[i, j, k]| !point_in_tri(
            hole_center,
            vertices[i],
            vertices[j],
            vertices[k]
        )));
    }
}

#[test]
fn rejects_degenerate_or_log_invalid_rings() {
    let line = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    assert!(triangulate_indexed_polygon(&line, &[0, 1, 2], &[], [AxisScale::Linear; 2]).is_none());
    let neg = vec![[-1.0, 1.0], [2.0, 1.0], [2.0, 2.0]];
    assert!(
        triangulate_indexed_polygon(&neg, &[0, 1, 2], &[], [AxisScale::Log10, AxisScale::Linear])
            .is_none()
    );
}

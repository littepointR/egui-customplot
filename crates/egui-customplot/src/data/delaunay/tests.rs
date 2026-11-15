use super::{AlphaShapeData, TriMesh2D, VoronoiData, bowyer::in_circumcircle};
use crate::PlotBounds;

#[test]
fn delaunay_triangles_satisfy_empty_circumcircle_and_deduplicated_edges() {
    let sites = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        [0.0, 3.0],
        [2.0, 1.5],
        [1.0, 0.8],
        [3.1, 2.2],
    ];
    let mesh = TriMesh2D::delaunay(&sites).unwrap();
    assert_eq!(mesh.vertices().len(), sites.len());
    // Planar graph with 7 vertices and 4 hull edges has 2*7 - 2 - 4 = 8 or 2*n - 2 - h triangles.
    assert!(!mesh.triangles().is_empty());
    assert!(mesh.edges().len() < mesh.triangles().len() * 3);
    for &t in mesh.triangles() {
        let [a, b, c] = t.map(|i| mesh.vertices()[i as usize]);
        for (vi, &p) in mesh.vertices().iter().enumerate() {
            if !t.contains(&(vi as u32)) {
                assert!(!in_circumcircle(a, b, c, p), "point {p:?} inside circle");
            }
        }
    }
}

#[test]
fn voronoi_handles_collinear_and_duplicate_sites() {
    let bounds = PlotBounds::new([-2.0, -2.0], [6.0, 4.0]).unwrap();
    let sites = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 0.0], [4.0, 0.0]];
    let vor = VoronoiData::from_sites(&sites, bounds).unwrap();
    assert_eq!(vor.duplicate_sites(), 1);
    assert_eq!(vor.sites().len(), 3);
    assert_eq!(vor.edges().len(), 2);
}

#[test]
fn alpha_shape_small_alpha_carves_inner_hole() {
    let mut ring = Vec::new();
    for k in 0..24 {
        let a = std::f64::consts::TAU * (k as f64 / 24.0);
        ring.push([2.0 * a.cos(), 2.0 * a.sin()]);
        ring.push([1.0 * a.cos(), 1.0 * a.sin()]);
    }
    let shape = AlphaShapeData::from_sites(&ring, 0.85).unwrap();
    assert!(shape.patch().face_count() > 0);
    // An annulus has both outer and inner boundary loops, strictly exceeding a simple disk.
    assert!(shape.boundary_edges().len() > 24);
}

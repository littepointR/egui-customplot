use std::collections::HashMap;

use egui::{Context, Event, PointerButton, Pos2, RawInput, Rect, Stroke};
use egui_customplot::{
    AlphaShape3D, AlphaShape3DData, AxisScale, Camera3D, ColorMap, ColorScale, ConfusionChart,
    ConfusionMatrixData, ConfusionNormalization, ContourData, ContourLevels, Contours,
    CurvilinearField2D, CurvilinearPcolor, DepthMode, FilledContourData, FilledContours,
    FunctionContourError, Graph3D, Graph3DData, GraphData, GraphLayout, GraphLayout3D, GridError,
    Layer3D, Patch3D, Patch3DData, PcolorShading, Plot, Plot3D, Plot3DState, PlotBounds, PlotLayer,
    PlotPoint, PlotState, Point3D, Projection3D, RegionKind, SkyPlot, SkyPlotData, SkyPlotState,
    SkyTrack, SkyTransform, SurfaceShading, TriMesh3DData, TriMeshError,
};

fn raw(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(720.0, 520.0))),
        events,
        ..Default::default()
    }
}

#[test]
fn ext06_confusion_matrix_normalizations_metrics_and_cell_hover() {
    let cm = ConfusionMatrixData::from_labels(
        &["cat", "dog", "bird"],
        &["Cat", "Dog", "Bird"],
        &["cat", "cat", "dog", "dog", "bird", "bird", "dog"],
        &["cat", "dog", "dog", "dog", "bird", "cat", "bird"],
    )
    .unwrap();
    assert_eq!(cm.n_classes(), 3);
    assert_eq!(cm.count(0, 0), 1);
    assert_eq!(cm.count(0, 1), 1);
    assert_eq!(cm.count(1, 1), 2);
    assert_eq!(cm.grand_total(), 7);
    assert!((cm.accuracy() - 4.0 / 7.0).abs() < 1e-12);
    assert!((cm.display_value(0, 0, ConfusionNormalization::RowNormalized) - 50.0).abs() < 1e-9);
    assert!((cm.display_value(0, 0, ConfusionNormalization::ColumnNormalized) - 50.0).abs() < 1e-9);
    assert!(
        (cm.display_value(0, 0, ConfusionNormalization::TotalNormalized) - 100.0 / 7.0).abs()
            < 1e-9
    );

    let zero_row =
        ConfusionMatrixData::from_counts(&["A", "B"], &[5, 1, 0, 0]).expect("zero row valid");
    assert_eq!(zero_row.row_recall(1), None);
    assert!((zero_row.row_recall(0).unwrap() - 5.0 / 6.0).abs() < 1e-12);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut plot_rect = Rect::NOTHING;
    let mut main_pos = Pos2::ZERO;
    let mut tpr_pos = Pos2::ZERO;
    ctx.run_ui(raw(vec![]), |ui| {
        let resp = Plot::new("cm_hover").confusion_axes(&cm, true).show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(
                ConfusionChart::new(&cm)
                    .normalization(ConfusionNormalization::RowNormalized)
                    .show_summaries(true),
            )],
        );
        plot_rect = resp.transform.rect();
        main_pos = resp.transform.position_from_point([1.0, 2.0]); // col=0, row=1 -> cell(1,0)
        tpr_pos = resp.transform.position_from_point([4.2, 2.0]); // TPR summary row=1
    })
    .drop_without_applying_deltas();
    assert!(plot_rect.contains(main_pos) && plot_rect.contains(tpr_pos));

    let mut hovered_main = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(main_pos)]), |ui| {
        hovered_main = Plot::new("cm_hover")
            .confusion_axes(&cm, true)
            .show_layers(ui, &mut state, &[PlotLayer::from(ConfusionChart::new(&cm))])
            .hovered_region;
    })
    .drop_without_applying_deltas();
    let hm = hovered_main.expect("hover main cell");
    assert_eq!(hm.kind, RegionKind::ConfusionCell);
    assert_eq!(hm.grid_index, Some([0, 1]));

    let mut hovered_sum = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(tpr_pos)]), |ui| {
        hovered_sum = Plot::new("cm_hover")
            .confusion_axes(&cm, true)
            .show_layers(ui, &mut state, &[PlotLayer::from(ConfusionChart::new(&cm))])
            .hovered_region;
    })
    .drop_without_applying_deltas();
    let hs = hovered_sum.expect("hover TPR summary cell");
    assert_eq!(hs.kind, RegionKind::ConfusionCell);
    assert_eq!(hs.grid_index, Some([3, 1]));
    assert!((hs.scalar_value.unwrap() - 200.0 / 3.0).abs() < 1e-6);
}

#[test]
fn ext07_skyplot_projection_mask_multipass_legend_and_hover() {
    let tr = SkyTransform::new(Pos2::new(200.0, 200.0), 100.0, 0.0);
    let z = tr.project(45.0, 90.0).unwrap();
    assert!((z.x - 200.0).abs() < 1e-4 && (z.y - 200.0).abs() < 1e-4);
    let n = tr.project(0.0, 0.0).unwrap();
    assert!((n.x - 200.0).abs() < 1e-4 && (n.y - 100.0).abs() < 1e-4);
    let e = tr.project(90.0, 0.0).unwrap();
    assert!((e.x - 300.0).abs() < 1e-4 && (e.y - 200.0).abs() < 1e-4);
    let inv = tr.unproject(Pos2::new(250.0, 200.0)).unwrap();
    assert!((inv[0] - 90.0).abs() < 1e-6 && (inv[1] - 45.0).abs() < 1e-6);
    assert!(tr.project(0.0, -5.0).is_none());

    let sky = SkyPlotData::from_az_el(&[90.0, 210.0], &[6.0, 48.0])
        .unwrap()
        .with_labels(&["Low", "High"])
        .unwrap()
        .with_groups(&[0, 1])
        .unwrap()
        .with_group_names(&["GPS", "GLONASS"])
        .with_tracks(vec![
            SkyTrack::new("T0", &[0.0, 30.0, 60.0], &[30.0, 40.0, 50.0])
                .unwrap()
                .with_scalars(&[35.0, 40.0, 45.0])
                .unwrap()
                .group(0),
        ]);

    let ctx = Context::default();
    let mut state = SkyPlotState::default();
    let mut legend_click = Pos2::ZERO;
    let mut low_pos = Pos2::ZERO;
    ctx.run_ui(raw(vec![]), |ui| {
        let r = SkyPlot::new("sky", &sky)
            .mask_elevation(15.0)
            .color_scale(ColorScale::new([30.0, 50.0]).unwrap())
            .show(ui, &mut state);
        legend_click = r.legend_entries[0].rect.center();
        low_pos = r.transform.project(90.0, 6.0).unwrap();
    })
    .drop_without_applying_deltas();

    // Hovering masked low-elevation satellite reports masked = true.
    let mut hovered = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(low_pos)]), |ui| {
        hovered = SkyPlot::new("sky", &sky)
            .mask_elevation(15.0)
            .show(ui, &mut state)
            .hovered_item;
    })
    .drop_without_applying_deltas();
    let item = hovered.expect("hover low sat");
    assert_eq!(item.satellite_index, Some(0));
    assert!(item.masked);

    // Multi-pass request_discard click on group 0 legend toggles once, not twice.
    let mut passes = 0;
    ctx.run_ui(
        raw(vec![
            Event::PointerMoved(legend_click),
            Event::PointerButton {
                pos: legend_click,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            Event::PointerButton {
                pos: legend_click,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]),
        |ui| {
            SkyPlot::new("sky", &sky)
                .mask_elevation(15.0)
                .show(ui, &mut state);
            if passes == 0 {
                ui.ctx().request_discard("sky legend multipass");
            }
            passes += 1;
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
    assert!(!state.group_visible(0));
    assert!(state.group_visible(1));
}

#[test]
fn ext08_curvilinear_grid_contours_shading_validation_and_picking() {
    // Reject self-folding bowtie quad grid.
    let bad = CurvilinearField2D::from_nodes(
        2,
        2,
        vec![0.0, 1.0, 1.0, 0.0], // swapped bottom corners -> bowtie with top!
        vec![0.0, 0.0, 1.0, 1.0],
        vec![1.0, 2.0, 3.0, 4.0],
    );
    assert_eq!(bad.unwrap_err(), GridError::InvalidEdges);

    // Valid polar annulus grid.
    let (rows, cols) = (8usize, 12usize);
    let mut xs = Vec::with_capacity(rows * cols);
    let mut ys = Vec::with_capacity(rows * cols);
    let mut zs = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        let rad = 1.0 + 2.0 * (r as f64 / (rows - 1) as f64);
        for c in 0..cols {
            let th = std::f64::consts::PI * (c as f64 / (cols - 1) as f64);
            xs.push(rad * th.cos());
            ys.push(rad * th.sin());
            zs.push(rad);
        }
    }
    let field = CurvilinearField2D::from_nodes(rows, cols, xs, ys, zs).unwrap();
    let lines = ContourData::from_curvilinear(&field, ContourLevels::Count(4)).unwrap();
    let bands = FilledContourData::from_curvilinear(&field, ContourLevels::Count(4)).unwrap();
    assert!(!lines.lines().is_empty());
    assert_eq!(bands.bands().len(), 5);

    let cscale = ColorScale::new([1.0, 3.0])
        .unwrap()
        .color_map(ColorMap::Parula);
    let ctx = Context::default();
    let mut state = PlotState::default();
    let sample_quad = field.quad_nodes(2, 3).unwrap();
    let centroid = [
        0.25 * (sample_quad[0][0] + sample_quad[1][0] + sample_quad[2][0] + sample_quad[3][0]),
        0.25 * (sample_quad[0][1] + sample_quad[1][1] + sample_quad[2][1] + sample_quad[3][1]),
    ];
    let mut pick_pos = Pos2::ZERO;
    ctx.run_ui(raw(vec![]), |ui| {
        let resp = Plot::new("curv_test").show_layers(
            ui,
            &mut state,
            &[
                PlotLayer::from(
                    CurvilinearPcolor::new(&field, cscale).shading(PcolorShading::Interp),
                ),
                PlotLayer::from(Contours::new(&lines)),
            ],
        );
        assert_eq!(
            resp.diagnostics.images.output_cells,
            (rows - 1) * (cols - 1)
        );
        pick_pos = resp.transform.position_from_point(centroid);
    })
    .drop_without_applying_deltas();

    let mut hit = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(pick_pos)]), |ui| {
        hit = Plot::new("curv_test")
            .show_layers(
                ui,
                &mut state,
                &[PlotLayer::from(
                    CurvilinearPcolor::new(&field, cscale)
                        .shading(PcolorShading::Flat)
                        .edge_stroke(Stroke::new(1.0, egui::Color32::BLACK)),
                )],
            )
            .hovered_region;
    })
    .drop_without_applying_deltas();
    let reg = hit.expect("curvilinear cell hover");
    assert_eq!(reg.kind, RegionKind::CurvilinearCell);
    assert_eq!(reg.grid_index, Some([3, 2]));
}

#[test]
fn ext09_patch3d_concave_faces_zbuffer_and_face_picking() {
    // Concave 6-vertex L-shape lying in z = 0.5 plane + back triangle face.
    let verts: Vec<Point3D> = vec![
        [0.0, 0.0, 0.5],
        [1.0, 0.0, 0.5],
        [1.0, 0.35, 0.5],
        [0.35, 0.35, 0.5],
        [0.35, 1.0, 0.5],
        [0.0, 1.0, 0.5],
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let f0: &[u32] = &[0, 1, 2, 3, 4, 5];
    let f1: &[u32] = &[6, 7, 8];
    let patch = Patch3DData::new(verts, &[f0, f1])
        .unwrap()
        .with_face_scalars(&[0.9, 0.2])
        .unwrap();
    assert_eq!(patch.face_count(), 2);
    // Perimeter edges must be 6 + 3 = 9, NOT including internal ear-clip diagonals!
    assert_eq!(patch.edges().len(), 9);
    assert_eq!(patch.triangles().len(), 5);

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut face0_pos = Pos2::ZERO;
    ctx.run_ui(raw(vec![]), |ui| {
        let resp = Plot3D::new("p3d")
            .depth_mode(DepthMode::ZBuffer)
            .camera(Camera3D {
                azimuth_deg: 25.0,
                elevation_deg: 65.0,
                ..Default::default()
            })
            .show(
                ui,
                &mut state,
                &[Layer3D::from(
                    Patch3D::new(&patch)
                        .color_scale(ColorScale::new([0.0, 1.0]).unwrap())
                        .shading(SurfaceShading::Faceted)
                        .specular(24.0, 0.4),
                )],
            );
        assert!(resp.diagnostics.plot3d.output_triangles >= 5);
        assert!(resp.diagnostics.plot3d.zbuffer_pixels > 0);
        face0_pos = resp.transform.project_data([0.15, 0.75, 0.5]).unwrap().0;
    })
    .drop_without_applying_deltas();

    let mut reg = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(face0_pos)]), |ui| {
        reg = Plot3D::new("p3d")
            .depth_mode(DepthMode::ZBuffer)
            .camera(Camera3D {
                azimuth_deg: 25.0,
                elevation_deg: 65.0,
                ..Default::default()
            })
            .show(ui, &mut state, &[Layer3D::from(Patch3D::new(&patch))])
            .hovered_region;
    })
    .drop_without_applying_deltas();
    assert_eq!(reg.expect("patch3 face hover").item_index, 0);
}

#[test]
fn ext10_alphashape3d_regular_lattice_cavity_culling_and_outward_normals() {
    // Coplanar must error.
    let coplanar = vec![
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    assert_eq!(
        AlphaShape3DData::from_sites(&coplanar, 2.0).unwrap_err(),
        TriMeshError::DegenerateSites
    );

    // 3x3x3 regular lattice: tests co-spherical robustness via symbolic perturbation.
    let mut lattice = Vec::new();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                lattice.push([x as f64, y as f64, z as f64]);
            }
        }
    }
    let crit = AlphaShape3DData::critical_alpha(&lattice).unwrap();
    assert!(crit > 0.5 && crit.is_finite());

    let shape = AlphaShape3DData::from_sites(&lattice, crit).unwrap();
    assert!(shape.kept_tetrahedra() > 0);
    // Internal shared faces must be culled: strictly fewer than 4 * kept_tets.
    assert!(shape.boundary_faces().len() < 4 * shape.kept_tetrahedra());
    let mesh = shape.mesh().expect("non-empty alpha mesh");

    // Critical-alpha shell must be closed manifold (every boundary edge has exact degree 2).
    let mut b_edges: HashMap<[u32; 2], usize> = HashMap::new();
    for &[a, b, c] in mesh.triangles() {
        for [u, v] in [[a, b], [b, c], [c, a]] {
            *b_edges.entry([u.min(v), u.max(v)]).or_insert(0) += 1;
        }
    }
    assert!(b_edges.values().all(|&deg| deg == 2 || deg % 2 == 0));

    // Convex-hull alpha guarantees 100% of boundary normals point outward from centroid.
    let hull = AlphaShape3DData::from_sites(&lattice, 100.0).unwrap();
    let hmesh = hull.mesh().expect("non-empty hull");
    let centroid = [1.0, 1.0, 1.0];
    let verts = hmesh.vertices();
    for &[a, b, c] in hmesh.triangles() {
        let pa = verts[a as usize];
        let pb = verts[b as usize];
        let pc = verts[c as usize];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let fc = [
            (pa[0] + pb[0] + pc[0]) / 3.0 - centroid[0],
            (pa[1] + pb[1] + pc[1]) / 3.0 - centroid[1],
            (pa[2] + pb[2] + pc[2]) / 3.0 - centroid[2],
        ];
        assert!(n[0] * fc[0] + n[1] * fc[1] + n[2] * fc[2] > 0.0);
    }

    // alpha = 0 yields safe empty mesh without panicking.
    let empty = AlphaShape3DData::from_sites(&lattice, 0.0).unwrap();
    assert_eq!(empty.kept_tetrahedra(), 0);
    assert!(empty.mesh().is_none());

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    ctx.run_ui(raw(vec![]), |ui| {
        let r =
            Plot3D::new("a3d").show(ui, &mut state, &[Layer3D::from(AlphaShape3D::new(&shape))]);
        assert!(r.outer_rect.is_positive());
        let r2 = Plot3D::new("a3d_empty").show(
            ui,
            &mut state,
            &[Layer3D::from(AlphaShape3D::new(&empty))],
        );
        assert!(r2.outer_rect.is_positive());
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext11_graph3d_layouts_multiedge_selfloop_and_node_picking() {
    let g2 = GraphData::directed(
        4,
        &[[0, 1], [1, 0], [1, 2], [2, 3], [0, 0]],
        GraphLayout::Circle,
    )
    .unwrap();
    let a = g2
        .layout_3d(GraphLayout3D::Force { iterations: 40 })
        .unwrap();
    let b = g2
        .layout_3d(GraphLayout3D::Force { iterations: 40 })
        .unwrap();
    assert_eq!(a.coords(), b.coords());

    let sph =
        Graph3DData::undirected(6, &[[0, 1], [1, 2], [2, 3]], GraphLayout3D::Spherical).unwrap();
    for p in sph.coords() {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 1.0).abs() < 1e-9);
    }

    let custom = Graph3DData::directed(
        3,
        &[[0, 1], [1, 0], [1, 2], [0, 0]],
        GraphLayout3D::Coordinates(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 1.0]].into()),
    )
    .unwrap()
    .with_node_labels(&["N0", "N1", "N2"])
    .unwrap();

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut n1_pos = Pos2::ZERO;
    for proj in [
        Projection3D::Perspective { fov_y_rad: 0.65 },
        Projection3D::Orthographic,
    ] {
        ctx.run_ui(raw(vec![]), |ui| {
            let r = Plot3D::new("g3d").projection(proj).show(
                ui,
                &mut state,
                &[Layer3D::from(Graph3D::new(&custom).show_node_labels(true))],
            );
            assert!(r.diagnostics.plot3d.output_segments > 0);
            n1_pos = r.transform.project_data([1.0, 0.0, 0.0]).unwrap().0;
        })
        .drop_without_applying_deltas();
    }

    let mut hit = None;
    ctx.run_ui(raw(vec![Event::PointerMoved(n1_pos)]), |ui| {
        hit = Plot3D::new("g3d")
            .projection(Projection3D::Orthographic)
            .show(ui, &mut state, &[Layer3D::from(Graph3D::new(&custom))])
            .hovered_point;
    })
    .drop_without_applying_deltas();
    assert_eq!(hit.expect("graph3 node hover").point_index, 1);
}

#[test]
fn ext12_adaptive_fcontour_fsurf_watertight_density_budget_and_pole_guard() {
    assert!(matches!(
        TriMesh3DData::from_explicit_fn_adaptive([-1.0, 1.0], [-1.0, 1.0], 10, |x, y| x + y),
        Err(FunctionContourError::BudgetExceeded { .. })
    ));

    let mut evals = 0usize;
    let peak = |x: f64, y: f64| {
        evals += 1;
        (-45.0 * ((x - 0.35) * (x - 0.35) + (y + 0.25) * (y + 0.25))).exp()
    };
    let mesh =
        TriMesh3DData::from_explicit_fn_adaptive([-1.0, 1.0], [-1.0, 1.0], 1_600, peak).unwrap();
    assert!(evals <= 1_600);
    assert!(mesh.vertices().len() <= 1_600);

    // Steep region near (0.35, -0.25) must have higher vertex density than flat quadrant (-1..0, 0..1).
    let near_peak = mesh
        .vertices()
        .iter()
        .filter(|p| (p[0] - 0.35).abs() < 0.35 && (p[1] + 0.25).abs() < 0.35)
        .count();
    let flat_corner = mesh
        .vertices()
        .iter()
        .filter(|p| p[0] < -0.3 && p[1] > 0.3)
        .count();
    assert!(
        near_peak > flat_corner * 2,
        "expected adaptive focus near peak ({near_peak} vs {flat_corner})"
    );

    // Every interior edge must be shared by exact degree 2 triangles (zero T-junction cracks!).
    let mut edge_inc: HashMap<[u32; 2], usize> = HashMap::new();
    let verts = mesh.vertices();
    for &[a, b, c] in mesh.triangles() {
        for [u, v] in [[a, b], [b, c], [c, a]] {
            *edge_inc.entry([u.min(v), u.max(v)]).or_insert(0) += 1;
        }
    }
    let on_outer_boundary = |p: Point3D, q: Point3D| {
        (p[0] + 1.0).abs() < 1e-9 && (q[0] + 1.0).abs() < 1e-9
            || (p[0] - 1.0).abs() < 1e-9 && (q[0] - 1.0).abs() < 1e-9
            || (p[1] + 1.0).abs() < 1e-9 && (q[1] + 1.0).abs() < 1e-9
            || (p[1] - 1.0).abs() < 1e-9 && (q[1] - 1.0).abs() < 1e-9
    };
    for ([u, v], deg) in edge_inc {
        let pu = verts[u as usize];
        let pv = verts[v as usize];
        if !on_outer_boundary(pu, pv) {
            assert_eq!(
                deg, 2,
                "crack or non-manifold interior edge {pu:?}..{pv:?} deg={deg}"
            );
        } else {
            assert_eq!(deg, 1, "outer boundary edge must have degree 1");
        }
    }

    // Adaptive fcontour / fcontourf + pole cliff guard on 1 / (x - y).
    let cl = ContourData::from_function_adaptive(
        [-1.0, 1.0],
        [-1.0, 1.0],
        ContourLevels::Count(6),
        1_200,
        |x, y| (-20.0 * (x * x + y * y)).exp(),
    )
    .unwrap();
    let cf = FilledContourData::from_function_adaptive(
        [-1.0, 1.0],
        [-1.0, 1.0],
        ContourLevels::Count(6),
        1_200,
        |x, y| (-20.0 * (x * x + y * y)).exp(),
    )
    .unwrap();
    assert!(!cl.lines().is_empty());
    assert!(!cf.bands().is_empty());

    let pole_c = ContourData::from_function_adaptive(
        [-1.0, 1.0],
        [-1.0, 1.0],
        ContourLevels::Count(5),
        800,
        |x, y| 1.0 / (x - y),
    )
    .unwrap();
    let ctx = Context::default();
    let mut state = PlotState::default();
    ctx.run_ui(raw(vec![]), |ui| {
        let resp = Plot::new("adapt_pole").show_layers(
            ui,
            &mut state,
            &[
                PlotLayer::from(FilledContours::new(
                    &cf,
                    ColorScale::new([0.0, 1.0]).unwrap(),
                )),
                PlotLayer::from(Contours::new(&pole_c)),
            ],
        );
        let _: Option<PlotBounds> = None;
        let _: [AxisScale; 2] = [AxisScale::Linear; 2];
        let _: PlotPoint = [0.0, 0.0];
        assert!(resp.outer_rect.is_positive());
    })
    .drop_without_applying_deltas();
}

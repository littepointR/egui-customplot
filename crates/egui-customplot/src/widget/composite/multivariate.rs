use egui::{Color32, Ui};

use crate::{
    Axis, AxisLink, BarOrientation, Bars, Figure, FigureError, FigureResponse, Line, MarginalKind,
    Plot, PlotLayer, PlotMatrixData, PlotResponse, PlotState, Scatter, ScatterHistogramData,
};

const GROUP_PALETTE: [Color32; 6] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(162, 20, 47),
];

/// Host-owned view state for [`ScatterHistogram`].
///
/// [`ScatterHistogram`] 中心散点图及顶部/右侧边际直方图的宿主持有联动状态。
#[derive(Clone, Default)]
pub struct ScatterHistogramState {
    pub scatter: PlotState,
    pub top: PlotState,
    pub right: PlotState,
}

/// Scatter plot with top-X and right-Y marginal histograms or KDEs (`scatterhistogram`, `ML-023`).
///
/// 带上方 X 轴与右侧 Y 轴边际直方图/核密度估计的分组散点图控件。
pub struct ScatterHistogram<'a> {
    id_salt: &'a str,
    data: &'a ScatterHistogramData,
    cell_height: f32,
    x_label: String,
    y_label: String,
    title: String,
    allow_scroll_zoom: bool,
}

impl<'a> ScatterHistogram<'a> {
    pub fn new(id_salt: &'a str, data: &'a ScatterHistogramData) -> Self {
        Self {
            id_salt,
            data,
            cell_height: 220.0,
            x_label: "X".to_owned(),
            y_label: "Y".to_owned(),
            title: String::new(),
            allow_scroll_zoom: false,
        }
    }

    pub fn cell_height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.cell_height = height.max(96.0);
        }
        self
    }

    pub fn labels(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        self.x_label = x.into();
        self.y_label = y.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        state: &mut ScatterHistogramState,
    ) -> Result<FigureResponse<PlotResponse>, FigureError> {
        let ctx = ui.ctx().clone();
        let res = Figure::new(self.id_salt)
            .columns(2)
            .cell_height(self.cell_height)
            .spacing(egui::vec2(6.0, 6.0))
            .show(ui, 4, |cell| match cell.index() {
                0 => {
                    let mut layers: Vec<PlotLayer<'_>> = vec![
                        Bars::new(&self.data.x_bars)
                            .color(Color32::from_rgb(76, 132, 196).gamma_multiply(0.75))
                            .into(),
                    ];
                    if let Some(kde) = &self.data.x_kde {
                        layers.push(
                            Line::new(kde)
                                .color(Color32::from_rgb(217, 83, 25))
                                .width(2.0)
                                .into(),
                        );
                    }
                    let plot = Plot::new((self.id_salt, "top"))
                        .title(self.title.clone())
                        .allow_scroll_zoom(self.allow_scroll_zoom)
                        .x_axis(Axis::new().tick_labels_visible(false))
                        .y_axis(Axis::new().label("Density"));
                    cell.show_layers(plot, &mut state.top, &layers)
                }
                1 => {
                    let empty = Plot::new((self.id_salt, "spacer"))
                        .canvas_axes()
                        .show_frame(false)
                        .show_grid(false)
                        .show_hover(false)
                        .allow_drag(false)
                        .show(cell.ui(), &mut PlotState::default(), &[]);
                    let rect = cell.rect();
                    let label = match self.data.marginal {
                        MarginalKind::Histogram => "Marginal: Histogram",
                        MarginalKind::Kernel => "Marginal: Histogram + KDE",
                        MarginalKind::None => "Marginal: Off",
                    };
                    let ui = cell.ui();
                    let font = egui::TextStyle::Small.resolve(ui.style());
                    let color = ui.visuals().text_color();
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{label}\nGroups: {}", self.data.groups.len()),
                        font,
                        color,
                    );
                    empty
                }
                2 => {
                    let mut layers: Vec<PlotLayer<'_>> = Vec::with_capacity(self.data.groups.len());
                    for (idx, (grp_id, pts)) in self.data.groups.iter().enumerate() {
                        layers.push(
                            Scatter::new(pts)
                                .name(match idx {
                                    0 => "Group A",
                                    1 => "Group B",
                                    2 => "Group C",
                                    _ => "Group",
                                })
                                .color(GROUP_PALETTE[grp_id % GROUP_PALETTE.len()])
                                .radius(3.5)
                                .into(),
                        );
                    }
                    let plot = Plot::new((self.id_salt, "scatter"))
                        .allow_scroll_zoom(self.allow_scroll_zoom)
                        .x_axis(Axis::new().label(self.x_label.clone()))
                        .y_axis(Axis::new().label(self.y_label.clone()))
                        .legend(self.data.groups.len() > 1);
                    cell.show_layers(plot, &mut state.scatter, &layers)
                }
                _ => {
                    let mut layers: Vec<PlotLayer<'_>> = vec![
                        Bars::new(&self.data.y_bars)
                            .orientation(BarOrientation::Horizontal)
                            .color(Color32::from_rgb(76, 132, 196).gamma_multiply(0.75))
                            .into(),
                    ];
                    if let Some(kde) = &self.data.y_kde {
                        layers.push(
                            Line::new(kde)
                                .color(Color32::from_rgb(217, 83, 25))
                                .width(2.0)
                                .into(),
                        );
                    }
                    let plot = Plot::new((self.id_salt, "right"))
                        .allow_scroll_zoom(self.allow_scroll_zoom)
                        .x_axis(Axis::new().label("Density"))
                        .y_axis(Axis::new().tick_labels_visible(false));
                    cell.show_layers(plot, &mut state.right, &layers)
                }
            })?;

        let mut x_refs = [&mut state.top, &mut state.scatter];
        let _ = AxisLink::new([true, false]).synchronize(&ctx, &mut x_refs);
        let mut y_refs = [&mut state.scatter, &mut state.right];
        let _ = AxisLink::new([false, true]).synchronize(&ctx, &mut y_refs);
        Ok(res)
    }
}

/// `M x M` pairwise scatter & diagonal histogram matrix (`plotmatrix`, `ML-033`).
///
/// 成对多变量矩阵散点图控件：对角线展示变量分布直方图，非对角格子行/列共享坐标联动。
pub struct PlotMatrix<'a> {
    id_salt: &'a str,
    data: &'a PlotMatrixData,
    cell_height: f32,
    allow_scroll_zoom: bool,
}

impl<'a> PlotMatrix<'a> {
    pub fn new(id_salt: &'a str, data: &'a PlotMatrixData) -> Self {
        Self {
            id_salt,
            data,
            cell_height: 155.0,
            allow_scroll_zoom: false,
        }
    }

    pub fn cell_height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.cell_height = height.max(80.0);
        }
        self
    }

    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        states: &mut Vec<PlotState>,
    ) -> Result<FigureResponse<PlotResponse>, FigureError> {
        let ctx = ui.ctx().clone();
        let m = self.data.n_vars;
        let needed = m * m;
        if states.len() < needed {
            states.resize(needed, PlotState::default());
        }
        let res = Figure::new(self.id_salt)
            .columns(m.max(1))
            .cell_height(self.cell_height)
            .spacing(egui::vec2(6.0, 6.0))
            .show(ui, needed, |cell| {
                let idx = cell.index();
                let r = idx / m;
                let c = idx % m;
                let mut x_axis = Axis::new().tick_labels_visible(r + 1 == m);
                if r + 1 == m {
                    x_axis = x_axis.label(self.data.names[c].clone());
                }
                let mut y_axis = Axis::new().tick_labels_visible(c == 0);
                if c == 0 {
                    y_axis = y_axis.label(self.data.names[r].clone());
                }
                let plot = Plot::new((self.id_salt, r, c))
                    .allow_scroll_zoom(self.allow_scroll_zoom)
                    .x_axis(x_axis)
                    .y_axis(y_axis);
                if r == c {
                    let bars = Bars::new(&self.data.diagonals[r])
                        .color(Color32::from_rgb(66, 135, 245).gamma_multiply(0.8));
                    cell.show_layers(plot, &mut states[idx], &[bars.into()])
                } else {
                    let layers: Vec<PlotLayer<'_>> = self.data.off_diagonal[idx]
                        .iter()
                        .map(|(grp, pts)| {
                            Scatter::new(pts)
                                .color(GROUP_PALETTE[grp % GROUP_PALETTE.len()])
                                .radius(2.8)
                                .into()
                        })
                        .collect();
                    cell.show_layers(plot, &mut states[idx], &layers)
                }
            })?;

        for c in 0..m {
            let mut refs: Vec<&mut PlotState> = states[..needed]
                .iter_mut()
                .enumerate()
                .filter_map(|(i, s)| (i % m == c).then_some(s))
                .collect();
            let _ = AxisLink::new([true, false]).synchronize(&ctx, &mut refs);
        }
        for r in 0..m {
            let mut refs: Vec<&mut PlotState> = states[..needed]
                .iter_mut()
                .enumerate()
                .filter_map(|(i, s)| (i / m == r && i % m != r).then_some(s))
                .collect();
            if refs.len() >= 2 {
                let _ = AxisLink::new([false, true]).synchronize(&ctx, &mut refs);
            }
        }
        Ok(res)
    }
}

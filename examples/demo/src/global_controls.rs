use eframe::egui;
use egui_customplot::{PlotDefaults, Projection3D, set_plot_defaults};

use crate::{
    appearance::{Appearance, Presentation},
    export::ExportPanel,
    i18n::{lang, tr},
};

/// Standalone global plot controls shared across every gallery via context `PlotDefaults`.
pub(super) struct GlobalPlotControls {
    pub open: bool,
    pub scroll_zoom: bool,
    pub box_zoom: bool,
    pub show_grid: bool,
    pub show_legend: bool,
    pub show_hover: bool,
    pub crosshair: bool,
    pub perspective_3d: bool,
}

impl Default for GlobalPlotControls {
    fn default() -> Self {
        Self {
            open: true,
            scroll_zoom: true,
            box_zoom: true,
            show_grid: true,
            show_legend: true,
            show_hover: true,
            crosshair: true,
            perspective_3d: false,
        }
    }
}

impl GlobalPlotControls {
    pub fn sync_defaults(
        &self,
        ctx: &egui::Context,
        appearance: &Appearance,
        presentation: &Presentation,
    ) {
        let projection_3d = Some(if self.perspective_3d {
            Projection3D::perspective_default()
        } else {
            Projection3D::Orthographic
        });
        set_plot_defaults(
            ctx,
            Some(PlotDefaults {
                theme: appearance.resolved_theme(presentation),
                inherited_style: appearance.inherited_plot_style(),
                allow_scroll_zoom: Some(self.scroll_zoom),
                allow_box_zoom: Some(self.box_zoom),
                show_grid: Some(self.show_grid),
                show_legend: Some(self.show_legend),
                show_hover: Some(self.show_hover),
                crosshair: Some(self.crosshair),
                projection_3d,
            }),
        );
    }

    /// Returns `true` when the user clicked the reset-active-view button.
    pub fn show_panel(
        &mut self,
        ui: &mut egui::Ui,
        appearance: &mut Appearance,
        export: &mut ExportPanel,
    ) -> bool {
        let l = lang(ui.ctx());
        ui.heading(tr(l, "Plot Controls", "绘图控制面板", "プロット制御"));
        ui.small(tr(
            l,
            "Applies globally across all 27 galleries.",
            "全局生效于所有 27 组图表画廊。",
            "全 27 ギャラリーへ一括適用されます。",
        ));
        ui.separator();

        appearance.show(ui);
        ui.separator();

        egui::CollapsingHeader::new(tr(
            l,
            "Navigation & Overlays",
            "交互导航与辅助显示",
            "操作・オーバーレイ",
        ))
        .id_salt("global_nav_overlays")
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(
                &mut self.scroll_zoom,
                tr(l, "Wheel zoom", "滚轮缩放", "ホイールズーム"),
            );
            ui.checkbox(
                &mut self.box_zoom,
                tr(
                    l,
                    "Box zoom (Right/Shift+drag)",
                    "框选缩放 (右键/Shift)",
                    "矩形ズーム",
                ),
            );
            ui.checkbox(
                &mut self.show_grid,
                tr(l, "Grid lines", "显示网格", "グリッド表示"),
            );
            ui.checkbox(
                &mut self.show_legend,
                tr(l, "Interactive legend", "显示可交互图例", "凡例表示"),
            );
            ui.checkbox(
                &mut self.show_hover,
                tr(l, "Hover inspection", "悬停数据提示", "ホバー表示"),
            );
            ui.checkbox(
                &mut self.crosshair,
                tr(l, "2D crosshair", "2D 十字准线", "2D 十字線"),
            );
            ui.checkbox(
                &mut self.perspective_3d,
                tr(l, "3D perspective projection", "3D 透视投影", "3D 透視投影"),
            );
        });

        ui.add_space(4.0);
        let reset_clicked = ui
            .button(tr(
                l,
                "Reset active gallery view",
                "重置当前画廊视图",
                "表示中のビューを初期化",
            ))
            .clicked();

        export.controls(ui);
        reset_clicked
    }
}

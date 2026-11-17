use eframe::egui;
use egui_customplot::{Axis, AxisScale, LineInterpolation, MarkerShape, Plot};

use crate::{
    appearance::Appearance,
    data::Scene,
    i18n::{Language, lang, tr},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum XAxisFormat {
    Numeric,
    DateTime,
    Duration,
}

pub(super) struct Options {
    pub appearance: Appearance,
    pub performance: crate::performance::Options,
    pub scene: Scene,
    pub count: usize,
    pub gaps: bool,
    pub second_view: bool,
    pub multi: crate::charts::MultiViewOptions,
    pub grid: bool,
    pub legend: bool,
    pub hover: bool,
    pub markers: bool,
    pub dashed: bool,
    pub interpolation: LineInterpolation,
    pub equal_units: bool,
    pub observations: bool,
    pub log_axes: [bool; 2],
    pub reversed: [bool; 2],
    pub x_format: XAxisFormat,
    pub lock_y: bool,
    pub width: f32,
    pub radius: f32,
    pub shape: MarkerShape,
    pub color: egui::Color32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            appearance: Appearance::default(),
            performance: crate::performance::Options::default(),
            scene: Scene::Signal,
            count: 10_000,
            gaps: true,
            second_view: false,
            multi: crate::charts::MultiViewOptions::default(),
            grid: true,
            legend: true,
            hover: true,
            markers: false,
            dashed: false,
            interpolation: LineInterpolation::Linear,
            equal_units: false,
            observations: true,
            log_axes: [false; 2],
            reversed: [false; 2],
            x_format: XAxisFormat::Numeric,
            lock_y: false,
            width: 1.5,
            radius: 3.5,
            shape: MarkerShape::Circle,
            color: egui::Color32::from_rgb(70, 160, 235),
        }
    }
}

pub(super) struct Changes {
    pub rebuild: bool,
    pub reset: bool,
    pub reindex: bool,
}

impl Options {
    pub fn plot(&self, id: &'static str, height: f32, l: Language) -> Plot {
        let axis = |index: usize, label: &str| {
            let mut ax = Axis::new()
                .scale(if self.log_axes[index] {
                    AxisScale::Log10
                } else {
                    AxisScale::Linear
                })
                .reversed(self.reversed[index])
                .minor_grid(true)
                .label(label);
            if index == 0 && !self.log_axes[0] {
                ax = match self.x_format {
                    XAxisFormat::Numeric => ax,
                    XAxisFormat::DateTime => ax.datetime(),
                    XAxisFormat::Duration => ax.duration(),
                };
            }
            ax
        };
        let plot = Plot::new(id)
            .height(height)
            .show_grid(self.grid)
            .allow_scroll_zoom(true)
            .allow_box_zoom(true)
            .allow_brush(true)
            .drag_axes([true, !self.lock_y])
            .zoom_axes([true, !self.lock_y])
            .title(match self.scene {
                Scene::Signal => tr(
                    l,
                    "Signal and observations",
                    "信号与观测散点",
                    "信号と観測散布図",
                ),
                Scene::PowerLaw => tr(
                    l,
                    "Power law: linear / semilog / loglog",
                    "幂律分布：线性 / 半对数 / 双对数",
                    "べき乗則：線形 / 片対数 / 両対数",
                ),
            })
            .x_axis(axis(0, "x"))
            .y_axis(axis(1, tr(l, "Amplitude", "幅值", "振幅")))
            .legend(self.legend)
            .show_hover(self.hover);
        if self.equal_units {
            plot.data_aspect(1.0)
        } else {
            plot
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, options: &mut Options) -> Changes {
    let l = lang(ui.ctx());
    let old_data = (options.scene, options.count, options.gaps);
    let old_axes = (options.log_axes, options.reversed);
    ui.strong(tr(
        l,
        "2D Signal & Power Law",
        "2D 信号与幂律场景",
        "2D 信号・べき乗則",
    ));
    let reindex = options.performance.show(ui);
    ui.separator();
    ui.selectable_value(
        &mut options.scene,
        Scene::Signal,
        tr(
            l,
            "Signal + scatter",
            "正弦信号 + 观测散点",
            "信号 + 散布図",
        ),
    );
    ui.selectable_value(
        &mut options.scene,
        Scene::PowerLaw,
        tr(l, "Positive power law", "正值幂律分布", "正べき乗則"),
    );
    if options.scene != old_data.0 {
        options.log_axes = [options.scene == Scene::PowerLaw; 2];
    }
    let count_label = |n: usize| match l {
        Language::English => format!("{n} points"),
        Language::ZhCn => format!("{n} 样本点"),
        Language::Japanese => format!("{n} 点"),
    };
    egui::ComboBox::from_id_salt("samples")
        .selected_text(count_label(options.count))
        .show_ui(ui, |ui| {
            for count in [1_000, 10_000, 100_000, 1_000_000, 10_000_000] {
                ui.selectable_value(&mut options.count, count, count_label(count));
            }
        });
    ui.checkbox(&mut options.gaps, tr(l, "NaN gap", "NaN 断口", "NaN 欠損"));
    options.multi.show(ui, &mut options.second_view);
    ui.separator();
    ui.checkbox(
        &mut options.grid,
        tr(
            l,
            "Grid (major + log minor)",
            "网格（主刻度 + 对数次刻度）",
            "グリッド（主＋対数補助）",
        ),
    );
    ui.checkbox(
        &mut options.legend,
        tr(l, "Clickable legend", "可交互图例", "操作可能な凡例"),
    );
    ui.checkbox(
        &mut options.hover,
        tr(
            l,
            "Raw-point hover + crosshair",
            "原始点悬停 + 十字线",
            "生データホバー + 十字線",
        ),
    );
    ui.checkbox(
        &mut options.observations,
        tr(
            l,
            "Per-point colored / sized scatter",
            "逐点着色 / 变径散点",
            "点別色・半径の散布図",
        ),
    );
    ui.checkbox(
        &mut options.markers,
        tr(
            l,
            "Markers on every line sample",
            "折线逐点标记",
            "線上の全点マーカー",
        ),
    );
    ui.checkbox(
        &mut options.dashed,
        tr(l, "Dashed signal stroke", "虚线信号线", "破線ストローク"),
    );
    let interp_label = |interp: LineInterpolation| match interp {
        LineInterpolation::Linear => tr(l, "Linear", "Linear（线性）", "Linear（線形）"),
        LineInterpolation::StepAfter => {
            tr(l, "StepAfter", "StepAfter（后阶）", "StepAfter（後階段）")
        }
        LineInterpolation::StepBefore => tr(
            l,
            "StepBefore",
            "StepBefore（前阶）",
            "StepBefore（前階段）",
        ),
    };
    egui::ComboBox::from_id_salt("interpolation")
        .selected_text(interp_label(options.interpolation))
        .show_ui(ui, |ui| {
            for interpolation in [
                LineInterpolation::Linear,
                LineInterpolation::StepAfter,
                LineInterpolation::StepBefore,
            ] {
                ui.selectable_value(
                    &mut options.interpolation,
                    interpolation,
                    interp_label(interpolation),
                );
            }
        });
    if options.interpolation != LineInterpolation::Linear {
        ui.small(tr(
            l,
            "StepAfter: hold then jump; StepBefore: jump then hold. Raw markers / hover; MinMax can change jump times.",
            "StepAfter：先保持后跳变；StepBefore：先跳变后保持。拾取使用原始点；MinMax 可能移动跳变时刻。",
            "StepAfter：維持後ジャンプ；StepBefore：ジャンプ後維持。ホバーは生点基準、MinMax は時刻がずれる場合があります。",
        ));
    }
    ui.checkbox(
        &mut options.equal_units,
        tr(
            l,
            "Equal X/Y units (log: decades)",
            "X/Y 等比例单位（对数为数量级）",
            "X/Y 等倍単位（対数は桁）",
        ),
    );
    if options.markers && options.count >= 100_000 {
        ui.colored_label(
            egui::Color32::YELLOW,
            tr(
                l,
                "Markers remain full-resolution: no marker LOD / hard mesh budget.",
                "逐点标记保持全分辨率：无标记降采样或硬网格预算。",
                "線上マーカーは全解像度維持（LOD なし）。",
            ),
        );
    }
    ui.horizontal(|ui| {
        ui.checkbox(&mut options.log_axes[0], tr(l, "Log X", "对数 X", "対数 X"));
        ui.checkbox(&mut options.log_axes[1], tr(l, "Log Y", "对数 Y", "対数 Y"));
    });
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut options.reversed[0],
            tr(l, "Reverse X", "反向 X", "反転 X"),
        );
        ui.checkbox(
            &mut options.reversed[1],
            tr(l, "Reverse Y", "反向 Y", "反転 Y"),
        );
    });
    let x_fmt_label = |fmt: XAxisFormat| match fmt {
        XAxisFormat::Numeric => tr(l, "X: Numeric", "X 轴：数值", "X 軸：数値"),
        XAxisFormat::DateTime => tr(l, "X: DateTime (UTC)", "X 轴：日期时间", "X 軸：日時"),
        XAxisFormat::Duration => tr(l, "X: Duration", "X 轴：时长", "X 軸：経過時間"),
    };
    egui::ComboBox::from_id_salt("x-format")
        .selected_text(x_fmt_label(options.x_format))
        .show_ui(ui, |ui| {
            for fmt in [
                XAxisFormat::Numeric,
                XAxisFormat::DateTime,
                XAxisFormat::Duration,
            ] {
                ui.selectable_value(&mut options.x_format, fmt, x_fmt_label(fmt));
            }
        });
    ui.checkbox(
        &mut options.lock_y,
        tr(l, "Lock Y navigation", "锁定 Y 轴导航", "Y 軸操作を固定"),
    );
    options.appearance.show_scene_extras(ui, &mut options.color);
    ui.add(egui::Slider::new(&mut options.width, 0.0..=5.0).text(tr(
        l,
        "Line width",
        "线宽",
        "線幅",
    )));
    ui.add(egui::Slider::new(&mut options.radius, 1.0..=10.0).text(tr(
        l,
        "Line marker radius",
        "标记半径",
        "マーカー半径",
    )));
    let shape_label = |shape: MarkerShape| match shape {
        MarkerShape::Circle => tr(l, "Circle", "圆形 (Circle)", "円 (Circle)"),
        MarkerShape::Square => tr(l, "Square", "方形 (Square)", "四角 (Square)"),
        MarkerShape::Diamond => tr(l, "Diamond", "菱形 (Diamond)", "ひし形 (Diamond)"),
        MarkerShape::TriangleUp => tr(l, "TriangleUp", "上三角 (TriangleUp)", "上三角"),
        MarkerShape::TriangleDown => tr(l, "TriangleDown", "下三角 (TriangleDown)", "下三角"),
        MarkerShape::Plus => tr(l, "Plus", "十字 (Plus)", "十字 (Plus)"),
        MarkerShape::Cross => tr(l, "Cross", "叉号 (Cross)", "バツ (Cross)"),
        MarkerShape::Asterisk => tr(l, "Asterisk", "星号 (Asterisk)", "アスタリスク"),
        MarkerShape::Pentagram => tr(l, "Pentagram", "五角星 (Pentagram)", "五芒星"),
        MarkerShape::Hexagram => tr(l, "Hexagram", "六角星 (Hexagram)", "六芒星"),
        MarkerShape::CircleHollow => tr(l, "CircleHollow", "空心圆", "白抜き円"),
        MarkerShape::SquareHollow => tr(l, "SquareHollow", "空心方", "白抜き四角"),
        MarkerShape::DiamondHollow => tr(l, "DiamondHollow", "空心菱形", "白抜きひし形"),
        MarkerShape::TriangleUpHollow => tr(l, "TriangleUpHollow", "空心上三角", "白抜き上三角"),
        MarkerShape::TriangleDownHollow => {
            tr(l, "TriangleDownHollow", "空心下三角", "白抜き下三角")
        }
    };
    egui::ComboBox::from_id_salt("shape")
        .selected_text(shape_label(options.shape))
        .show_ui(ui, |ui| {
            for shape in MarkerShape::ALL {
                ui.selectable_value(&mut options.shape, shape, shape_label(shape));
            }
        });
    let reset = ui
        .button(tr(
            l,
            "Auto-fit both views",
            "自动适配全部视图",
            "全ビュー自動フィット",
        ))
        .clicked();
    ui.separator();
    ui.small(tr(
        l,
        "Drag: pan\nWheel: zoom\nRight/Shift-drag: box zoom\nAlt-drag: brush (Esc clears)\nDouble-click / focused Home: fit\nLegend: toggle visibility",
        "拖拽：平移\n滚轮：缩放\n右键 / Shift+拖拽：框选放大\nAlt+拖拽：数据刷选（Esc 清除）\n双击 / 聚焦 Home：适配\n点击图例：切换显隐",
        "ドラッグ：移動\nホイール：拡大縮小\n右/Shiftドラッグ：矩形ズーム\nAltドラッグ：データ選択 (Esc解除)\nダブルクリック / Home：フィット\n凡例クリック：表示切替",
    ));
    ui.small(tr(
        l,
        "Nonpositive log samples are omitted / break lines. Raw data is never changed.",
        "对数轴下非正样本被省略或断线；原始样本永不修改。",
        "対数軸では非正サンプルは省略・切断され、元データは不変です。",
    ));
    Changes {
        rebuild: old_data != (options.scene, options.count, options.gaps),
        reset: reset
            || old_axes != (options.log_axes, options.reversed)
            || old_data.0 != options.scene,
        reindex,
    }
}

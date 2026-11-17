use eframe::egui::{self, Color32, Pos2, Vec2};

use crate::i18n::{Language, tr};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ActiveGallery {
    #[default]
    Default2D,
    Stairs,
    Scalar,
    Statistical,
    Stems,
    Quiver,
    Polygons,
    Contours,
    Functions,
    Geometry,
    Distributions,
    Multivariate,
    Polar,
    Networks,
    Plot3D,
    Surfaces3D,
    Volumes3D,
    Geo,
    Stage13,
    Stage14,
    Stage15,
    Stage16,
    Dynamic10m,
    CustomAxes,
    CustomAnnotations,
    CustomAnimation,
    CustomStyling,
}

impl ActiveGallery {
    pub const ALL: [Self; 27] = [
        Self::Default2D,
        Self::Stairs,
        Self::Scalar,
        Self::Statistical,
        Self::Stems,
        Self::Quiver,
        Self::Polygons,
        Self::Contours,
        Self::Functions,
        Self::Geometry,
        Self::Distributions,
        Self::Multivariate,
        Self::Polar,
        Self::Networks,
        Self::Plot3D,
        Self::Surfaces3D,
        Self::Volumes3D,
        Self::Geo,
        Self::Stage13,
        Self::Stage14,
        Self::Stage15,
        Self::Stage16,
        Self::Dynamic10m,
        Self::CustomAxes,
        Self::CustomAnnotations,
        Self::CustomAnimation,
        Self::CustomStyling,
    ];

    pub fn from_cli(arg: Option<&str>) -> Self {
        match arg {
            Some("--stairs") => Self::Stairs,
            Some("--scalar") => Self::Scalar,
            Some("--statistical") => Self::Statistical,
            Some("--stems") => Self::Stems,
            Some("--quiver") => Self::Quiver,
            Some("--polygons") => Self::Polygons,
            Some("--contours") => Self::Contours,
            Some("--functions") => Self::Functions,
            Some("--geometry") => Self::Geometry,
            Some("--distributions") => Self::Distributions,
            Some("--multivariate") => Self::Multivariate,
            Some("--polar") => Self::Polar,
            Some("--networks") => Self::Networks,
            Some("--plot3d") => Self::Plot3D,
            Some("--surfaces3d") => Self::Surfaces3D,
            Some("--volumes3d") => Self::Volumes3D,
            Some("--geo") => Self::Geo,
            Some("--stage13") => Self::Stage13,
            Some("--stage14") => Self::Stage14,
            Some("--stage15") => Self::Stage15,
            Some("--stage16") => Self::Stage16,
            Some("--dynamic-10m") => Self::Dynamic10m,
            Some("--custom-axes") => Self::CustomAxes,
            Some("--custom-annotations") => Self::CustomAnnotations,
            Some("--custom-animation") => Self::CustomAnimation,
            Some("--custom-styling") => Self::CustomStyling,
            _ => Self::Default2D,
        }
    }

    pub fn title(self, l: Language) -> &'static str {
        match self {
            Self::Default2D => tr(
                l,
                "2D Signal & Power Law",
                "2D 信号与幂律",
                "2D 信号・べき乗則",
            ),
            Self::Stairs => tr(l, "2D Stairs Step Plot", "2D 阶梯折线图", "2D 階段プロット"),
            Self::Scalar => tr(
                l,
                "Heatmap & ColorBar",
                "热力图与共享色条",
                "ヒートマップ・色バー",
            ),
            Self::Statistical => tr(
                l,
                "Bars, Errors & Histogram",
                "柱状图 / 误差棒 / 直方图",
                "棒・誤差棒・ヒストグラム",
            ),
            Self::Stems => tr(
                l,
                "Discrete Stems & Baseline",
                "离散火柴杆与基线",
                "離散ステム・基準線",
            ),
            Self::Quiver => tr(
                l,
                "2D Vector Field (Quiver)",
                "2D 矢量场 (Quiver)",
                "2D ベクトル場 (Quiver)",
            ),
            Self::Polygons => tr(
                l,
                "Areas, Polygons & Patches",
                "面积 / 多边形 / 面片",
                "領域・多角形・パッチ",
            ),
            Self::Contours => tr(
                l,
                "Contours, Pcolor & Image",
                "等高线 / 伪彩 / 图像",
                "等高線・疑似カラー・画像",
            ),
            Self::Functions => tr(
                l,
                "Functions, Comet & EX-027",
                "自适应函数 / 彗星动画",
                "適応関数・動的ライン",
            ),
            Self::Geometry => tr(
                l,
                "Triangulation, Flow & Spy",
                "三角剖分 / 流场 / Spy",
                "三角網・流線・Spy",
            ),
            Self::Distributions => tr(
                l,
                "Distributions & Clouds",
                "统计分布与云图",
                "統計分布・クラウド",
            ),
            Self::Multivariate => tr(
                l,
                "Multivariate & Composite",
                "多变量与复合子图",
                "多変量・複合図",
            ),
            Self::Polar => tr(
                l,
                "Polar Plots & Rose",
                "极坐标与风玫瑰图",
                "極座標・ローズ図",
            ),
            Self::Networks => tr(
                l,
                "Graphs & Elimination Trees",
                "网络图与消去树",
                "グラフ・消去木",
            ),
            Self::Plot3D => tr(
                l,
                "3D Core, Discrete & Comet3",
                "3D 基础 / 柱杆 / 彗星",
                "3D 基本・棒・comet3",
            ),
            Self::Surfaces3D => tr(
                l,
                "3D Surfaces & Implicit",
                "3D 曲面 / 网格 / 隐式面",
                "3D 曲面・陰関数",
            ),
            Self::Volumes3D => tr(
                l,
                "3D Volumes, Iso & Flow",
                "3D 体切片 / 等值面 / 流粒子",
                "3D 体積・等値面・流れ",
            ),
            Self::Geo => tr(
                l,
                "2D Geographic & Basemaps",
                "2D 地理绘图与底图",
                "2D 地理プロット",
            ),
            Self::Stage13 => tr(
                l,
                "Confusion, Sky, 3D Patch/Alpha",
                "混淆矩阵 / 天球 / 3D 凹包",
                "混同・天球・3D メッシュ",
            ),
            Self::Stage14 => tr(
                l,
                "Stats, Clustering, SPC & OHLC",
                "概率图 / 聚类 / SPC / K 线",
                "確率・クラスタ・金融",
            ),
            Self::Stage15 => tr(
                l,
                "RF Smith, Control & DSP/CWT",
                "Smith 圆图 / 控制 / 时频谱",
                "Smith・制御・CWT",
            ),
            Self::Stage16 => tr(
                l,
                "Volume Raycast, PointCloud, Globe & Modern",
                "体渲染 / 点云 / 地球仪 / 现代图",
                "体描画・点群・地球儀",
            ),
            Self::Dynamic10m => tr(
                l,
                "Dynamic 10M @ 120Hz+ Live",
                "动态 1,000 万点 @ 120Hz+",
                "動的 1,000 万点ライブ",
            ),
            Self::CustomAxes => tr(
                l,
                "Custom Axes, Ticks, Time & Pan Limits",
                "客制化：数轴刻度 / 时间轴 / 边界限位",
                "カスタム：軸目盛・時刻・制限",
            ),
            Self::CustomAnnotations => tr(
                l,
                "Custom DataTips, Annotations & Brush",
                "客制化：DataTip 游标 / 注解 / 刷选",
                "カスタム：DataTip・注釈・選択",
            ),
            Self::CustomAnimation => tr(
                l,
                "Custom Animation & Playback Transport",
                "客制化：动画播放进度 / 倍速 / 流式",
                "カスタム：アニメ再生・タイムライン",
            ),
            Self::CustomStyling => tr(
                l,
                "Custom 5-Layer Styles & ColorMaps",
                "客制化：五层样式继承与色条布局",
                "カスタム：5層スタイル・カラーバー",
            ),
        }
    }

    pub fn category(self, l: Language) -> &'static str {
        match self {
            Self::Default2D
            | Self::Stairs
            | Self::Scalar
            | Self::Statistical
            | Self::Stems
            | Self::Quiver => tr(l, "2D Core Plots", "基础二维与矢量", "基本 2D プロット"),
            Self::Polygons | Self::Contours | Self::Functions | Self::Geometry => tr(
                l,
                "2D Geometry & Fields",
                "二维几何、场与函数",
                "2D 幾何・場・関数",
            ),
            Self::Distributions | Self::Multivariate | Self::Polar | Self::Networks => tr(
                l,
                "Stats, Polar & Networks",
                "统计分布、极坐标与网络",
                "統計・極座標・グラフ",
            ),
            Self::Plot3D | Self::Surfaces3D | Self::Volumes3D | Self::Geo => tr(
                l,
                "3D, Volumes & Geo",
                "三维曲面体绘与地理",
                "3D・体積・地理",
            ),
            Self::Stage13 | Self::Stage14 | Self::Stage15 | Self::Stage16 | Self::Dynamic10m => tr(
                l,
                "Toolboxes, Modern & 10M",
                "专业工具箱、现代图与极限性能",
                "専門ツールボックス・10M",
            ),
            Self::CustomAxes
            | Self::CustomAnnotations
            | Self::CustomAnimation
            | Self::CustomStyling => tr(
                l,
                "Customization & Widgets",
                "客制化部件与交互",
                "カスタマイズ部品",
            ),
        }
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&g| g == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&g| g == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

fn nav_arrow_button(ui: &mut egui::Ui, forward: bool, tooltip: &str) -> egui::Response {
    let size = Vec2::new(24.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
        painter.rect_stroke(
            rect,
            visuals.corner_radius,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let c = rect.center();
        let dx = if forward { 3.5 } else { -3.5 };
        let pts = vec![
            Pos2::new(c.x + dx, c.y),
            Pos2::new(c.x - dx, c.y - 4.5),
            Pos2::new(c.x - dx, c.y + 4.5),
        ];
        painter.add(egui::Shape::convex_polygon(
            pts,
            visuals.fg_stroke.color,
            egui::Stroke::NONE,
        ));
    }
    response.on_hover_text(tooltip)
}

pub(crate) fn show_selector(ui: &mut egui::Ui, current: &mut ActiveGallery, l: Language) -> bool {
    let before = *current;
    let idx = ActiveGallery::ALL
        .iter()
        .position(|&g| g == *current)
        .unwrap_or(0);
    ui.horizontal(|ui| {
        ui.strong(tr(l, "Gallery", "图表画廊", "ギャラリー"));
        ui.weak(format!("{}/{}", idx + 1, ActiveGallery::ALL.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if nav_arrow_button(
                ui,
                true,
                tr(l, "Next gallery", "下一个图表画廊", "次のギャラリー"),
            )
            .clicked()
            {
                *current = current.next();
            }
            if nav_arrow_button(
                ui,
                false,
                tr(l, "Previous gallery", "上一个图表画廊", "前のギャラリー"),
            )
            .clicked()
            {
                *current = current.prev();
            }
        });
    });

    egui::ComboBox::from_id_salt("active_gallery_selector")
        .width(ui.available_width())
        .selected_text(current.title(l))
        .show_ui(ui, |ui| {
            let mut last_cat: Option<&str> = None;
            for item in ActiveGallery::ALL {
                let cat = item.category(l);
                if last_cat != Some(cat) {
                    if last_cat.is_some() {
                        ui.separator();
                    }
                    ui.label(
                        egui::RichText::new(cat)
                            .small()
                            .strong()
                            .color(Color32::GRAY),
                    );
                    last_cat = Some(cat);
                }
                ui.selectable_value(current, item, item.title(l));
            }
        });

    *current != before
}

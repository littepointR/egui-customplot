use std::sync::Arc;

use egui::{AsIdSalt, IdSalt};

use crate::{
    Axis3D, BoxStyle3D, BubbleLegend, Camera3D, ColorScale, ConstantPlane3D, DataAspect3D,
    DepthMode, PlotStyle, PlotTheme, Projection3D, Text3D,
};

/// 3D Cartesian axes widget (`plot3`, `surf`, `mesh`, `contour3`, `fimplicit3`, `volshow`, `pcshow`, etc.).
///
/// 三维笛卡尔绘图控件：支持轨道旋转、缩放/平移、正交/透视投影、画家排序与 `ZBuffer` 像素级深度遮挡、体渲染及点云。
pub struct Plot3D {
    id_salt: IdSalt,
    height: f32,
    title: String,
    axes: [Axis3D; 3],
    camera: Camera3D,
    lens: Option<crate::CameraLens3D>,
    explicit_projection: Option<Projection3D>,
    aspect: DataAspect3D,
    depth_mode: DepthMode,
    box_style: BoxStyle3D,
    planes: Arc<[ConstantPlane3D]>,
    texts: Arc<[Text3D]>,
    bubble_legend: Option<BubbleLegend>,
    color_bar: Option<(ColorScale, String)>,
    controller: Option<Arc<dyn crate::CameraNavigation3D>>,
    allow_scroll_zoom: bool,
    decorations: crate::layout::DecorationSlots,
    legend_component: Option<Arc<dyn crate::LegendComponent>>,
    show_legend: bool,
    show_hover: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl Plot3D {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 300.0,
            title: String::new(),
            axes: [Axis3D::new(), Axis3D::new(), Axis3D::new()],
            camera: Camera3D::default(),
            lens: None,
            explicit_projection: None,
            aspect: DataAspect3D::Auto,
            depth_mode: DepthMode::SortedPainter,
            box_style: BoxStyle3D::BackPlanes,
            planes: Arc::from([]),
            texts: Arc::from([]),
            bubble_legend: None,
            color_bar: None,
            controller: None,
            allow_scroll_zoom: false,
            decorations: Default::default(),
            legend_component: None,
            show_legend: true,
            show_hover: true,
            pick_radius: 10.0,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            theme: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.height = height.max(32.0);
        }
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn x_axis(mut self, axis: Axis3D) -> Self {
        self.axes[0] = axis;
        self
    }
    pub fn y_axis(mut self, axis: Axis3D) -> Self {
        self.axes[1] = axis;
        self
    }
    pub fn z_axis(mut self, axis: Axis3D) -> Self {
        self.axes[2] = axis;
        self
    }
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.axes[0] = self.axes[0].clone().label(label);
        self
    }
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.axes[1] = self.axes[1].clone().label(label);
        self
    }
    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.axes[2] = self.axes[2].clone().label(label);
        self
    }
    pub fn camera(mut self, mut camera: Camera3D) -> Self {
        if let Some(proj) = self.explicit_projection {
            camera.projection = proj;
        }
        self.camera = camera;
        self
    }
    pub fn view(mut self, azimuth_deg: f64, elevation_deg: f64) -> Self {
        self.camera.azimuth_deg = azimuth_deg;
        self.camera.elevation_deg = elevation_deg.clamp(-90.0, 90.0);
        self
    }
    pub fn projection(mut self, projection: Projection3D) -> Self {
        self.camera.projection = projection;
        self.explicit_projection = Some(projection);
        self
    }
    /// Replace the camera-to-image lens while retaining orbit, pan and depth.
    pub fn lens(mut self, lens: crate::CameraLens3D) -> Self {
        self.lens = Some(lens);
        self
    }
    pub fn data_aspect(mut self, aspect: DataAspect3D) -> Self {
        self.aspect = aspect;
        self
    }
    pub fn depth_mode(mut self, mode: DepthMode) -> Self {
        self.depth_mode = mode;
        self
    }
    pub fn box_style(mut self, style: BoxStyle3D) -> Self {
        self.box_style = style;
        self
    }
    pub fn constant_planes(mut self, planes: impl Into<Arc<[ConstantPlane3D]>>) -> Self {
        self.planes = planes.into();
        self
    }
    pub fn texts(mut self, texts: impl Into<Arc<[Text3D]>>) -> Self {
        self.texts = texts.into();
        self
    }
    pub fn bubble_legend(mut self, legend: BubbleLegend) -> Self {
        self.bubble_legend = Some(legend);
        self
    }
    pub fn title_component(mut self, component: Arc<dyn crate::PlotDecoration>) -> Self {
        self.decorations.title = Some(component);
        self
    }
    pub fn color_bar_component(mut self, component: Arc<dyn crate::PlotDecoration>) -> Self {
        self.decorations.color_bar = Some(component);
        self
    }
    pub fn color_bar(mut self, scale: ColorScale, label: impl Into<String>) -> Self {
        self.color_bar = Some((scale, label.into()));
        self
    }
    pub fn camera_navigation(mut self, controller: Arc<dyn crate::CameraNavigation3D>) -> Self {
        self.controller = Some(controller);
        self
    }
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    /// Use a host-measured legend with custom entry widgets.
    pub fn legend_component(mut self, component: Arc<dyn crate::LegendComponent>) -> Self {
        self.legend_component = Some(component);
        self.show_legend = true;
        self
    }
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }
    pub fn show_hover(mut self, show: bool) -> Self {
        self.show_hover = show;
        self
    }
    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style;
        self
    }
    pub fn theme(mut self, theme: impl Into<Arc<PlotTheme>>) -> Self {
        self.theme = Some(theme.into());
        self
    }
}

mod show;

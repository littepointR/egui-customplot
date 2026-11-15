use std::f64::consts::TAU;

use crate::{ContourData, PlotBounds, PlotData};

mod adaptive;
mod implicit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FunctionPlotOptions {
    pub initial_samples: usize,
    pub max_depth: u8,
    pub max_evaluations: usize,
    pub tolerance: f64,
}

impl Default for FunctionPlotOptions {
    fn default() -> Self {
        Self {
            initial_samples: 65,
            max_depth: 10,
            max_evaluations: 4096,
            tolerance: 1e-3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImplicitOptions {
    pub coarse_grid: [usize; 2],
    pub max_depth: u8,
    pub max_evaluations: usize,
}

impl Default for ImplicitOptions {
    fn default() -> Self {
        Self {
            coarse_grid: [33, 33],
            max_depth: 4,
            max_evaluations: 16_384,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionPlotError {
    InvalidRange,
    InvalidOptions,
    BudgetExceeded {
        required: usize,
        max_evaluations: usize,
    },
}

impl std::fmt::Display for FunctionPlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRange => {
                f.write_str("function domain min must be finite and less than max")
            }
            Self::InvalidOptions => f.write_str("invalid adaptive function sampling options"),
            Self::BudgetExceeded {
                required,
                max_evaluations,
            } => write!(
                f,
                "initial function grid requires {required} evaluations, exceeding budget {max_evaluations}"
            ),
        }
    }
}

impl std::error::Error for FunctionPlotError {}

/// Immutable host-owned adaptively sampled explicit or parametric 2D curve (`fplot` / `ezplot`).
#[derive(Clone, Debug)]
pub struct FunctionCurveData {
    data: PlotData,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
}

impl FunctionCurveData {
    pub fn explicit(
        x_range: [f64; 2],
        options: FunctionPlotOptions,
        mut f: impl FnMut(f64) -> f64,
    ) -> Result<Self, FunctionPlotError> {
        adaptive::sample_explicit(x_range, options, &mut f)
    }

    pub fn parametric(
        t_range: [f64; 2],
        options: FunctionPlotOptions,
        mut f: impl FnMut(f64) -> [f64; 2],
    ) -> Result<Self, FunctionPlotError> {
        adaptive::sample_parametric(t_range, options, &mut f)
    }

    pub fn ezplot_explicit(
        max_evaluations: usize,
        f: impl FnMut(f64) -> f64,
    ) -> Result<Self, FunctionPlotError> {
        let options = FunctionPlotOptions {
            max_evaluations,
            ..Default::default()
        };
        Self::explicit([-TAU, TAU], options, f)
    }

    pub fn ezplot_parametric(
        max_evaluations: usize,
        f: impl FnMut(f64) -> [f64; 2],
    ) -> Result<Self, FunctionPlotError> {
        let options = FunctionPlotOptions {
            max_evaluations,
            ..Default::default()
        };
        Self::parametric([0.0, TAU], options, f)
    }

    pub fn data(&self) -> &PlotData {
        &self.data
    }
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }
    pub fn discontinuities(&self) -> usize {
        self.discontinuities
    }
    pub fn budget_truncated(&self) -> bool {
        self.budget_truncated
    }
}

/// Immutable host-owned adaptive implicit zero-level curve `F(x, y) = 0` (`fimplicit` / `ezplot`).
#[derive(Clone, Debug)]
pub struct ImplicitCurveData {
    contours: ContourData,
    evaluations: usize,
    singular_cells: usize,
    budget_truncated: bool,
}

impl ImplicitCurveData {
    pub fn new(
        bounds: PlotBounds,
        options: ImplicitOptions,
        mut f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionPlotError> {
        implicit::sample_implicit(bounds, options, &mut f)
    }

    pub fn ezplot_implicit(
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionPlotError> {
        let bounds = PlotBounds::new([-TAU, -TAU], [TAU, TAU])
            .map_err(|_| FunctionPlotError::InvalidRange)?;
        let options = ImplicitOptions {
            max_evaluations,
            ..Default::default()
        };
        Self::new(bounds, options, f)
    }

    pub fn contours(&self) -> &ContourData {
        &self.contours
    }
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }
    pub fn singular_cells(&self) -> usize {
        self.singular_cells
    }
    pub fn budget_truncated(&self) -> bool {
        self.budget_truncated
    }
}

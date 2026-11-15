mod categorical;
mod parallel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultivariateError {
    Empty,
    ShapeMismatch,
    InvalidColumn,
    NoFiniteValues,
}

impl std::fmt::Display for MultivariateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid multivariate/categorical data: {self:?}")
    }
}
impl std::error::Error for MultivariateError {}

pub use categorical::{CategoricalHeatmapData, SortOrder, TableAggregation};
pub use parallel::{ParallelData, ParallelNormalization};

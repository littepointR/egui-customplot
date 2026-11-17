/// Connection between consecutive valid samples, in their original order.
/// Affects line geometry only: markers, auto bounds and picking remain raw.
///
/// 相邻有效样本之间的连接方式（按原始顺序，仅影响折线几何，不影响散点标记、自动边界与拾取）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineInterpolation {
    /// Straight segments (in transformed coordinates on log axes).
    ///
    /// 直线段连接（在对数坐标轴下于变换后坐标系中插值）。
    #[default]
    Linear,
    /// Hold `y[i]` to `x[i+1]`, then jump to `y[i+1]` (MATLAB-style stairs).
    /// Starts at the first sample and ends at the last; no extrapolated plateau.
    ///
    /// 先水平保持 `y[i]` 至 `x[i+1]` 再阶跃到 `y[i+1]`（MATLAB `stairs` 风格，首尾不外推平台）。
    StepAfter,
    /// Jump to `y[i+1]` at `x[i]`, then hold to `x[i+1]`. No endpoint extension.
    ///
    /// 在 `x[i]` 处先阶跃到 `y[i+1]` 再水平保持至 `x[i+1]`，首尾不外推。
    StepBefore,
}

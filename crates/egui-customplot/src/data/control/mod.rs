mod freq_response;
mod poly;
mod rlocus;
mod step;
pub use step::{StepResponseData, StepResponseError};

pub use freq_response::{FrequencyResponseData, StabilityMargins};
pub use poly::{ControlError, TransferFunction, poly_eval_complex, poly_roots_aberth};
pub use rlocus::RootLocusData;

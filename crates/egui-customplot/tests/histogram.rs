use egui_customplot::{HistogramData, HistogramError, HistogramNormalization as Norm};

#[test]
fn explicit_bins_use_half_open_edges_and_keep_last_edge_missing_and_outside_counts() {
    let h = HistogramData::new(
        &[-1.0, 0.0, 0.5, 1.0, 2.0, 3.0, 4.0, f64::NAN, f64::INFINITY],
        vec![0.0, 1.0, 3.0],
    )
    .unwrap();
    assert_eq!(h.counts(), &[2, 3]);
    assert_eq!(
        (
            h.included_samples(),
            h.non_finite_samples(),
            h.outside_samples()
        ),
        (5, 2, 2)
    );
    let bars = h.to_bars(Norm::Count).unwrap();
    assert_eq!(
        (
            bars.bars()[0].center,
            bars.bars()[0].width,
            bars.bars()[0].value
        ),
        (0.5, 1.0, 2.0)
    );
    assert_eq!(
        (
            bars.bars()[1].center,
            bars.bars()[1].width,
            bars.bars()[1].value
        ),
        (2.0, 2.0, 3.0)
    );
}

#[test]
fn normalization_handles_unequal_bins_cumulative_and_empty_observations() {
    let h = HistogramData::new(&[0.0, 0.5, 1.0, 2.0], vec![0.0, 1.0, 3.0]).unwrap();
    for (norm, values) in [
        (Norm::Count, [2.0, 2.0]),
        (Norm::Probability, [0.5, 0.5]),
        (Norm::Density, [0.5, 0.25]),
        (Norm::CumulativeCount, [2.0, 4.0]),
        (Norm::CumulativeProbability, [0.5, 1.0]),
    ] {
        let bars = h.to_bars(norm).unwrap();
        assert_eq!(
            bars.bars().iter().map(|b| b.value).collect::<Vec<_>>(),
            values
        );
    }
    let density = h.to_bars(Norm::Density).unwrap();
    assert_eq!(
        density
            .bars()
            .iter()
            .map(|b| b.width * b.value)
            .sum::<f64>(),
        1.0
    );
    for samples in [vec![], vec![f64::NAN, -1.0, 4.0]] {
        let h = HistogramData::new(&samples, vec![0.0, 1.0, 3.0]).unwrap();
        for norm in [
            Norm::Probability,
            Norm::Density,
            Norm::CumulativeProbability,
        ] {
            assert!(
                h.to_bars(norm)
                    .unwrap()
                    .bars()
                    .iter()
                    .all(|b| b.value == 0.0)
            );
        }
    }
}

#[test]
fn malformed_edges_and_unrepresentable_density_are_explicit_errors() {
    for edges in [
        vec![],
        vec![0.0],
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![0.0, f64::INFINITY],
        vec![-f64::MAX, f64::MAX],
    ] {
        assert_eq!(
            HistogramData::new(&[], edges).unwrap_err(),
            HistogramError::InvalidEdges
        );
    }
    let h = HistogramData::new(&[0.0], vec![0.0, f64::from_bits(1)]).unwrap();
    assert_eq!(
        h.to_bars(Norm::Density).unwrap_err(),
        HistogramError::UnrepresentableBin
    );
}

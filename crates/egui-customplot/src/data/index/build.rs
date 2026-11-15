use std::{mem::size_of, ops::Range};

use crate::PlotPoint;

use super::{BLOCK_SIZE, LineIndex, LineIndexError, tree::Extrema, valid};

impl LineIndex {
    pub(in crate::data) fn build(
        points: &[PlotPoint],
        budget_bytes: usize,
    ) -> Result<Self, LineIndexError> {
        let mut counts = [0usize; 4];
        let mut open = [false; 4];
        let mut previous_x = None;
        // Validate and size every domain's run table before allocating it.
        for (i, point) in points.iter().enumerate() {
            if valid(*point, 0) {
                if previous_x.is_some_and(|x| point[0] < x) {
                    return Err(LineIndexError::UnsortedX { point_index: i });
                }
                previous_x = Some(point[0]);
            } else {
                previous_x = None;
            }
            for mask in 0..4 {
                let next = valid(*point, mask);
                if next && !open[mask] {
                    counts[mask] += 1;
                }
                open[mask] = next;
            }
        }
        let leaf_count = points
            .len()
            .div_ceil(BLOCK_SIZE)
            .max(1)
            .checked_next_power_of_two()
            .ok_or(LineIndexError::SizeOverflow)?;
        let tree_len = leaf_count
            .checked_mul(2)
            .ok_or(LineIndexError::SizeOverflow)?;
        let run_count = counts
            .iter()
            .try_fold(0usize, |total, n| total.checked_add(*n))
            .ok_or(LineIndexError::SizeOverflow)?;
        let bytes = tree_len
            .checked_mul(size_of::<Extrema>())
            .and_then(|tree| {
                run_count
                    .checked_mul(size_of::<Range<usize>>())
                    .and_then(|runs| tree.checked_add(runs))
            })
            .and_then(|payload| payload.checked_add(size_of::<Self>()))
            .ok_or(LineIndexError::SizeOverflow)?;
        check_budget(bytes, budget_bytes)?;
        let mut runs: [Vec<Range<usize>>; 4] = std::array::from_fn(|_| Vec::new());
        for (runs, count) in runs.iter_mut().zip(counts) {
            reserve(runs, count)?;
        }
        let mut tree = Vec::new();
        reserve(&mut tree, tree_len)?;
        // Account actual Vec capacities too; no geometric growth is used.
        let allocated = size_of::<Self>()
            + tree.capacity() * size_of::<Extrema>()
            + runs
                .iter()
                .map(|r| r.capacity() * size_of::<Range<usize>>())
                .sum::<usize>();
        check_budget(allocated, budget_bytes)?;
        tree.resize(tree_len, Extrema::default());
        let mut starts = [None; 4];
        for (i, point) in points.iter().enumerate() {
            if valid(*point, 0) {
                let leaf = leaf_count + i / BLOCK_SIZE;
                tree[leaf] = tree[leaf].merge(Extrema::point(i), points);
            }
            for mask in 0..4 {
                if valid(*point, mask) {
                    starts[mask].get_or_insert(i);
                } else if let Some(start) = starts[mask].take() {
                    runs[mask].push(start..i);
                }
            }
        }
        for (mask, start) in starts.into_iter().enumerate() {
            if let Some(start) = start {
                runs[mask].push(start..points.len());
            }
        }
        for i in (1..leaf_count).rev() {
            tree[i] = tree[i * 2].merge(tree[i * 2 + 1], points);
        }
        Ok(Self {
            runs: runs.map(Vec::into_boxed_slice),
            tree: tree.into_boxed_slice(),
            leaf_count,
            bytes,
        })
    }
}

fn check_budget(required_bytes: usize, budget_bytes: usize) -> Result<(), LineIndexError> {
    if required_bytes > budget_bytes {
        Err(LineIndexError::BudgetExceeded {
            required_bytes,
            budget_bytes,
        })
    } else {
        Ok(())
    }
}

fn reserve<T>(vec: &mut Vec<T>, count: usize) -> Result<(), LineIndexError> {
    vec.try_reserve_exact(count)
        .map_err(|_| LineIndexError::AllocationFailed)
}

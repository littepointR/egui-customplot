//! Wall-clock streaming stress of a bounded host queue and the public API.
//! No GPU/window. The host explicitly drops NEW batches when its queue is full.
use std::{
    collections::VecDeque,
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, TrySendError},
    },
    thread,
    time::{Duration, Instant},
};

use egui_customplot::{Line, LineSampling, Plot, PlotCache, PlotData, PlotState, PlotStream};

const CAPACITY: usize = 65_536;
const BATCH_SIZE: usize = 256;
const QUEUE_CAPACITY: usize = 4;
const CACHE_BUDGET: usize = 1024 * 1024;

struct Batch {
    id: u64,
    points: [[f64; 2]; BATCH_SIZE],
}

#[derive(Default)]
struct ProducerCounts {
    sent: AtomicU64,
    dropped: AtomicU64,
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_STRESS_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(1800.0);
    println!(
        "# stream_stress; seconds={seconds}; queue_capacity={QUEUE_CAPACITY}; batch_points={BATCH_SIZE}; ring_capacity={CAPACITY}; snapshot_limit=2; cache_budget={CACHE_BUDGET}"
    );
    println!(
        "# Host producer tries every 1ms, consumer sleeps 33ms; queue full drops NEW batch; batch boundaries contain NaN. CPU-only, shared machine, not a frame-rate benchmark."
    );
    println!(
        "elapsed_s,frames,received_batches,producer_sent,producer_dropped,ring_points,ring_bytes,snapshot_count,snapshot_sample_bytes,snapshot_index_bytes,cache_entries,cache_bytes,recent_work_p95_ms"
    );
    let (send, receive) = mpsc::sync_channel::<Batch>(QUEUE_CAPACITY);
    let stop = Arc::new(AtomicBool::new(false));
    let counts = Arc::new(ProducerCounts::default());
    let producer = start_producer(send, stop.clone(), counts.clone());
    let mut ring = PlotStream::new(CAPACITY, CAPACITY * 16).unwrap();
    let mut snapshots = VecDeque::<PlotData>::new();
    let mut cache = PlotCache::new(CACHE_BUDGET);
    let mut state = PlotState::default();
    let ctx = egui::Context::default();
    let start = Instant::now();
    let mut next_log = 10.0;
    let mut frames = 0u64;
    let mut received = 0u64;
    let mut recent_work = Vec::new();
    while start.elapsed().as_secs_f64() < seconds {
        let work_start = Instant::now();
        let mut drained = 0;
        // Bound work per display update as well as the queue's storage.
        for batch in receive.try_iter().take(QUEUE_CAPACITY) {
            assert!(ring.append_once(batch.id, &batch.points).unwrap().applied);
            assert!(!ring.append_once(batch.id, &batch.points).unwrap().applied);
            drained += 1;
            received += 1;
        }
        if drained != 0 {
            let mut data = ring.snapshot();
            data.build_line_index(1024 * 1024).unwrap();
            if snapshots.len() == 2 {
                snapshots.pop_front();
            }
            snapshots.push_back(data);
        }
        assert_eq!(ring.revision(), received);
        assert_eq!(ring.allocated_bytes(), CAPACITY * 16);
        assert!(ring.len() <= CAPACITY && snapshots.len() <= 2);
        if let Some(data) = snapshots.back() {
            let version = data.version();
            let mut passes = 0;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    assert_eq!(data.version(), version);
                    let response = Plot::new("stress").height(500.0).show_cached(
                        ui,
                        &mut state,
                        &[Line::new(data).sampling(LineSampling::MinMax)],
                        &mut cache,
                    );
                    assert_eq!(response.diagnostics.input_points, ring.len());
                    if passes == 0 {
                        ui.ctx().request_discard("stress multi-pass");
                    }
                    passes += 1;
                },
            );
            assert_eq!(passes, 2);
            black_box(ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point));
            output.drop_without_applying_deltas();
        }
        assert!(cache.retained_bytes() <= CACHE_BUDGET && cache.entry_count() <= 64);
        frames += 1;
        // Repeatedly release all retained derived geometry and view state.
        if frames.is_multiple_of(300) {
            cache.set_budget(0);
            assert_eq!(cache.retained_bytes(), 0);
            assert_eq!(cache.entry_count(), 0);
            cache.set_budget(CACHE_BUDGET);
            state = PlotState::default();
        }
        recent_work.push(work_start.elapsed().as_secs_f64() * 1000.0);
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed >= next_log {
            log(
                elapsed,
                frames,
                received,
                &counts,
                &ring,
                &snapshots,
                &cache,
                &mut recent_work,
            );
            recent_work.clear();
            next_log += 10.0;
        }
        thread::sleep(Duration::from_millis(33));
    }
    stop.store(true, Ordering::Relaxed);
    producer.join().expect("producer thread must finish");
    let queued_at_end = receive.try_iter().count() as u64;
    let sent = counts.sent.load(Ordering::Relaxed);
    assert_eq!(sent, received + queued_at_end);
    assert!(
        counts.dropped.load(Ordering::Relaxed) > 0,
        "overload must be exercised"
    );
    log(
        start.elapsed().as_secs_f64(),
        frames,
        received,
        &counts,
        &ring,
        &snapshots,
        &cache,
        &mut recent_work,
    );
    snapshots.clear();
    cache.clear();
    assert_eq!(
        (snapshots.len(), cache.entry_count(), cache.retained_bytes()),
        (0, 0, 0)
    );
    println!(
        "# completed=true; queued_batches_discarded_at_shutdown={queued_at_end}; post_clear_snapshots=0; post_clear_cache_bytes=0; ring_drop_follows_main"
    );
}

fn start_producer(
    send: mpsc::SyncSender<Batch>,
    stop: Arc<AtomicBool>,
    counts: Arc<ProducerCounts>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut id = 0u64;
        while !stop.load(Ordering::Relaxed) {
            let batch = Batch {
                id,
                points: std::array::from_fn(|i| {
                    let x = (id * BATCH_SIZE as u64 + i as u64) as f64 * 0.001;
                    if i == 0 { [f64::NAN; 2] } else { [x, x.sin()] }
                }),
            };
            match send.try_send(batch) {
                Ok(()) => {
                    counts.sent.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Full(_)) => {
                    counts.dropped.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Disconnected(_)) => break,
            }
            id += 1;
            thread::sleep(Duration::from_millis(1));
        }
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "one CSV record of the distinct resource owners"
)]
fn log(
    elapsed: f64,
    frames: u64,
    received: u64,
    counts: &ProducerCounts,
    ring: &PlotStream,
    snapshots: &VecDeque<PlotData>,
    cache: &PlotCache,
    work: &mut [f64],
) {
    work.sort_unstable_by(f64::total_cmp);
    let p95 = if work.is_empty() {
        0.0
    } else {
        work[((work.len() - 1) as f64 * 0.95).round() as usize]
    };
    println!(
        "{elapsed:.3},{frames},{received},{},{},{},{},{},{},{},{},{},{p95:.4}",
        counts.sent.load(Ordering::Relaxed),
        counts.dropped.load(Ordering::Relaxed),
        ring.len(),
        ring.allocated_bytes(),
        snapshots.len(),
        snapshots.iter().map(PlotData::sample_bytes).sum::<usize>(),
        snapshots
            .iter()
            .map(PlotData::line_index_bytes)
            .sum::<usize>(),
        cache.entry_count(),
        cache.retained_bytes()
    );
}

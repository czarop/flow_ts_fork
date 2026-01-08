use once_cell::sync::Lazy;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_PLOT_POOL_THREADS: usize = 4;
// Default to 1 because Plotters/font rendering has historically been prone to
// global locks and thread-safety issues; parallelism still happens *within* a job.
pub const DEFAULT_MAX_CONCURRENT_PLOT_JOBS: usize = 1;

fn env_usize(name: &str) -> Option<usize> {
    std::env::var(name).ok().and_then(|v| v.parse::<usize>().ok())
}

fn plot_pool_threads() -> usize {
    env_usize("FLOW_PLOT_POOL_THREADS").unwrap_or(DEFAULT_PLOT_POOL_THREADS)
}

fn max_concurrent_plot_jobs() -> usize {
    env_usize("FLOW_MAX_CONCURRENT_PLOT_JOBS").unwrap_or(DEFAULT_MAX_CONCURRENT_PLOT_JOBS)
}

static PLOT_POOL: Lazy<ThreadPool> = Lazy::new(|| {
    let threads = plot_pool_threads();
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("flow-plot-{i}"))
        .build()
        .expect("failed to build plot thread pool")
});

struct PlotLimiter {
    max_in_flight: usize,
    in_flight: Mutex<usize>,
    cv: Condvar,
}

impl PlotLimiter {
    fn new(max_in_flight: usize) -> Self {
        Self {
            max_in_flight: max_in_flight.max(1),
            in_flight: Mutex::new(0),
            cv: Condvar::new(),
        }
    }

    fn acquire(&'static self, label: &str) -> PlotPermit {
        let started_waiting = Instant::now();
        let mut guard = self.in_flight.lock().expect("plot limiter mutex poisoned");

        while *guard >= self.max_in_flight {
            // Wake periodically so we can log long waits (helps diagnose lockups).
            let (g, _) = self
                .cv
                .wait_timeout(guard, Duration::from_millis(250))
                .expect("plot limiter mutex poisoned while waiting");
            guard = g;

            let waited = started_waiting.elapsed();
            if waited >= Duration::from_secs(2) {
                eprintln!(
                    "⏳ [PLOT_LIMITER] Waiting {:?} for permit (job={label}, in_flight={}, max={})",
                    waited, *guard, self.max_in_flight
                );
            }
        }

        *guard += 1;
        drop(guard);

        PlotPermit {
            limiter: self,
            label: label.to_string(),
            acquired_at: Instant::now(),
        }
    }

    fn release(&self) {
        let mut guard = self.in_flight.lock().expect("plot limiter mutex poisoned");
        *guard = guard.saturating_sub(1);
        drop(guard);
        self.cv.notify_one();
    }
}

struct PlotPermit {
    limiter: &'static PlotLimiter,
    label: String,
    acquired_at: Instant,
}

impl Drop for PlotPermit {
    fn drop(&mut self) {
        let held = self.acquired_at.elapsed();
        if held >= Duration::from_secs(2) {
            eprintln!("⏱️  [PLOT_LIMITER] Released permit after {:?} (job={})", held, self.label);
        }
        self.limiter.release();
    }
}

static PLOT_LIMITER: Lazy<PlotLimiter> =
    Lazy::new(|| PlotLimiter::new(max_concurrent_plot_jobs()));

static PLOT_RENDER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn plot_pool_thread_count() -> usize {
    PLOT_POOL.current_num_threads()
}

/// Serialize calls into the Plotters drawing pipeline.
///
/// Even if we parallelize data preparation, rendering/encoding tends to hit
/// global caches/locks (fonts, backend resources). Keeping it single-threaded
/// avoids rare low-CPU deadlocks.
pub fn with_render_lock<R>(f: impl FnOnce() -> R) -> R {
    let _guard = PLOT_RENDER_LOCK.lock().expect("plot render mutex poisoned");
    f()
}

/// Run a single "plot job".
///
/// Guarantees:
/// - bounded number of concurrent plot jobs (queueing via Condvar)
/// - all Rayon parallelism inside `job` uses a dedicated plotting pool
pub fn run_plot_job_named<R: Send>(label: &str, job: impl FnOnce() -> R + Send) -> R {
    let _permit = PLOT_LIMITER.acquire(label);
    PLOT_POOL.install(job)
}

pub fn run_plot_job<R: Send>(job: impl FnOnce() -> R + Send) -> R {
    run_plot_job_named("plot_job", job)
}

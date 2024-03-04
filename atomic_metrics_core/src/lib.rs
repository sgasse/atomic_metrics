use anyhow::{bail, Result};
use glob::glob;
use regex::Regex;
use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::Path,
    process,
};

/// Get the counter `name` as borrow of the atomic value.
#[macro_export]
macro_rules! get_counter {
    ($name:ident) => {
        &METRICS_RECORDER.$name
    };
}

/// Increment the counter `name` by `value`.
#[macro_export]
macro_rules! increment_metric {
    ($name:ident, $value:expr) => {
        METRICS_RECORDER
            .$name
            .fetch_add($value, std::sync::atomic::Ordering::Relaxed)
    };
}

/// Increment the counter `name` by one.
#[macro_export]
macro_rules! tick_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    };
}

/// Set the counter `name` to `value`.
#[macro_export]
macro_rules! set_metric {
    ($name:ident, $value:expr) => {
        METRICS_RECORDER
            .$name
            .store($value, std::sync::atomic::Ordering::Relaxed)
    };
}

/// Reset the counter `name` to zero.
#[macro_export]
macro_rules! reset_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .store(0, std::sync::atomic::Ordering::Relaxed)
    };
}

/// Load the value of the counter `name`.
#[macro_export]
macro_rules! load_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .load(std::sync::atomic::Ordering::Relaxed)
    };
}

/// Generate the global `MetricsRecorder` based on all metrics usages in the source directory.
pub fn generate_metrics_recorder() -> Result<()> {
    println!("cargo:rerun-if-changed=src/");
    let metric_names = get_metric_names("src/**/*.rs")?;

    generate_metrics_recorder_with_names(metric_names.iter().map(|x| x.as_str()))
}

/// Generate the global `MetricsRecorder` with all the metrics names passed.
///
/// There will be a compilation error if you try to access/modify a metric not mentioned here.
pub fn generate_metrics_recorder_with_names<'a>(
    metric_names: impl Iterator<Item = &'a str> + Clone,
) -> Result<()> {
    let output = Path::new(&env::var("OUT_DIR")?).join("metrics.rs");
    let mut out = io::BufWriter::new(fs::File::create(&output)?);

    writeln!(out, "use std::sync::atomic::AtomicU64;")?;
    writeln!(out)?;
    writeln!(out, "pub struct MetricsRecorder {{")?;

    for metric in metric_names.clone() {
        writeln!(out, "pub {metric}: AtomicU64,")?;
    }

    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "impl MetricsRecorder {{")?;
    writeln!(out, "pub const fn new() -> Self {{")?;
    writeln!(out, "Self {{")?;

    for metric in metric_names {
        writeln!(out, "{metric}: AtomicU64::new(0),")?;
    }

    writeln!(out, "}}")?;
    writeln!(out, "}}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(
        out,
        "pub static METRICS_RECORDER: MetricsRecorder = MetricsRecorder::new();"
    )?;

    drop(out);

    let output = process::Command::new("rustfmt").arg(&output).output()?;
    if !output.status.success() {
        bail!(
            "failed to format generated code:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

static GET_COUNTER_REGEX: &str = r"get_counter!\([\n]?[\s]*([\d\w]+)[)\n,]";
static INCREMENT_METRIC_REGEX: &str = r"increment_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static TICK_METRIC_REGEX: &str = r"tick_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static SET_METRIC_REGEX: &str = r"set_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static RESET_METRIC_REGEX: &str = r"reset_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static LOAD_METRIC_REGEX: &str = r"load_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";

/// Extract metric names by sifting through the files in the glob pattern for macro usages.
fn get_metric_names(pattern: &str) -> Result<Vec<String>> {
    let src_files = glob(pattern)?;

    let regexes = [
        Regex::new(GET_COUNTER_REGEX).expect("failed to compile regex"),
        Regex::new(INCREMENT_METRIC_REGEX).expect("failed to compile regex"),
        Regex::new(TICK_METRIC_REGEX).expect("failed to compile regex"),
        Regex::new(SET_METRIC_REGEX).expect("failed to compile regex"),
        Regex::new(RESET_METRIC_REGEX).expect("failed to compile regex"),
        Regex::new(LOAD_METRIC_REGEX).expect("failed to compile regex"),
    ];

    let mut metric_names = HashSet::new();

    for src_file in src_files.filter_map(|x| x.ok()) {
        if let Ok(contents) = fs::read_to_string(src_file) {
            for re in regexes.iter() {
                for captures in re.captures_iter(&contents) {
                    if let Some(name) = captures.get(1) {
                        metric_names.insert(name.as_str().to_owned());
                    }
                }
            }
        }
    }

    let mut metric_names: Vec<_> = metric_names.into_iter().collect();
    metric_names.sort();

    Ok(metric_names)
}

use anyhow::Result;
use glob::glob;
use regex::Regex;
use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::Path,
    process,
};

#[macro_export]
macro_rules! get_counter {
    ($name:ident) => {
        &METRICS_RECORDER.$name
    };
}

#[macro_export]
macro_rules! increment_metric {
    ($name:ident, $value:expr) => {
        METRICS_RECORDER
            .$name
            .fetch_add($value, std::sync::atomic::Ordering::Relaxed)
    };
}

#[macro_export]
macro_rules! tick_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    };
}

#[macro_export]
macro_rules! set_metric {
    ($name:ident, $value:expr) => {
        METRICS_RECORDER
            .$name
            .store($value, std::sync::atomic::Ordering::Relaxed)
    };
}

#[macro_export]
macro_rules! reset_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .store(0, std::sync::atomic::Ordering::Relaxed)
    };
}

#[macro_export]
macro_rules! load_metric {
    ($name:ident) => {
        METRICS_RECORDER
            .$name
            .load(std::sync::atomic::Ordering::Relaxed)
    };
}

pub fn generate_metrics_facade() -> Result<()> {
    println!("cargo:rerun-if-changed=src/");

    let metric_names = get_metric_names()?;

    let output = Path::new(&env::var("OUT_DIR")?).join("metrics.rs");
    let mut out = io::BufWriter::new(fs::File::create(&output)?);

    writeln!(out, "use std::sync::atomic::AtomicU64;")?;
    writeln!(out)?;
    writeln!(out, "pub struct MetricsRecorder {{")?;

    for metric in metric_names.iter() {
        writeln!(out, "pub {metric}: AtomicU64,")?;
    }

    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "impl MetricsRecorder {{")?;
    writeln!(out, "pub const fn new() -> Self {{")?;
    writeln!(out, "Self {{")?;

    for metric in metric_names.iter() {
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

    process::Command::new("rustfmt").arg(&output).output()?;

    Ok(())
}

static GET_COUNTER_REGEX: &str = r"get_counter!\([\n]?[\s]*([\d\w]+)[)\n,]";
static INCREMENT_METRIC_REGEX: &str = r"increment_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static TICK_METRIC_REGEX: &str = r"tick_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static SET_METRIC_REGEX: &str = r"set_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static RESET_METRIC_REGEX: &str = r"reset_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";
static LOAD_METRIC_REGEX: &str = r"load_metric!\([\n]?[\s]*([\d\w]+)[)\n,]";

fn get_metric_names() -> Result<Vec<String>> {
    let src_files = glob("src/**/*.rs")?;

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

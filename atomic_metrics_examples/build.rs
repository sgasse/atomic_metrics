use anyhow::Result;

fn main() -> Result<()> {
    atomic_metrics_core::generate_metrics_recorder()
}

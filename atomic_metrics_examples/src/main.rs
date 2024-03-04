use atomic_metrics_core::{
    get_counter, increment_metric, load_metric, reset_metric, set_metric, tick_metric,
};
use atomic_metrics_examples::METRICS_RECORDER;

fn main() {
    println!("Examples of atomic metrics");

    let value = get_counter!(value);
    increment_metric!(value_inc, 3);
    tick_metric!(value_tick);
    tick_metric!(value_tick);
    set_metric!(value_set, 7);

    dbg!(value);
    dbg!(load_metric!(value_inc));
    dbg!(load_metric!(value_tick));
    dbg!(load_metric!(value_set));
    dbg!(load_metric!(value_only_loaded));

    let _long_name_counter = get_counter!(
        really_long_name3_________________________________________________________________________
    );

    reset_metric!(value_inc);
    dbg!(load_metric!(value_inc));
}

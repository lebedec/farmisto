use prometheus::{
    register_gauge_vec_with_registry, register_histogram_vec_with_registry,
    register_int_counter_with_registry, register_int_gauge_vec_with_registry, GaugeVec,
    HistogramVec, IntCounter, IntGauge, IntGaugeVec, Registry,
};

pub struct SceneMetrics {
    pub draw: GaugeVec,
    pub text: GaugeVec,
    pub draw_calls: IntGaugeVec,
    pub frames: IntCounter,
}

impl SceneMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let draw = register_gauge_vec_with_registry!(
            "scene_draw_seconds",
            "scene_draw_seconds",
            &["pipeline"],
            registry
        )?;
        let text = register_gauge_vec_with_registry!(
            "scene_text_seconds",
            "scene_text_seconds",
            &["length"],
            registry
        )?;
        let draw_calls = register_int_gauge_vec_with_registry!(
            "scene_draw_calls_total",
            "scene_draw_calls_total",
            &["pipeline"],
            registry
        )?;
        let frames = register_int_counter_with_registry!("scene_frames", "scene_frames", registry)?;
        let metrics = Self {
            draw,
            text,
            draw_calls,
            frames,
        };
        Ok(metrics)
    }
}

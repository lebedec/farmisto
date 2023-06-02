use prometheus::{register_gauge_vec_with_registry, GaugeVec, Registry};

pub struct GameplayMetrics {
    pub update: GaugeVec,
}

impl GameplayMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let update = register_gauge_vec_with_registry!(
            "gameplay_update_seconds",
            "gameplay_update_seconds",
            &["stage"],
            registry
        )?;
        let metrics = Self { update };
        Ok(metrics)
    }
}

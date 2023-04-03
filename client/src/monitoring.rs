use prometheus::HistogramVec;
use std::time::Instant;

pub struct Timer {
    instant: Instant,
}

impl Timer {
    pub fn now() -> Self {
        Self {
            instant: Instant::now(),
        }
    }

    pub fn time(&mut self) -> f64 {
        let elapsed = self.instant.elapsed();
        self.instant = Instant::now();
        elapsed.as_secs_f64()
    }

    pub fn record(&mut self, label: &str, histogram: &HistogramVec) {
        histogram.with_label_values(&[label]).observe(self.time());
    }

    pub fn record2(&mut self, label1: &str, label2: &str, histogram: &HistogramVec) {
        histogram
            .with_label_values(&[label1, label2])
            .observe(self.time());
    }
}

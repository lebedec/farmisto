use log::{error, info};
use prometheus::{labels, Encoder, GaugeVec, HistogramVec, Registry, TextEncoder};
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

pub trait TimerIntegration {
    fn record(&self, label: &str, timer: &mut Timer);
}

impl TimerIntegration for HistogramVec {
    fn record(&self, label: &str, timer: &mut Timer) {
        self.with_label_values(&[label]).observe(timer.time());
    }
}

impl TimerIntegration for GaugeVec {
    fn record(&self, label: &str, timer: &mut Timer) {
        self.with_label_values(&[label]).set(timer.time());
    }
}

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

    pub fn gauge(&mut self, label: &str, gauge: &GaugeVec) {
        gauge.with_label_values(&[label]).set(self.time());
    }

    pub fn gauge_with_labels(&mut self, labels: &[&str], gauge: &GaugeVec) {
        gauge.with_label_values(labels).set(self.time());
    }

    pub fn record2(&mut self, label1: &str, label2: &str, histogram: &HistogramVec) {
        histogram
            .with_label_values(&[label1, label2])
            .observe(self.time());
    }
}

pub fn spawn_prometheus_metrics_server() {
    thread::Builder::new()
        .name("prometheus".into())
        .spawn(|| {
            let encoder = TextEncoder::new();
            let listener = TcpListener::bind("127.0.0.1:9092").unwrap();
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let status_line = "HTTP/1.1 200 OK";
                let mut contents = String::new();
                let metric_families = prometheus::gather();
                encoder
                    .encode_utf8(&metric_families, &mut contents)
                    .unwrap();
                let length = contents.len();
                let response =
                    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
                stream.write_all(response.as_bytes()).unwrap();
            }
        })
        .unwrap();
}

static mut MONITORING_CONTEXT: Option<String> = None;

pub fn set_monitoring_context(player: &str) {
    info!("Set player monitoring context to '{player}'");
    unsafe {
        MONITORING_CONTEXT = Some(String::from(player));
    }
}

pub fn spawn_prometheus_metrics_pusher(gateway: String, registry: Registry) {
    thread::Builder::new()
        .name("prometheus-push".into())
        .spawn(move || loop {
            thread::sleep(Duration::from_millis(500));
            let metrics = registry.gather();

            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            encoder.encode(&metrics, &mut buffer).unwrap();

            if let Some(player) = unsafe { MONITORING_CONTEXT.as_ref() } {
                let auth = None;
                let result = prometheus::push_metrics(
                    "push",
                    labels! {"player".to_owned() => player.to_owned(),},
                    &gateway,
                    metrics,
                    auth,
                );

                if let Err(error) = result {
                    error!("Unable to push metrics, {error:?}");
                    break;
                }
            }
        })
        .unwrap();
}

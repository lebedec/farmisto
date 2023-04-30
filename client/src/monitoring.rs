use log::{error, info};
use prometheus::proto::MetricFamily;
use prometheus::{labels, Encoder, GaugeVec, HistogramVec, Registry, TextEncoder};
use std::collections::HashMap;
use std::hash::BuildHasher;
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
                // let auth = None;
                // let result = prometheus::push_metrics(
                //     "push",
                //     labels! {"player".to_owned() => player.to_owned(),},
                //     &gateway,
                //     metrics,
                //     auth,
                // );

                let result = push_metrics(
                    "push",
                    labels! {"player".to_owned() => player.to_owned(),},
                    &gateway,
                    metrics,
                );

                if let Err(error) = result {
                    error!("Unable to push metrics, {error:?}");
                    break;
                }
            }
        })
        .unwrap();
}

fn push_metrics<S: BuildHasher>(
    job: &str,
    grouping: HashMap<String, String, S>,
    url: &str,
    mfs: Vec<MetricFamily>,
) -> Result<(), prometheus::Error> {
    // Suppress clippy warning needless_pass_by_value.
    let grouping = grouping;

    let mut push_url = if url.contains("://") {
        url.to_owned()
    } else {
        format!("http://{}", url)
    };

    if push_url.ends_with('/') {
        push_url.pop();
    }

    let mut url_components = Vec::new();
    if job.contains('/') {
        return Err(prometheus::Error::Msg(format!("job contains '/': {}", job)));
    }

    // TODO: escape job
    url_components.push(job.to_owned());

    for (ln, lv) in &grouping {
        // TODO: check label name
        if lv.contains('/') {
            return Err(prometheus::Error::Msg(format!(
                "value of grouping label {} contains '/': {}",
                ln, lv
            )));
        }
        url_components.push(ln.to_owned());
        url_components.push(lv.to_owned());
    }

    push_url = format!("{}/metrics/job/{}", push_url, url_components.join("/"));

    let encoder = prometheus::ProtobufEncoder::new();
    let mut buf = Vec::new();

    for mf in mfs {
        // Check for pre-existing grouping labels:
        for m in mf.get_metric() {
            for lp in m.get_label() {
                if lp.get_name() == "job" {
                    return Err(prometheus::Error::Msg(format!(
                        "pushed metric {} already contains a \
                         job label",
                        mf.get_name()
                    )));
                }
                if grouping.contains_key(lp.get_name()) {
                    return Err(prometheus::Error::Msg(format!(
                        "pushed metric {} already contains \
                         grouping label {}",
                        mf.get_name(),
                        lp.get_name()
                    )));
                }
            }
        }
        // Ignore error, `no metrics` and `no name`.
        let _ = encoder.encode(&[mf], &mut buf);
    }

    let sending = ureq::put(&push_url)
        .set("content-type", encoder.format_type())
        .send_bytes(&buf);

    match sending {
        Ok(response) => {}
        Err(error) => {
            error!("Unable to push metrics {error}")
        }
    }

    Ok(())
}

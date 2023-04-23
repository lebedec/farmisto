use prometheus::{register_int_counter_with_registry, IntCounter, Registry};

#[derive(Clone)]
pub struct ClientMetrics {
    pub sent_bytes: IntCounter,
    pub received_bytes: IntCounter,
}

impl ClientMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let sent_bytes = register_int_counter_with_registry!(
            "client_sent_bytes",
            "client_sent_bytes",
            registry
        )?;

        let received_bytes = register_int_counter_with_registry!(
            "client_received_bytes",
            "client_received_bytes",
            registry
        )?;

        Ok(Self {
            sent_bytes,
            received_bytes,
        })
    }

    pub fn new_ai(registry: &Registry) -> Result<Self, prometheus::Error> {
        // TODO: remove from network module (it hack)
        let sent_bytes = register_int_counter_with_registry!(
            "ai_client_sent_bytes",
            "ai_client_sent_bytes",
            registry
        )?;

        let received_bytes = register_int_counter_with_registry!(
            "ai_client_received_bytes",
            "ai_client_received_bytes",
            registry
        )?;

        Ok(Self {
            sent_bytes,
            received_bytes,
        })
    }
}

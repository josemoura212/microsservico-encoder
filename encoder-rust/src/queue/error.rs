#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("connection failed: {0}")]
    Connection(#[from] lapin::Error),

    #[error("failed to declare queue '{queue}': {source}")]
    QueueDeclare { queue: String, source: lapin::Error },

    #[error("failed to register consumer '{consumer}': {source}")]
    ConsumerRegister {
        consumer: String,
        source: lapin::Error,
    },

    #[error("publish failed: {0}")]
    Publish(lapin::Error),

    #[error("channel unavailable — call connect() first")]
    ChannelUnavailable,
}

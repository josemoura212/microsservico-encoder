use std::env;

use crate::config::ConfigError;

pub const ENV_RABBITMQ_USER: &str = "RABBITMQ_DEFAULT_USER";
pub const ENV_RABBITMQ_PASS: &str = "RABBITMQ_DEFAULT_PASS";
pub const ENV_RABBITMQ_HOST: &str = "RABBITMQ_DEFAULT_HOST";
pub const ENV_RABBITMQ_PORT: &str = "RABBITMQ_DEFAULT_PORT";
pub const ENV_RABBITMQ_VHOST: &str = "RABBITMQ_DEFAULT_VHOST";
pub const ENV_CONSUMER_QUEUE_NAME: &str = "RABBITMQ_CONSUMER_QUEUE_NAME";
pub const ENV_CONSUMER_NAME: &str = "RABBITMQ_CONSUMER_NAME";
pub const ENV_DLX: &str = "RABBITMQ_DLX";
pub const ENV_NOTIFICATION_EX: &str = "RABBITMQ_NOTIFICATION_EX";
pub const ENV_NOTIFICATION_ROUTING_KEY: &str = "RABBITMQ_NOTIFICATION_ROUTING_KEY";

const DEFAULT_PORT: &str = "5672";
const DEFAULT_VHOST: &str = "/";
const DEFAULT_CONSUMER_QUEUE: &str = "videos";
const DEFAULT_CONSUMER_NAME: &str = "app-name";
const DEFAULT_DLX: &str = "dlx";
const DEFAULT_NOTIFICATION_EX: &str = "amq.direct";
const DEFAULT_NOTIFICATION_ROUTING_KEY: &str = "jobs";

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub vhost: String,
    pub consumer_queue_name: String,
    pub consumer_name: String,
    pub dlx: String,
    pub notification_exchange: String,
    pub notification_routing_key: String,
}

impl QueueConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let user = required_env(ENV_RABBITMQ_USER)?;
        let password = required_env(ENV_RABBITMQ_PASS)?;
        let host = required_env(ENV_RABBITMQ_HOST)?;

        let port = env::var(ENV_RABBITMQ_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
        let vhost = env::var(ENV_RABBITMQ_VHOST).unwrap_or_else(|_| DEFAULT_VHOST.to_string());
        let consumer_queue_name = env::var(ENV_CONSUMER_QUEUE_NAME)
            .unwrap_or_else(|_| DEFAULT_CONSUMER_QUEUE.to_string());
        let consumer_name =
            env::var(ENV_CONSUMER_NAME).unwrap_or_else(|_| DEFAULT_CONSUMER_NAME.to_string());
        let dlx = env::var(ENV_DLX).unwrap_or_else(|_| DEFAULT_DLX.to_string());
        let notification_exchange =
            env::var(ENV_NOTIFICATION_EX).unwrap_or_else(|_| DEFAULT_NOTIFICATION_EX.to_string());
        let notification_routing_key = env::var(ENV_NOTIFICATION_ROUTING_KEY)
            .unwrap_or_else(|_| DEFAULT_NOTIFICATION_ROUTING_KEY.to_string());

        Ok(QueueConfig {
            user,
            password,
            host,
            port,
            vhost,
            consumer_queue_name,
            consumer_name,
            dlx,
            notification_exchange,
            notification_routing_key,
        })
    }

    pub fn dsn(&self) -> String {
        let encoded_vhost = self.vhost.replace('/', "%2F");
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, encoded_vhost
        )
    }
}

fn required_env(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::Missing(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        for key in [
            ENV_RABBITMQ_USER,
            ENV_RABBITMQ_PASS,
            ENV_RABBITMQ_HOST,
            ENV_RABBITMQ_PORT,
            ENV_RABBITMQ_VHOST,
            ENV_CONSUMER_QUEUE_NAME,
            ENV_CONSUMER_NAME,
            ENV_DLX,
            ENV_NOTIFICATION_EX,
            ENV_NOTIFICATION_ROUTING_KEY,
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    fn set_required_env() {
        unsafe {
            env::set_var(ENV_RABBITMQ_USER, "guest");
            env::set_var(ENV_RABBITMQ_PASS, "guest");
            env::set_var(ENV_RABBITMQ_HOST, "localhost");
        }
    }

    #[test]
    fn from_env_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        set_required_env();

        let config = QueueConfig::from_env().unwrap();

        assert_eq!(config.user, "guest");
        assert_eq!(config.password, "guest");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, "5672");
        assert_eq!(config.vhost, "/");
        assert_eq!(config.consumer_queue_name, "videos");
        assert_eq!(config.consumer_name, "app-name");
        assert_eq!(config.dlx, "dlx");
        assert_eq!(config.notification_exchange, "amq.direct");
        assert_eq!(config.notification_routing_key, "jobs");
    }

    #[test]
    fn from_env_custom_values() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        set_required_env();

        unsafe {
            env::set_var(ENV_RABBITMQ_PORT, "5673");
            env::set_var(ENV_RABBITMQ_VHOST, "/dev");
            env::set_var(ENV_CONSUMER_QUEUE_NAME, "my-queue");
        }

        let config = QueueConfig::from_env().unwrap();

        assert_eq!(config.port, "5673");
        assert_eq!(config.vhost, "/dev");
        assert_eq!(config.consumer_queue_name, "my-queue");

        clear_env();
    }

    #[test]
    fn from_env_missing_user() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var(ENV_RABBITMQ_PASS, "guest");
            env::set_var(ENV_RABBITMQ_HOST, "localhost");
        }

        let result = QueueConfig::from_env();
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains(ENV_RABBITMQ_USER));

        clear_env();
    }

    #[test]
    fn from_env_missing_password() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var(ENV_RABBITMQ_USER, "guest");
            env::set_var(ENV_RABBITMQ_HOST, "localhost");
        }

        let result = QueueConfig::from_env();
        assert!(result.is_err());

        clear_env();
    }

    #[test]
    fn from_env_missing_host() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var(ENV_RABBITMQ_USER, "guest");
            env::set_var(ENV_RABBITMQ_PASS, "guest");
        }

        let result = QueueConfig::from_env();
        assert!(result.is_err());

        clear_env();
    }

    #[test]
    fn dsn_default_vhost() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        set_required_env();

        let config = QueueConfig::from_env().unwrap();
        assert_eq!(config.dsn(), "amqp://guest:guest@localhost:5672/%2F");

        clear_env();
    }

    #[test]
    fn dsn_custom_vhost() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        set_required_env();
        unsafe { env::set_var(ENV_RABBITMQ_VHOST, "/production") };

        let config = QueueConfig::from_env().unwrap();
        assert_eq!(config.dsn(), "amqp://guest:guest@localhost:5672/%2Fproduction");

        clear_env();
    }
}

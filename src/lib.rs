use async_nats::ConnectOptions;
use serde::Deserialize;
use std::{path::Path, time::Duration};

pub fn some_and_true(optional_bool: Option<bool>) -> bool {
    optional_bool.unwrap_or_default()
}

/**
 * Connection methods:
 * server / server[]
 * name
 *
 *  -- Basic Options
 *  ping_interval
 *  flush_interval // to send to server so maybe useless for us?
 *  no_echo
 *
 *  retry_on_failed_connect
 *  max_reconnects
 *  reconnect_buffer_size
 *  connection_timeout
 *
 *  subscription_capacity
 *  sender_capacity
 *
 *  inbox_prefix
 *  request_timeout
 *  retry_on_initial_connect
 *  ignore_discovered_servers
 *  retain_servers_order
 *  read_buffer_capacity
 *  
 *  pedantic // found this on nats site
 *  verbose // found this on nats site
 * */

/**
 * !important Auth Options
 * - No Auth
 * - User/Password
 * - Token
 * - Nkey
 * - Credential File (.creds)
*/
/** I guess these are TLS options?
    enable/disable
    pub(crate) certificates: Vec<PathBuf>,
    pub(crate) client_cert: Option<PathBuf>,
    pub(crate) client_key: Option<PathBuf>,
    pub(crate) tls_client_config: Option<rustls::ClientConfig>,
*/
#[derive(Deserialize)]
#[serde(untagged)]
enum ServerName {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Deserialize)]
struct PlainOptions {
    name: Option<String>,
    server: ServerName,

    ping_interval: Option<Duration>,
    flush_interval: Option<Duration>, // to send to server so maybe useless for us?
    no_echo: Option<bool>,
    retry_on_failed_connect: Option<bool>,
    max_reconnects: Option<usize>,
    // no option to change both below
    // reconnect_buffer_size: Option<usize>,
    // connection_timeout: Option<Duration>,
    subscription_capacity: Option<usize>,
    // method name is client_capacity
    sender_capacity: Option<usize>,

    inbox_prefix: Option<String>,
    request_timeout: Option<Duration>,
    retry_on_initial_connect: Option<bool>,
    ignore_discovered_servers: Option<bool>,
    retain_servers_order: Option<bool>,
    read_buffer_capacity: Option<u16>,
    connection_timeout: Option<Duration>,
    // pedantic // found this on nats site
    // verbose // found this on nats site
}

pub struct ConnectOptionsAdapter {
    pub(crate) connect_options: ConnectOptions,
    pub(crate) servers: ServerName,
    // to do: add auth options
    // pub(crate) auth: Option<String>,
}

impl ConnectOptionsAdapter {
    pub fn from(file_path: &Path) -> Self {
        let file = std::fs::File::open(file_path).unwrap();
        let options: PlainOptions = serde_yaml::from_reader(file).unwrap();
        let mut co = ConnectOptions::default();
        if let Some(name) = options.name {
            co = co.name(name);
        }

        if let Some(ping_interval) = options.ping_interval {
            co = co.ping_interval(ping_interval);
        }
        if let Some(flush_interval) = options.flush_interval {
            co = co.flush_interval(flush_interval);
        }
        if some_and_true(options.no_echo) {
            co = co.no_echo();
        }
        if some_and_true(options.retry_on_failed_connect) {
            co = co.retry_on_initial_connect(); // Todo: Ask about this; seems like this is same in official nats
        }
        /* Todo: ask about this
        if let Some(max_reconnects) = options.max_reconnects {
            co.max_reconnects = co.max_reconnects(max_reconnects);
        }
        if let Some(reconnect_buffer_size) = options.reconnect_buffer_size {
            co = co.with_reconnect_buffer_size(reconnect_buffer_size);
        }
        */
        if let Some(connection_timeout) = options.connection_timeout {
            co = co.connection_timeout(connection_timeout);
        }
        if let Some(subscription_capacity) = options.subscription_capacity {
            co = co.subscription_capacity(subscription_capacity);
        }
        if let Some(sender_capacity) = options.sender_capacity {
            co = co.client_capacity(sender_capacity); // is intentional. see official nats
        }
        if let Some(inbox_prefix) = options.inbox_prefix {
            co = co.custom_inbox_prefix(inbox_prefix);
        }
        if let Some(request_timeout) = options.request_timeout {
            co = co.request_timeout(Some(request_timeout));
        }
        if some_and_true(options.retry_on_initial_connect) {
            co = co.retry_on_initial_connect();
        }
        if some_and_true(options.ignore_discovered_servers) {
            co = co.ignore_discovered_servers();
        }
        if some_and_true(options.retain_servers_order) {
            co = co.retain_servers_order();
        }
        if let Some(read_buffer_capacity) = options.read_buffer_capacity {
            co = co.read_buffer_capacity(read_buffer_capacity);
        }
        ConnectOptionsAdapter {
            connect_options: co,
            servers: options.server,
        }
    }
}

impl ConnectOptionsAdapter {
    pub fn auth() {
        print!("Will be implemented soon");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_and_true_should_return_true_on_true() {
        assert_eq!(some_and_true(Some(true)), true);
    }

    #[test]
    fn some_and_true_should_return_false_on_false() {
        assert_eq!(some_and_true(Some(false)), false);
    }

    #[test]
    fn some_and_true_should_return_false_on_none() {
        assert_eq!(some_and_true(None), false);
    }

    #[test]
    fn convert_successful_mock_file() {
        let co = ConnectOptionsAdapter::from(Path::new("tests/mock_1.yml"));
    }
}

use async_nats::ConnectOptions;
use serde::{Deserialize, Serialize, Serializer};
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
enum AuthOptions {
    NoAuth,
    UserPassword(String, String),
    Token(String),
    Nkey(String),
    CredentialFile(String),
}
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
    server: ServerName,

    //-- auth section --//
    // I added this to deterministically create auth options
    auth_type: Option<String>,

    // with_user_and_password
    // co.user_password(user: string, pass: string)
    user: Option<String>,
    pass: Option<String>,

    // with_token
    // co.token(token: string)
    token: Option<String>,

    // with_nkey
    // co.nkey(seed: string)
    nkey: Option<String>,

    // with_credential_file
    // credentials_file(path: PathBuf)
    // self.credentials(&cred_file_contents) (?)
    credential_file: Option<String>,

    // JWT is also an option, but doesnt look serializable
    // Should be added as a plugin

    //
    // -- end auth section --//
    name: Option<String>,

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

fn parse_auth_options(po: &PlainOptions) -> AuthOptions {
    let auth_type = po.auth_type.clone();
    match auth_type {
        Some(auth_type) => match auth_type.as_str() {
            "no_auth" => AuthOptions::NoAuth,
            "user_password" => {
                let user = po.user.clone().unwrap();
                let password = po.pass.clone().unwrap();
                AuthOptions::UserPassword(user.to_string(), password.to_string())
            }
            "token" => {
                let token = po.token.clone().unwrap();
                AuthOptions::Token(token.to_string())
            }
            "nkey" => {
                let nkey = po.nkey.clone().unwrap();
                AuthOptions::Nkey(nkey.to_string())
            }
            "credential_file" => {
                let credential_path = po.credential_file.clone().unwrap();
                let credential_file = std::fs::read_to_string(credential_path).unwrap();
                AuthOptions::CredentialFile(credential_file.to_string())
            }
            _ => panic!("Invalid auth type"),
        },
        None => AuthOptions::NoAuth,
    }
}

pub struct ConnectOptionsAdapter {
    pub(crate) connect_options: ConnectOptions,
    pub(crate) servers: ServerName,
}

impl ConnectOptionsAdapter {
    fn cook_auth(&self, options: &PlainOptions) {}
    pub fn from(file_path: &Path) -> Self {
        let file = std::fs::File::open(file_path).unwrap();
        let options: PlainOptions = serde_yaml::from_reader(file).unwrap();
        let mut co = ConnectOptions::default();
        if let Some(name) = options.name.clone() {
            co = co.name(name);
        }
        // probably converted into some MACRO?
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
        if let Some(inbox_prefix) = options.inbox_prefix.clone() {
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

        let auth: AuthOptions = parse_auth_options(&options);

        match auth {
            AuthOptions::NoAuth => {}
            AuthOptions::UserPassword(user, password) => {
                co = co.user_and_password(user, password);
            }
            AuthOptions::Token(token) => {
                co = co.token(token);
            }
            AuthOptions::Nkey(nkey) => {
                co = co.nkey(nkey);
            }
            AuthOptions::CredentialFile(credential_file) => {
                co = co.credentials(&credential_file).unwrap();
            }
        }

        let adapter = ConnectOptionsAdapter {
            connect_options: co,
            servers: options.server,
        };

        adapter
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
        let _co = ConnectOptionsAdapter::from(Path::new("tests/mock_1.yml"));
        // to do: add more asserts
    }

    // todo: add more tests for auth_types
}

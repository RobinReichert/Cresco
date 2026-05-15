#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct LoginData {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<32>,
}

#[derive(PartialEq, Debug)]
pub enum WifiAction {
    Ignore,
    RetrieveCredentials,
    WaitForCredentials,
    EstablishConnection { credentials: LoginData },
    WaitForDisconnect,
}

pub enum WifiEvent {
    CredentialsMissing,
    CredentialsFound { credentials: LoginData },
    CredentialsReceived { credentials: LoginData },
    ConnectionEstablished,
    ConnectionNotEstablished,
    Disconnected,
    Timeout,
}

pub trait WifiManager {
    fn handle_event(&mut self, event: WifiEvent) -> WifiAction;
}

pub mod simple {
    use super::*;

    const MAX_CONNECT_RETRIES: usize = 3;

    #[derive(Debug)]
    pub enum SimpleWifiState {
        RetrievingCredentials,
        WaitingForCredentials,
        EstablishingConnection {
            retries: usize,
            credentials: LoginData,
        },
        Connected,
    }

    pub struct SimpleWifiManager {
        state: SimpleWifiState,
    }

    impl WifiManager for SimpleWifiManager {
        fn handle_event(&mut self, event: WifiEvent) -> WifiAction {
            match (&self.state, event) {
                (SimpleWifiState::RetrievingCredentials, WifiEvent::CredentialsMissing) => {
                    self.state = SimpleWifiState::WaitingForCredentials;
                    WifiAction::WaitForCredentials
                }
                (
                    SimpleWifiState::RetrievingCredentials,
                    WifiEvent::CredentialsFound { credentials },
                ) => {
                    self.state = SimpleWifiState::EstablishingConnection {
                        retries: 0,
                        credentials: credentials.clone(),
                    };
                    WifiAction::EstablishConnection { credentials }
                }
                (
                    SimpleWifiState::WaitingForCredentials,
                    WifiEvent::CredentialsReceived { credentials },
                ) => {
                    self.state = SimpleWifiState::EstablishingConnection {
                        retries: 0,
                        credentials: credentials.clone(),
                    };
                    WifiAction::EstablishConnection { credentials }
                }
                (SimpleWifiState::WaitingForCredentials, WifiEvent::Timeout) => {
                    self.state = SimpleWifiState::RetrievingCredentials;
                    WifiAction::RetrieveCredentials
                }
                (
                    SimpleWifiState::EstablishingConnection { .. },
                    WifiEvent::ConnectionEstablished,
                ) => {
                    self.state = SimpleWifiState::Connected;
                    WifiAction::WaitForDisconnect
                }
                (
                    SimpleWifiState::EstablishingConnection {
                        retries,
                        credentials,
                    },
                    WifiEvent::ConnectionNotEstablished,
                ) if *retries >= MAX_CONNECT_RETRIES => {
                    self.state = SimpleWifiState::WaitingForCredentials;
                    WifiAction::WaitForCredentials
                }
                (
                    SimpleWifiState::EstablishingConnection {
                        retries,
                        credentials,
                    },
                    WifiEvent::ConnectionNotEstablished,
                ) if *retries < MAX_CONNECT_RETRIES => {
                    let c = credentials.clone();
                    self.state = SimpleWifiState::EstablishingConnection {
                        retries: *retries + 1,
                        credentials: c.clone(),
                    };
                    WifiAction::EstablishConnection { credentials: c }
                }
                (SimpleWifiState::Connected, WifiEvent::Disconnected) => {
                    self.state = SimpleWifiState::RetrievingCredentials;
                    WifiAction::RetrieveCredentials
                }
                _ => WifiAction::Ignore,
            }
        }
    }

    impl SimpleWifiManager {
        pub fn new() -> Self {
            Self {
                state: SimpleWifiState::RetrievingCredentials,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use heapless::String;

        use super::*;

        #[test]
        fn test_credentials_missing_should_wait_for_credentials() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::RetrievingCredentials,
            };
            let action = c.handle_event(WifiEvent::CredentialsMissing);
            assert_eq!(action, WifiAction::WaitForCredentials);
        }

        #[test]
        fn test_credentials_found_should_start_client() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::RetrievingCredentials,
            };
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let action = c.handle_event(WifiEvent::CredentialsFound {
                credentials: credentials.clone(),
            });
            assert_eq!(action, WifiAction::EstablishConnection { credentials });
        }

        #[test]
        fn test_credentials_received_should_start_client() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::WaitingForCredentials,
            };
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let action = c.handle_event(WifiEvent::CredentialsReceived {
                credentials: credentials.clone(),
            });
            assert_eq!(action, WifiAction::EstablishConnection { credentials });
        }

        #[test]
        fn test_timeout_on_wait_for_credentials_should_check_credentials() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::WaitingForCredentials,
            };
            let action = c.handle_event(WifiEvent::Timeout);
            assert_eq!(action, WifiAction::RetrieveCredentials);
        }

        #[test]
        fn test_connect_should_connect() {
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::EstablishingConnection {
                    retries: 0,
                    credentials: credentials.clone(),
                },
            };
            let action = c.handle_event(WifiEvent::ConnectionEstablished);
            assert_eq!(action, WifiAction::WaitForDisconnect);
        }

        #[test]
        fn test_no_connect_should_retry() {
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::EstablishingConnection {
                    retries: 0,
                    credentials: credentials.clone(),
                },
            };
            let action = c.handle_event(WifiEvent::ConnectionNotEstablished);
            assert_eq!(
                action,
                WifiAction::EstablishConnection {
                    credentials: credentials
                }
            );
        }

        #[test]
        fn test_no_connection_after_retries_should_wait_for_credentials() {
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::EstablishingConnection {
                    retries: 4,
                    credentials: credentials.clone(),
                },
            };
            let action = c.handle_event(WifiEvent::ConnectionNotEstablished);
            assert_eq!(action, WifiAction::WaitForCredentials);
        }

        #[test]
        fn test_disconnect_should_retrieve_credentials() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::Connected,
            };
            let action = c.handle_event(WifiEvent::Disconnected);
            assert_eq!(action, WifiAction::RetrieveCredentials);
        }

        #[test]
        fn test_invalid_event_on_retrieve_credentials_should_ignore() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::RetrievingCredentials,
            };
            let action = c.handle_event(WifiEvent::Disconnected);
            assert_eq!(action, WifiAction::Ignore);
        }

        #[test]
        fn test_invalid_event_on_ap_sta_started_should_ignore() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::WaitingForCredentials,
            };
            let action = c.handle_event(WifiEvent::Disconnected);
            assert_eq!(action, WifiAction::Ignore);
        }

        #[test]
        fn test_invalid_event_on_client_started_should_ignore() {
            let credentials = LoginData {
                ssid: String::new(),
                password: String::new(),
            };
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::EstablishingConnection {
                    retries: 0,
                    credentials,
                },
            };
            let action = c.handle_event(WifiEvent::Disconnected);
            assert_eq!(action, WifiAction::Ignore);
        }

        #[test]
        fn test_invalid_event_on_connected_should_ignore() {
            let mut c = SimpleWifiManager {
                state: SimpleWifiState::Connected,
            };
            let action = c.handle_event(WifiEvent::ConnectionEstablished);
            assert_eq!(action, WifiAction::Ignore);
        }
    }
}

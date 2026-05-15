#[derive(serde::Deserialize, Debug, Clone)]

pub struct LoginData {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<32>,
}

pub enum WifiAction {
    Ignore,
    RetrieveCredentials,
    StartWifiApSta,
    StartWifiClient { credentials: LoginData },
    WaitForDisconnect,
}

pub enum WifiEvent {
    CredentialsMissing,
    CredentialsFound { credentials: LoginData },
    CredentialsReceived { credentials: LoginData },
    Connected,
    NotConnected,
    Disconnected,
    Timeout,
}

pub trait WifiControl {
    fn handle_event(&mut self, event: WifiEvent) -> WifiAction;
}

pub mod simple {
    use super::*;

    const MAX_CONNECT_RETRIES: usize = 3;

    #[derive(Debug)]
    pub enum SimpleWifiState {
        RetrievingCredentials,
        WifiApStaStarted,
        WifiClientStarted {
            retries: usize,
            credentials: LoginData,
        },
        Connected,
    }

    pub struct SimpleWifiControl {
        state: SimpleWifiState,
    }

    impl WifiControl for SimpleWifiControl {
        fn handle_event(&mut self, event: WifiEvent) -> WifiAction {
            match (&self.state, event) {
                (SimpleWifiState::RetrievingCredentials, WifiEvent::CredentialsMissing) => {
                    self.state = SimpleWifiState::WifiApStaStarted;
                    WifiAction::StartWifiApSta
                }
                (
                    SimpleWifiState::RetrievingCredentials,
                    WifiEvent::CredentialsFound { credentials },
                ) => {
                    self.state = SimpleWifiState::WifiClientStarted {
                        retries: 0,
                        credentials: credentials.clone(),
                    };
                    WifiAction::StartWifiClient { credentials }
                }
                (
                    SimpleWifiState::WifiApStaStarted,
                    WifiEvent::CredentialsReceived { credentials },
                ) => {
                    self.state = SimpleWifiState::WifiClientStarted {
                        retries: 0,
                        credentials: credentials.clone(),
                    };
                    WifiAction::StartWifiClient { credentials }
                }
                (SimpleWifiState::WifiApStaStarted, WifiEvent::Timeout) => {
                    self.state = SimpleWifiState::RetrievingCredentials;
                    WifiAction::RetrieveCredentials
                }
                (SimpleWifiState::WifiClientStarted { .. }, WifiEvent::Connected) => {
                    self.state = SimpleWifiState::Connected;
                    WifiAction::WaitForDisconnect
                }
                (
                    SimpleWifiState::WifiClientStarted {
                        retries,
                        credentials,
                    },
                    WifiEvent::NotConnected,
                ) => {
                    let c = credentials.clone();
                    if *retries >= MAX_CONNECT_RETRIES {
                        self.state = SimpleWifiState::WifiApStaStarted;
                        WifiAction::StartWifiApSta
                    } else {
                        self.state = SimpleWifiState::WifiClientStarted {
                            retries: *retries + 1,
                            credentials: c.clone(),
                        };
                        WifiAction::StartWifiClient { credentials: c }
                    }
                }
                (SimpleWifiState::Connected, WifiEvent::Disconnected) => {
                    self.state = SimpleWifiState::RetrievingCredentials;
                    WifiAction::RetrieveCredentials
                }
                _ => WifiAction::Ignore,
            }
        }
    }

    impl SimpleWifiControl {
        pub fn new() -> Self {
            Self {
                state: SimpleWifiState::RetrievingCredentials,
            }
        }
    }
}

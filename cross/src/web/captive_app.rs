use crate::web::{
    pages::{INDEX_HTML, STYLE_CSS},
    services::{
        captive_login::CaptiveLogin,
        captive_redirect::CaptiveRedirect,
        captive_ssids::{CaptiveSsids, SsidsList},
    },
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use logic::wifi::LoginData;
use picoserve::{
    self, AppBuilder,
    response::{File, StatusCode},
    routing::{get, get_service, post_service},
};

pub struct CaptiveApp<'a> {
    pub ssids: &'a Mutex<CriticalSectionRawMutex, SsidsList>,
    pub credentials: &'a Signal<CriticalSectionRawMutex, LoginData>,
}

impl AppBuilder for CaptiveApp<'_> {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::from_service(CaptiveRedirect)
            .route("/style.css", get_service(File::css(STYLE_CSS)))
            .route(
                "/",
                get(|| async {
                    (
                        StatusCode::OK,
                        [("content-type", "text/html; charset=utf-8")],
                        INDEX_HTML,
                    )
                }),
            )
            .route(
                "/login",
                post_service(CaptiveLogin {
                    credentials: self.credentials,
                }),
            )
            .route("/ssids", get_service(CaptiveSsids { ssids: self.ssids }))
    }
}

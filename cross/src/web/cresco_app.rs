use crate::web::{
    pages::{DASHBOARD_HTML, INDEX_HTML, PLANT_WEBP, STYLE_CSS},
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

pub struct CrescoApp {}

impl AppBuilder for CrescoApp {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
            .route("/style.css", get_service(File::css(STYLE_CSS)))
            .route(
                "/plant.webp",
                get_service(File::with_content_type("image/webp", PLANT_WEBP)),
            )
            .route(
                "/",
                get(|| async {
                    (
                        StatusCode::OK,
                        [("content-type", "text/html; charset=utf-8")],
                        DASHBOARD_HTML,
                    )
                }),
            )
    }
}

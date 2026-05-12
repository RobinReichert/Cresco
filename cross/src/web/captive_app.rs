use crate::web::{
    pages::INDEX_HTML,
    services::{captive_login::CaptiveLogin, captive_redirect::CaptiveRedirect},
};
use picoserve::{
    self, AppBuilder,
    response::StatusCode,
    routing::{get, post_service},
};

pub struct CaptiveApp;

impl AppBuilder for CaptiveApp {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::from_service(CaptiveRedirect)
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
            .route("/login", post_service(CaptiveLogin))
    }
}

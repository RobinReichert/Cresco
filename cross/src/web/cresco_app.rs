use crate::web::pages::{CALIBRATE_HTML, DASHBOARD_HTML, PLANT_WEBP, STYLE_CSS};
use picoserve::{
    self, AppBuilder,
    response::{File, StatusCode},
    routing::{get, get_service},
};

async fn calibrate_page() -> impl picoserve::response::IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        CALIBRATE_HTML,
    )
}

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
            .route("/calibrate/ph", get(calibrate_page))
            .route("/calibrate/ec", get(calibrate_page))
    }
}

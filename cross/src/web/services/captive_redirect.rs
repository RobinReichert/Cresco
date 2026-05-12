use logic::config::SERVER_URL;
use picoserve::{
    self,
    request::Request,
    response::{IntoResponse, Redirect},
    routing::PathRouterService,
};

pub struct CaptiveRedirect;

impl<State> PathRouterService<State> for CaptiveRedirect {
    async fn call_request_handler_service<
        R: picoserve::io::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        _state: &State,
        _current_path_parameters: (),
        _path: picoserve::request::Path<'_>,
        request: Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        Redirect::to(SERVER_URL)
            .write_to(request.body_connection.finalize().await?, response_writer)
            .await
    }
}

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use logic::wifi::LoginData;
use picoserve::{
    extract::{Form, FromRequest},
    response::{Response, StatusCode},
    routing::RequestHandlerService,
};

pub struct CaptiveLogin<'a> {
    pub credentials: &'a Signal<CriticalSectionRawMutex, LoginData>,
}

impl<State> RequestHandlerService<State> for CaptiveLogin<'_> {
    async fn call_request_handler_service<
        R: picoserve::io::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        state: &State,
        _path_parameters: (),
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        let (status_code, response_data) = match Form::<LoginData>::from_request(
            state,
            request.parts,
            request.body_connection.body(),
        )
        .await
        {
            Ok(Form(data)) => {
                self.credentials.signal(data);
                (StatusCode::OK, "")
            }
            Err(_) => (StatusCode::BAD_REQUEST, "Failed to decode body"),
        };
        let response = Response::new(status_code, response_data);
        response_writer
            .write_response(request.body_connection.finalize().await?, response)
            .await
    }
}

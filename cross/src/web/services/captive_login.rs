use defmt::info;
use picoserve::{
    extract::{Form, FromRequest},
    response::{Response, StatusCode},
    routing::RequestHandlerService,
};

#[derive(serde::Deserialize)]
struct LoginData {
    ssid: heapless::String<32>,
    password: heapless::String<32>,
}

pub struct CaptiveLogin;

impl<State> RequestHandlerService<State> for CaptiveLogin {
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
                info!("ssid: {}", data.ssid);
                (StatusCode::OK, "Ok")
            }
            Err(_) => (StatusCode::BAD_REQUEST, "Failed to decode body"),
        };
        let response = Response::new(status_code, response_data);
        response_writer
            .write_response(request.body_connection.finalize().await?, response)
            .await
    }
}

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::{String, Vec};
use picoserve::{response::Json, routing::RequestHandlerService};

const MAX_SSID_LENGTH: usize = 32;
pub const MAX_SSID_COUNT: usize = 5;

pub type SsidsList = Vec<String<MAX_SSID_LENGTH>, MAX_SSID_COUNT>;

pub struct CaptiveSsids<'a> {
    pub ssids: &'a Mutex<CriticalSectionRawMutex, SsidsList>,
}

#[derive(serde::Serialize)]
struct SsidData {
    ssids: SsidsList,
}

impl<State> RequestHandlerService<State> for CaptiveSsids<'_> {
    async fn call_request_handler_service<
        R: picoserve::io::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        _state: &State,
        _path_parameters: (),
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        let ssids = self.ssids.lock().await.clone();
        let ssids_json: Json<SsidData> = Json(SsidData { ssids });
        response_writer
            .write_response(
                request.body_connection.finalize().await?,
                ssids_json.into_response(),
            )
            .await
    }
}

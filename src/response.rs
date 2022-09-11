use worker::Response as WorkerResponse;

pub struct Response(WorkerResponse);

impl Response {
    pub(crate) fn new(resp: WorkerResponse) -> Self {
        Self(resp)
    }
}

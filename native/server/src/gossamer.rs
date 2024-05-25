use server::proto::{gossamer_server::Gossamer, ActionRequest, ActionResponse};
use tonic::{Request, Response, Status};

pub struct InMemoryGossamer;

#[tonic::async_trait]
impl Gossamer for InMemoryGossamer {
    async fn perform(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        todo!()
    }
}

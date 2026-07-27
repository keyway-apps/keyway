pub mod client;
pub mod server;

#[tarpc::service]
pub trait KeywayService {
    async fn execute();
    async fn commands();
}

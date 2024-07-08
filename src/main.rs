mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    web::WebServer::new().run().await
}

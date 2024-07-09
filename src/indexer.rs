use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct Indexer {}

impl Indexer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let indexer =
            pindexer::Indexer::new().with_index(crate::component::block::Component::new());
        Mutex::new(indexer).into_inner().run().await?;

        Ok(())
    }
}

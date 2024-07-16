#[derive(Clone, Debug)]
pub struct Indexer {
    options: pindexer::Options,
}

impl Indexer {
    pub fn new(options: pindexer::Options) -> Self {
        Self { options }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let mut indexer = pindexer::Indexer::new(self.options);
        indexer = crate::component::block::Component::new().attach_to_indexer(indexer);
        indexer = crate::component::validator::Component::new().attach_to_indexer(indexer);
        indexer.run().await?;

        Ok(())
    }
}

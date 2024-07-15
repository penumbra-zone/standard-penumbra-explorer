#[derive(Clone, Debug)]
pub struct Indexer {
    options: pindexer::Options,
}

impl Indexer {
    pub fn new(options: pindexer::Options) -> Self {
        Self { options }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let indexer = pindexer::Indexer::new(self.options)
            .with_index(pindexer::stake::ValidatorSet {})
            .with_index(crate::component::block::Component::new());
        indexer.run().await?;

        Ok(())
    }
}

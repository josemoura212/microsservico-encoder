use std::error::Error;
use uuid::Uuid;

pub trait Repository<T>: Send + Sync {
    type Error: Error + Send;

    async fn insert(&self, item: &T) -> Result<T, Self::Error>;
    async fn find(&self, id: &Uuid) -> Result<T, Self::Error>;

    async fn update(&self, _item: &T) -> Result<T, Self::Error> {
        unimplemented!("update not implemented for this repository")
    }
}

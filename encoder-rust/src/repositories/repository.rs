use std::error::Error;
use std::future::Future;

use uuid::Uuid;

pub trait Repository<T>: Send + Sync {
    type Error: Error + Send;

    fn insert(&self, item: &T) -> impl Future<Output = Result<T, Self::Error>> + Send;
    fn find(&self, id: &Uuid) -> impl Future<Output = Result<T, Self::Error>> + Send;

    fn update(&self, _item: &T) -> impl Future<Output = Result<T, Self::Error>> + Send {
        async { unimplemented!("update not implemented for this repository") }
    }
}

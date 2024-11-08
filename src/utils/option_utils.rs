use std::future::Future;

pub trait GetOrInsertWithAsync<T: 'static, GeneratorFuture: Future<Output = T>> {
    async fn get_or_insert_with_async(&mut self, f: GeneratorFuture) -> &mut T;
}

impl<T: 'static, GeneratorFuture: Future<Output = T>> GetOrInsertWithAsync<T, GeneratorFuture>
    for Option<T>
{
    async fn get_or_insert_with_async(&mut self, f: GeneratorFuture) -> &mut T {
        match self {
            Some(value) => value,
            None => self.get_or_insert(f.await),
        }
    }
}

pub trait TryGetOrInsertWithAsync<T: 'static, E, GeneratorFuture: Future<Output = Result<T, E>>> {
    async fn try_get_or_insert_with_async(&mut self, f: GeneratorFuture) -> Result<&mut T, E>;
}

impl<T: 'static, E, GeneratorFuture: Future<Output = Result<T, E>>>
    TryGetOrInsertWithAsync<T, E, GeneratorFuture> for Option<T>
{
    async fn try_get_or_insert_with_async(&mut self, f: GeneratorFuture) -> Result<&mut T, E> {
        match self {
            Some(value) => Ok(value),
            None => Ok(self.get_or_insert(f.await?)),
        }
    }
}

pub trait TryGetOrInsertWithOptionAsync<
    T: 'static,
    E,
    GeneratorFuture: Future<Output = Result<Option<T>, E>>,
>
{
    async fn try_get_or_insert_with_async(
        &mut self,
        f: GeneratorFuture,
    ) -> Result<Option<&mut T>, E>;
}

impl<T: 'static, E, GeneratorFuture: Future<Output = Result<Option<T>, E>>>
    TryGetOrInsertWithOptionAsync<T, E, GeneratorFuture> for Option<T>
{
    async fn try_get_or_insert_with_async(
        &mut self,
        f: GeneratorFuture,
    ) -> Result<Option<&mut T>, E> {
        match self {
            Some(value) => Ok(Some(value)),
            None => {
                let optional_value = f.await?;
                match optional_value {
                    Some(value) => Ok(Some(self.get_or_insert(value))),
                    None => Ok(None),
                }
            }
        }
    }
}

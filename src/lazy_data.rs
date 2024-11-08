use std::sync::Arc;

use parking_lot::RwLock;

#[derive(Clone)]
enum LazyDataInner<T, InitFnType = fn() -> T> {
    InitFn(InitFnType),
    Data(T),
}

#[derive(Clone)]
pub struct LazyData<T, InitFnType = fn() -> T> {
    inner: Arc<RwLock<LazyDataInner<T, InitFnType>>>,
}

impl<T, InitFnType: Fn() -> T> LazyData<T, InitFnType> {
    pub fn with_data(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LazyDataInner::Data(data))),
        }
    }

    pub fn with_init_fn(init_fn: InitFnType) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LazyDataInner::InitFn(init_fn))),
        }
    }

    pub fn get_ref(&self) -> &T {
        unsafe { self.init() }
    }

    unsafe fn init(&self) -> &T {
        let inner = self.inner.read();

        match &*inner {
            LazyDataInner::InitFn(_) => (),
            LazyDataInner::Data(data) => {
                return std::mem::transmute::<&T, &T>(data);
            }
        }

        drop(inner);

        let mut inner = self.inner.write();
        if let LazyDataInner::InitFn(init_fn) = &*inner {
            let data = init_fn();
            *inner = LazyDataInner::Data(data);
        }

        if let LazyDataInner::Data(data) = &*inner {
            return std::mem::transmute::<&T, &T>(data);
        }

        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use super::LazyData;

    #[test]
    fn with_data() {
        let expected_value = "foobar".to_string();
        let init_value = expected_value.clone();

        let lazy = LazyData::<String>::with_data(init_value);

        assert_eq!(expected_value, *lazy.get_ref());
    }

    #[test]
    fn with_init_fn() {
        let expected_value = "foobar".to_string();
        let init_value = expected_value.clone();

        let lazy = LazyData::with_init_fn(move || init_value.clone());

        assert_eq!(expected_value, *lazy.get_ref());
    }
}

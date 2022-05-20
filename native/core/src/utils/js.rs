use napi::bindgen_prelude::ToNapiValue;
use napi::{Result, Env, JsUnknown, JsObject, NapiValue};
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction};

pub type JsDummyResult = Result<Vec<JsUnknown>>;
pub fn create_dummy_tsfn<T, F, R> (env: &Env, name: &str, callback: F) -> Result<ThreadsafeFunction<T>>
where
    T: 'static,
    F: 'static + Send + FnMut(ThreadSafeCallContext<T>) -> Result<Vec<R>>,
    R: ToNapiValue
{
    let closure = env.create_function_from_closure(name, |ctx| ctx.get::<JsUnknown>(0))?;
    return closure.create_threadsafe_function(0, callback);
}

#[macro_export]
macro_rules! future_to_promise {
    ($env:expr, move $future:expr) => {
        $env.execute_tokio_future(
            async move { $future.await.map_err(|err| napi::Error::new(Status::GenericFailure, err.to_string())) },
            |env, _| env.get_undefined()
        )
    };
}

pub fn js_for_each<T, C> (array: JsObject, mut consume: C) -> Result<()>
where T: NapiValue, C: FnMut(T) -> Result<()>
{
    let length = array.get_array_length_unchecked()?;
    for i in 0..length {
        consume(array.get_element_unchecked(i)?)?;
    }

    return Ok(());
}

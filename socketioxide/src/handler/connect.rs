//! [`ConnectHandler`] trait and implementations, used to handle the connect event
//! It has a flexible axum-like API, you can put any arguments as long as it implements the [`FromConnectParts`] trait
//! Handlers can be async or not
//!
//! ## Example
//! ```rust
//! # use socketioxide::SocketIo;
//! # use serde_json::Error;
//! # use socketioxide::extract::*;
//! let (svc, io) = SocketIo::new_svc();
//! io.ns("/", move |s: SocketRef, TryData(auth): TryData<String>| {
//!     println!("Socket connected on / namespace with id and auth data: {} {:?}", s.id, auth);
//! });
//! // Here the auth data is not serialized, if there is any it is dropped
//! io.ns("/async_nsp", move |s: SocketRef| async move {
//!     println!("Socket connected on /async_nsp namespace with id: {}", s.id);
//! });
//! // Here the auth data is not serialized, if there is a serialization error, the handler is not called
//! io.ns("/nsp", move |s: SocketRef, Data(auth): Data<String>| {
//!     println!("Socket connected on /nsp namespace with id: {} and data: {}", s.id, auth);
//! });
//! ```
//!
use std::sync::Arc;

use futures::Future;

use crate::{adapter::Adapter, socket::Socket};

use super::MakeErasedHandler;

/// A Type Erased [`ConnectHandler`] so it can be stored in a HashMap
pub(crate) type BoxedConnectHandler<A> = Box<dyn ErasedConnectHandler<A>>;
pub(crate) trait ErasedConnectHandler<A: Adapter>: Send + Sync + 'static {
    fn call(&self, s: Arc<Socket<A>>, auth: Option<String>);
}

impl<A: Adapter, T, H, Fut> MakeErasedHandler<H, A, T, Fut>
where
    T: Send + Sync + 'static,
    H: ConnectHandler<A, T, Fut> + Send + Sync + 'static,
    Fut: Send + Sync + 'static,
{
    pub fn new_ns_boxed(inner: H) -> Box<dyn ErasedConnectHandler<A>> {
        Box::new(MakeErasedHandler::new(inner))
    }
}

impl<A: Adapter, T, H, Fut> ErasedConnectHandler<A> for MakeErasedHandler<H, A, T, Fut>
where
    H: ConnectHandler<A, T, Fut> + Send + Sync + 'static,
    T: Send + Sync + 'static,
    Fut: Send + Sync + 'static,
{
    #[inline(always)]
    fn call(&self, s: Arc<Socket<A>>, auth: Option<String>) {
        self.handler.call(s, auth);
    }
}

/// A trait used to extract the arguments from the connect event
/// The `Result` is used to return an error if the extraction fails, in this case the handler is not called
pub trait FromConnectParts<A: Adapter>: Sized {
    fn from_connect_parts(s: &Arc<Socket<A>>, auth: &Option<String>) -> Result<Self, ()>;
}

/// Define a handler for the connect event
/// It is implemented for closures with up to 16 arguments that implement the [`FromConnectParts`] trait
/// The closure can be async or not
pub trait ConnectHandler<A: Adapter, T, F>: Send + Sync + 'static {
    fn call(&self, s: Arc<Socket<A>>, auth: Option<String>);

    fn phantom(&self) -> std::marker::PhantomData<T> {
        std::marker::PhantomData
    }

    fn phantom_fut(&self) -> std::marker::PhantomData<F> {
        std::marker::PhantomData
    }
}

macro_rules! impl_handler_async {
    (
        [$($ty:ident),*]
    ) => {
        #[allow(non_snake_case, unused)]
        impl<A, F, Fut, $($ty,)*> ConnectHandler<A, ($($ty,)*), (Fut,)> for F
        where
            F: FnOnce($($ty,)*) -> Fut + Send + Sync + Clone + 'static,
            Fut: Future<Output = ()> + Send + 'static,
            A: Adapter,
            $( $ty: FromConnectParts<A> + Send, )*
        {
            fn call(&self, s: Arc<Socket<A>>, auth: Option<String>) {
                $(
                    let $ty = match $ty::from_connect_parts(&s, &auth) {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                )*

                let fut = (self.clone())($($ty,)*);
                tokio::spawn(fut);

            }
        }
    };
}

macro_rules! impl_handler {
    (
        [$($ty:ident),*]
    ) => {
        #[allow(non_snake_case, unused)]
        impl<A, F, $($ty,)*> ConnectHandler<A, ($($ty,)*), ()> for F
        where
            F: FnOnce($($ty,)*) -> () + Send + Sync + Clone + 'static,
            A: Adapter,
            $( $ty: FromConnectParts<A> + Send, )*
        {
            fn call(&self, s: Arc<Socket<A>>, auth: Option<String>) {
                $(
                    let $ty = match $ty::from_connect_parts(&s, &auth) {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                )*

                (self.clone())($($ty,)*);
            }
        }
    };
}
#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([]);
        $name!([T1]);
        $name!([T1, T2]);
        $name!([T1, T2, T3]);
        $name!([T1, T2, T3, T4]);
        $name!([T1, T2, T3, T4, T5]);
        $name!([T1, T2, T3, T4, T5, T6]);
        $name!([T1, T2, T3, T4, T5, T6, T7]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15]);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16]);
    };
}

all_the_tuples!(impl_handler_async);
all_the_tuples!(impl_handler);

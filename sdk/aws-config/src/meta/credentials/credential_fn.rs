use aws_types::credentials;
use aws_types::credentials::ProvideCredentials;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;

/// A [`ProvideCredentials`] implemented by a closure.
///
/// See [`provide_credentials_fn`] for more details.
#[derive(Copy, Clone)]
pub struct ProvideCredentialsFn<'c, T> {
    f: T,
    phantom: PhantomData<&'c T>,
}

impl<T> Debug for ProvideCredentialsFn<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ProvideCredentialsFn")
    }
}

impl<'c, T, F> ProvideCredentials for ProvideCredentialsFn<'c, T>
where
    T: Fn() -> F + Send + Sync + 'c,
    F: Future<Output = credentials::Result> + Send + 'static,
{
    fn provide_credentials<'a>(&'a self) -> credentials::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        credentials::future::ProvideCredentials::new((self.f)())
    }
}

/// Returns a new credentials provider built with the given closure. This allows you
/// to create an [`ProvideCredentials`] implementation from an async block that returns
/// a [`credentials::Result`].
///
/// # Examples
///
/// ```
/// use aws_types::Credentials;
/// use aws_config::meta::credentials::provide_credentials_fn;
///
/// async fn load_credentials() -> Credentials {
///     todo!()
/// }
///
/// provide_credentials_fn(|| async {
///     // Async process to retrieve credentials goes here
///     let credentials = load_credentials().await;
///     Ok(credentials)
/// });
/// ```
pub fn provide_credentials_fn<'c, T, F>(f: T) -> ProvideCredentialsFn<'c, T>
where
    T: Fn() -> F + Send + Sync + 'c,
    F: Future<Output = credentials::Result> + Send + 'static,
{
    ProvideCredentialsFn {
        f,
        phantom: Default::default(),
    }
}

#[cfg(test)]
mod test {
    use crate::meta::credentials::credential_fn::provide_credentials_fn;
    use async_trait::async_trait;
    use aws_types::credentials::ProvideCredentials;
    use aws_types::{credentials, Credentials};
    use std::fmt::{Debug, Formatter};

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn creds_are_send_sync() {
        assert_send_sync::<Credentials>()
    }

    #[async_trait]
    trait AnotherTrait: Send + Sync {
        async fn creds(&self) -> Credentials;
    }

    struct AnotherTraitWrapper<T> {
        inner: T,
    }

    impl<T> Debug for AnotherTraitWrapper<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "wrapper")
        }
    }

    impl<T: AnotherTrait> ProvideCredentials for AnotherTraitWrapper<T> {
        fn provide_credentials<'a>(&'a self) -> credentials::future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            credentials::future::ProvideCredentials::new(
                async move { Ok(self.inner.creds().await) },
            )
        }
    }

    // Test that the closure passed to `provide_credentials_fn` is allowed to borrow things
    #[tokio::test]
    async fn provide_credentials_fn_closure_can_borrow() {
        fn check_is_str_ref(_input: &str) {}
        async fn test_async_provider(input: String) -> credentials::Result {
            Ok(Credentials::from_keys(&input, &input, None))
        }

        let things_to_borrow = vec!["one".to_string(), "two".to_string()];

        let mut providers = Vec::new();
        for thing in &things_to_borrow {
            let provider = provide_credentials_fn(move || {
                check_is_str_ref(thing);
                test_async_provider(thing.into())
            });
            providers.push(provider);
        }

        let (two, one) = (providers.pop().unwrap(), providers.pop().unwrap());
        assert_eq!(
            "one",
            one.provide_credentials().await.unwrap().access_key_id()
        );
        assert_eq!(
            "two",
            two.provide_credentials().await.unwrap().access_key_id()
        );
    }
}

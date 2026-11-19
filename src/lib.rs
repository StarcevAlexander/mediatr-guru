use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Request: 'static {
    type Response: 'static;
}

#[derive(Debug)]
pub enum MediatorError {
    NoHandler { request_type: &'static str },
}

impl std::fmt::Display for MediatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediatorError::NoHandler { request_type } => {
                write!(f, "no handler registered for {request_type}")
            }
        }
    }
}

impl std::error::Error for MediatorError {}

type Handler = Box<dyn Fn(Box<dyn Any>) -> Box<dyn Any> + Send + Sync>;

#[derive(Default)]
pub struct Mediator {
    handlers: HashMap<TypeId, Handler>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<R, F>(&mut self, handler: F)
    where
        R: Request,
        F: Fn(R) -> R::Response + Send + Sync + 'static,
    {
        self.handlers.insert(
            TypeId::of::<R>(),
            Box::new(move |boxed: Box<dyn Any>| {
                let request = *boxed
                    .downcast::<R>()
                    .expect("request type checked at registration");
                Box::new(handler(request)) as Box<dyn Any>
            }),
        );
    }

    pub fn send<R>(&self, request: R) -> Result<R::Response, MediatorError>
    where
        R: Request + Any,
        R::Response: Any,
    {
        let handler =
            self.handlers
                .get(&TypeId::of::<R>())
                .ok_or_else(|| MediatorError::NoHandler {
                    request_type: std::any::type_name::<R>(),
                })?;
        let boxed = handler(Box::new(request));
        let response = boxed
            .downcast::<R::Response>()
            .expect("handler returned the declared response type");
        Ok(*response)
    }

    pub fn registered_count(&self) -> usize {
        self.handlers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Ping;
    impl Request for Ping {
        type Response = String;
    }

    struct Fire;
    impl Request for Fire {
        type Response = ();
    }

    struct Sum;
    impl Request for Sum {
        type Response = usize;
    }

    #[test]
    fn request_dispatched_to_handler() {
        let mut mediator = Mediator::new();
        mediator.register(|_: Ping| format!("pong from {}", "handler"));
        let response = mediator.send(Ping).unwrap();
        assert_eq!(response, "pong from handler");
    }

    #[test]
    fn no_handler_is_an_error() {
        let mediator = Mediator::new();
        let err = mediator.send(Ping).unwrap_err();
        assert!(err.to_string().contains("no handler"));
    }

    #[test]
    fn multiple_request_types_do_not_interfere() {
        let mut mediator = Mediator::new();
        mediator.register(|_: Ping| "pong".to_string());
        mediator.register(|_: Sum| 42usize);
        assert_eq!(mediator.send(Ping).unwrap(), "pong");
        assert_eq!(mediator.send(Sum).unwrap(), 42usize);
        assert_eq!(mediator.registered_count(), 2);
    }

    #[test]
    fn notifications_with_unit_response() {
        let mut mediator = Mediator::new();
        mediator.register(|_: Fire| ());
        assert_eq!(mediator.send(Fire).unwrap(), ());
    }
}

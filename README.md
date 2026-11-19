# mediatrguru

**MediatR-style in-process mediator for Rust** — register typed request/response handlers in a registry, dispatch by type, zero dependencies.

For developers coming from C#'s [MediatR](https://github.com/jbogard/MediatR) who miss `IMediator.Send(request)` when splitting application logic into handlers.

## Usage

```rust
use mediatrguru::{Mediator, Request};

struct Ping;
impl Request for Ping {
    type Response = String;
}

fn main() {
    let mut mediator = Mediator::new();
    mediator.register(|_: Ping| "pong".to_string());

    let response = mediator.send(Ping).unwrap();
    assert_eq!(response, "pong");
}
```

Handlers are plain closures — anything `Fn(R) -> R::Response + Send + Sync + 'static`. Unit-response requests double as notifications: register `|_: Fire| ()` and `send` fires them.

## License

Licensed under either of MIT or Apache-2.0, at your option.

Name is a tribute to C#'s [MediatR](https://github.com/jbogard/MediatR); this crate is an independent implementation for Rust with zero dependencies.

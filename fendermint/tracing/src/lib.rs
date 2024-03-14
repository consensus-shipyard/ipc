// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

/// Emit an event that conforms to a flat event structure type using the [tracing::event!](https://github.com/tokio-rs/tracing/blob/908cc432a5994f6e17c8f36e13c217dc40085704/tracing/src/macros.rs#L854) macro.
#[macro_export]
macro_rules! emit {
    ($event:ident { $($field:ident : $value:expr),* $(,)? } ) => {{
        // Make sure the emitted fields match the schema of the event.
        let _event = || $event {
            $($field : $value),*
        };
        tracing::event!(
            name: stringify!($event),
            tracing::Level::INFO,
            { $($field = $value),* }
        )
    }};
}

#[cfg(test)]
mod tests {

    #[allow(dead_code)]
    struct TestEvent<'a> {
        pub foo: u32,
        pub bar: &'a str,
    }

    #[test]
    fn test_emit() {
        emit!(TestEvent {
            foo: 123,
            bar: "spam",
        });
    }
}

// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use lazy_static::lazy_static;
use tracing::metadata::Metadata

pub type FieldNames = &'static [&'static str];

/// A tracer defined outside the module which emits the events,
/// so that we can define centrally how to deal with them, e.g.
/// which fields to log, with what level, and which metrics to
/// increment.
///
/// It's meant to be defined using a macro that projects the
/// event into the flag key-value structure preferred by `tracing`,
/// while it also maintains the opportunity to deal with the
/// original event structure.
pub trait Tracer<T> {
    /// Defines the logging level of the event type;
    fn level() -> tracing::Level;

    /// Produce an instance independent list of fields which will be traced
    /// for this particular event type.
    fn field_names() -> FieldNames;

    // /// Project an event into a vlue set used by `tracing`.
    // fn value_set<'a>(field_set: &'a FieldSet, event: &T) -> tracing::field::ValueSet<'a>;

    // /// Do custom tracing of the value.
    // ///
    // /// The macro responsible for putting together the fields and the callsite metadata will
    // /// take care emitting the event for `tracing` itself.
    // fn trace(&self, event: &T);
}

/// Define a tracer implementation.
///
/// We want to preserve the emitting callsite, but also take care of which values are emitted based on the event,
/// rather than enumerating them in a call to the raw `tracing` macros, which could lead to inconsistencies if the
/// same event is emitted from multiple places.
///
/// Based on the following macros:
/// * [callsite2](https://github.com/tokio-rs/tracing/blob/tracing-0.1.40/tracing/src/macros.rs#L2732)
/// * [metadata](https://github.com/tokio-rs/tracing/blob/tracing-0.1.40/tracing-core/src/lib.rs#L216)
#[macro_export]
macro_rules! tracer {
    ($tracer:ty, $event:ty, $lvl:expr, { $($fields:tt)* } ) => {
        impl $crate::Tracer<$event> for $tracer {
            fn level() -> tracing::Level {
                $lvl
            }

            fn field_names() -> $crate::FieldNames {
                static FIELD_NAMES: $crate::FieldNames = tracing::fieldset!( $($fields)* );
                FIELD_NAMES
            }
        }
    }
}

/// Emit an event.
///
/// Based on the following macros:
/// * [event](https://github.com/tokio-rs/tracing/blob/908cc432a5994f6e17c8f36e13c217dc40085704/tracing/src/macros.rs#L854) module in `tracing`.
#[macro_export]
macro_rules! emit {
    ($tracer:expr, $event_ty:ty : $event:expr) => {{
        lazy_static::lazy_static! {
            //static ref __LVL: tracing::Level = <$tracer_ty as $crate::Tracer<$event_ty>>::level();
            static ref __LVL: tracing::Level = $crate::Tracer::<$event_ty>::level($tracer);
        }

        //use tracing::__macro_support::Callsite as _;
        // static __CALLSITE: tracing::__macro_support::MacroCallsite = {
        //     static META: tracing::Metadata<'static> = {
        //         tracing::metadata::Metadata::new(
        //             std::any::type_name::<$event_ty>(), // name
        //             module_path!(),                     // target
        //             __LVL,
        //             ::core::option::Option::Some(file!()),
        //             ::core::option::Option::Some(line!()),
        //             ::core::option::Option::Some(module_path!()),
        //             tracing::field::FieldSet::new(
        //                 <$tracer_ty as $crate::Tracer<$event_ty>>::field_names(),
        //                 tracing::callsite::Identifier(&__CALLSITE),
        //             ),
        //             tracing::metadata::Kind::EVENT,
        //         )
        //     };
        //     tracing::callsite::DefaultCallsite::new(&META)
        // };

        // let enabled = tracing::level_enabled!(__LVL) && {
        //     let interest = __CALLSITE.interest();
        //     !interest.is_never() && __CALLSITE.is_enabled(interest)
        // };

        // if enabled {
        //     (|value_set: tracing::field::ValueSet| {
        //         let meta = __CALLSITE.metadata();
        //         // event with contextual parent
        //         tracing::Event::dispatch(
        //             meta,
        //             &value_set
        //         );
        //         tracing::__tracing_log!(
        //             __LVL,
        //             __CALLSITE,
        //             &value_set
        //         );
        //     })(tracing::valueset!(__CALLSITE.metadata().fields(), $($fields)*));
        // } else {
        //     tracing::__tracing_log!(
        //         __LVL,
        //         __CALLSITE,
        //         &tracing::valueset!(__CALLSITE.metadata().fields(), $($fields)*)
        //     );
        // }
    }};
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use crate::Tracer;

    struct TestEvent<'a> {
        pub foo: u32,
        pub bar: &'a str,
    }

    struct TestTracer;

    tracer!(TestTracer, TestEvent<'_>, Level::DEBUG, { a = foo, b = bar, c = 1} );

    fn generic_emit_test<T>(tracer: &T)
    where
        T: for<'a> Tracer<TestEvent<'a>>,
    {
        lazy_static::lazy_static! {
            static ref __LVL: tracing::Level = <T as Tracer<TestEvent<'_>>>::level();
        }
        // emit!(t, TestEvent<'static> : &TestEvent {
        //         foo: 123,
        //         bar: "spam"
        //     }
        // );
    }

    #[test]
    fn test_level() {
        let level1 = TestTracer::level();
        let level2 = Tracer::<TestEvent<'static>>::level(&TestTracer);
        assert_eq!(level1, Level::DEBUG);
        assert_eq!(level1, level2);
    }

    #[test]
    fn test_field_names() {
        let fns = TestTracer::field_names();
        assert_eq!(*fns, ["a", "b", "c"]);
    }

    // #[test]
    // fn test_emit() {
    //     emit!(
    //         TestTracer : TestTracer,
    //         TestEvent<'_> : &TestEvent {
    //             foo: 123,
    //             bar: "spam"
    //         }
    //     );
    // }
}

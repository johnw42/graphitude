//! Tracing support for performance monitoring.
//!
//! This module provides tracing functionality when the `tracing` feature is enabled,
//! and provides no-op implementations when it's disabled.

#[cfg(feature = "tracing")]
mod enabled {
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        collections::HashMap,
        sync::Once,
        time::{Duration, Instant},
    };

    use tracing_subscriber::{
        Layer, Registry, layer::Context, layer::SubscriberExt, registry::LookupSpan,
        util::SubscriberInitExt,
    };

    thread_local! {
        #[allow(clippy::type_complexity)]
        static TIMING_SCOPES: RefCell<HashMap<TimingScope, BTreeMap<&'static str, (Duration, usize)>>> =
            RefCell::new(HashMap::new());
        static TIMING_SCOPE: RefCell<TimingScope> = const { RefCell::new(TimingScope::Test) };
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum TimingScope {
        Test,
        Consistency,
    }

    pub struct TimingScopeGuard {
        previous: TimingScope,
    }

    impl Drop for TimingScopeGuard {
        fn drop(&mut self) {
            TIMING_SCOPE.with(|scope| {
                *scope.borrow_mut() = self.previous;
            });
        }
    }

    pub fn set_timing_scope(scope: TimingScope) -> TimingScopeGuard {
        let previous = TIMING_SCOPE.with(|current| {
            let mut current = current.borrow_mut();
            let prev = *current;
            *current = scope;
            prev
        });
        TimingScopeGuard { previous }
    }

    struct TimingLayer;

    impl<S> Layer<S> for TimingLayer
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            _attrs: &tracing::span::Attributes<'_>,
            id: &tracing::Id,
            ctx: Context<'_, S>,
        ) {
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(Instant::now());
            }
        }

        fn on_close(&self, id: tracing::Id, ctx: Context<'_, S>) {
            if let Some(span) = ctx.span(&id) {
                let name = span.metadata().name();
                if let Some(start) = span.extensions().get::<Instant>() {
                    let elapsed = start.elapsed();
                    let scope = TIMING_SCOPE.with(|scope| *scope.borrow());
                    TIMING_SCOPES.with(|totals| {
                        let mut totals = totals.borrow_mut();
                        let entries = totals.entry(scope).or_insert_with(BTreeMap::new);
                        let entry = entries.entry(name).or_insert((Duration::ZERO, 0));
                        entry.0 += elapsed;
                        entry.1 += 1;
                    });
                }
            }
        }
    }

    pub fn init_tracing() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = Registry::default().with(TimingLayer).try_init();
        });
    }

    #[doc(hidden)]
    pub fn dump_method_timings() {
        dump_scope_timings(TimingScope::Test);
        dump_scope_timings(TimingScope::Consistency);
    }

    #[doc(hidden)]
    pub fn reset_method_timings() {
        init_tracing();
        TIMING_SCOPES.with(|totals| totals.borrow_mut().clear());
    }

    fn dump_scope_timings(scope: TimingScope) {
        TIMING_SCOPES.with(|totals| {
            let totals = totals.borrow();
            let label = format!("{scope:?} timings (desc):");
            let Some(entries) = totals.get(&scope) else {
                eprintln!("{}", label);
                return;
            };
            let mut entries: Vec<_> = entries.iter().collect();
            entries.sort_by(|a, b| b.1.0.cmp(&a.1.0));
            eprintln!("{}", label);
            for (name, (duration, count)) in entries {
                eprintln!("  {name}: {:?} ({}x)", duration, count);
            }
        });
    }

    // Re-export tracing macros for convenience
    pub use tracing::info_span;
}

#[cfg(not(feature = "tracing"))]
mod disabled {
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum TimingScope {
        Test,
        Consistency,
    }

    pub struct TimingScopeGuard;

    pub fn set_timing_scope(_scope: TimingScope) -> TimingScopeGuard {
        TimingScopeGuard
    }

    pub fn init_tracing() {
        // No-op when tracing is disabled
    }

    #[doc(hidden)]
    pub fn dump_method_timings() {
        // No-op when tracing is disabled
    }

    #[doc(hidden)]
    pub fn reset_method_timings() {
        // No-op when tracing is disabled
    }

    // Provide a no-op macro replacement for info_span
    #[macro_export]
    macro_rules! info_span {
        ($name:expr) => {{ $crate::tracing_support::NoOpSpan }};
        ($name:expr, $($fields:tt)*) => {{ $crate::tracing_support::NoOpSpan }};
    }

    pub use info_span;

    pub struct NoOpSpan;

    impl NoOpSpan {
        pub fn entered(self) -> NoOpSpanGuard {
            NoOpSpanGuard
        }
    }

    pub struct NoOpSpanGuard;
}

// Re-export the appropriate implementation
#[cfg(feature = "tracing")]
pub use enabled::*;

#[cfg(not(feature = "tracing"))]
pub use disabled::*;

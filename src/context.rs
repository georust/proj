use std::rc::Rc;

use proj_sys::{PJ_CONTEXT, proj_context_clone, proj_context_create, proj_context_destroy};

/// A PROJ context (`PJ_CONTEXT`).
///
/// A thin wrapper around the raw pointer: dropping a `Context` calls `proj_context_destroy`.
/// This is the only type responsible for destroying a context, whether the context belongs to a
/// single [`Proj`](crate::Proj) or is shared between many via [`ProjContext::Shared`].
pub(crate) struct Context {
    ctx: *mut PJ_CONTEXT,
}

impl Context {
    /// Create a new PROJ context.
    pub(crate) fn new() -> Self {
        Self {
            ctx: unsafe { proj_context_create() },
        }
    }

    /// The raw context pointer, for passing to PROJ FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut PJ_CONTEXT {
        self.ctx
    }
}

impl Clone for Context {
    /// Clone the underlying context via `proj_context_clone`, yielding an independently owned
    /// `Context`.
    fn clone(&self) -> Self {
        Self {
            ctx: unsafe { proj_context_clone(self.ctx) },
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // The single place a PROJ context is destroyed. This can run during thread-local teardown
        // (for the shared per-thread context), so it must not re-enter the `SHARED_CONTEXT`
        // thread-local. We deliberately do not call proj_cleanup() (see the note in `Drop for
        // Proj`).
        unsafe { proj_context_destroy(self.ctx) };
    }
}

/// The context backing a [`Proj`](crate::Proj): either uniquely owned by that `Proj`, or the
/// per-thread context shared between every `Proj` created via [`Proj::new`](crate::Proj::new) and
/// [`Proj::new_known_crs`](crate::Proj::new_known_crs).
///
/// `Shared` reference counts the context so that it outlives every `Proj` that uses it, regardless
/// of the order in which the thread-local and any surviving `Proj` instances are dropped at thread
/// exit.
pub(crate) enum ProjContext {
    Owned(Context),
    // Wired into Proj::new/new_known_crs in a follow-up commit.
    #[allow(dead_code)]
    Shared(Rc<Context>),
}

impl ProjContext {
    fn context(&self) -> &Context {
        match self {
            ProjContext::Owned(ctx) => ctx,
            ProjContext::Shared(ctx) => ctx.as_ref(),
        }
    }

    /// The raw context pointer, for passing to PROJ FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut PJ_CONTEXT {
        self.context().as_ptr()
    }

    /// Clone the underlying context into a new, independently owned `ProjContext::Owned`.
    ///
    /// Used where a derived `Proj` needs its own context rather than continuing to share the
    /// per-thread one.
    pub(crate) fn clone_owned(&self) -> Self {
        ProjContext::Owned(self.context().clone())
    }
}

thread_local! {
    /// One PROJ context per thread, reused by `Proj::new`/`Proj::new_known_crs`. Creating a fresh
    /// context per object opens a new connection to the PROJ database with cold caches, which
    /// dominates `Proj` construction time (see https://github.com/georust/proj/issues/256).
    static SHARED_CONTEXT: Rc<Context> = Rc::new(Context::new());
}

/// Return a reference-counted handle to the calling thread's shared PROJ context.
#[allow(dead_code)] // wired into Proj::new/new_known_crs in a follow-up commit
pub(crate) fn thread_local_context() -> ProjContext {
    ProjContext::Shared(SHARED_CONTEXT.with(Rc::clone))
}

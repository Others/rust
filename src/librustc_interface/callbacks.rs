//! Throughout the compiler tree, there are several places which want to have
//! access to state or queries while being inside crates that are dependencies
//! of librustc. To facilitate this, we have the
//! `rustc_data_structures::AtomicRef` type, which allows us to setup a global
//! static which can then be set in this file at program startup.
//!
//! See `SPAN_DEBUG` for an example of how to set things up.
//!
//! The functions in this file should fall back to the default set in their
//! origin crate when the `TyCtxt` is not present in TLS.

use rustc::ty::tls;
use rustc_errors::{Diagnostic, TRACK_DIAGNOSTICS};
use std::fmt;

/// This is a callback from libsyntax as it cannot access the implicit state
/// in librustc otherwise.
fn span_debug(span: rustc_span::Span, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    tls::with_opt(|tcx| {
        if let Some(tcx) = tcx {
            write!(f, "{}", tcx.sess.source_map().span_to_string(span))
        } else {
            rustc_span::default_span_debug(span, f)
        }
    })
}

/// This is a callback from libsyntax as it cannot access the implicit state
/// in librustc otherwise. It is used to when diagnostic messages are
/// emitted and stores them in the current query, if there is one.
fn track_diagnostic(diagnostic: &Diagnostic) {
    tls::with_context_opt(|icx| {
        if let Some(icx) = icx {
            if let Some(ref diagnostics) = icx.diagnostics {
                let mut diagnostics = diagnostics.lock();
                diagnostics.extend(Some(diagnostic.clone()));
            }
        }
    })
}

/// This is a callback from librustc_hir as it cannot access the implicit state
/// in librustc otherwise.
fn def_id_debug(def_id: rustc_hir::def_id::DefId, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "DefId({}:{}", def_id.krate, def_id.index.index())?;
    tls::with_opt(|opt_tcx| {
        if let Some(tcx) = opt_tcx {
            write!(f, " ~ {}", tcx.def_path_debug_str(def_id))?;
        }
        Ok(())
    })?;
    write!(f, ")")
}

/// Sets up the callbacks in prior crates which we want to refer to the
/// TyCtxt in.
pub fn setup_callbacks() {
    rustc_span::SPAN_DEBUG.swap(&(span_debug as fn(_, &mut fmt::Formatter<'_>) -> _));
    rustc_hir::def_id::DEF_ID_DEBUG.swap(&(def_id_debug as fn(_, &mut fmt::Formatter<'_>) -> _));
    TRACK_DIAGNOSTICS.swap(&(track_diagnostic as fn(&_)));
}

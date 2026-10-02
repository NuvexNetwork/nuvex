//! Callback CPI is performed by `fulfill`. See ADR 0006.
//!
//! The callee receives no accounts. A failing callback aborts the transaction,
//! so the request is not left both fulfilled and unfulfilled.

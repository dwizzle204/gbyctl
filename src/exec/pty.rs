//! PTY compatibility layer.

/// Whether PTY support is available in this build.
#[must_use]
pub const fn supported() -> bool {
    false
}

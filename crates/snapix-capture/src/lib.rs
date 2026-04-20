mod backend;
pub use backend::*;

#[cfg(all(unix, feature = "x11"))]
pub mod x11;

#[cfg(all(unix, feature = "wayland"))]
pub mod wayland;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_backend_returns_valid_backend() {
        // This test verifies that detect_backend() doesn't panic
        // and returns a valid backend based on the current session.
        let backend = detect_backend();
        let name = backend.name();
        assert!(!name.is_empty());
        // Should be either "x11rb" or "ashpd-portal"
        assert!(name == "x11rb" || name == "ashpd-portal");
    }

    #[cfg(feature = "x11")]
    #[test]
    fn x11_backend_creation() {
        let backend = x11::X11Backend::new();
        assert_eq!(backend.name(), "x11rb");
        assert!(backend.supports_interactive());
    }

    #[cfg(feature = "wayland")]
    #[test]
    fn wayland_backend_creation() {
        let backend = wayland::WaylandBackend::new();
        assert_eq!(backend.name(), "ashpd-portal");
        // Portal handles selection dialog, so we don't support interactive ourselves
        assert!(!backend.supports_interactive());
    }
}

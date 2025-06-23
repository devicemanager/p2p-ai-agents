/// Resource manager for network operations.
#[derive(Debug, Default)]
pub struct ResourceManager;

impl ResourceManager {
    /// Creates a new resource manager.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

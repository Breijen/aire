#[derive(Debug)]
pub enum AireError {
    CommandBufferFull,
}

impl std::fmt::Display for AireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AireError::CommandBufferFull => write!(f, "command buffer full"),
        }
    }
}

impl std::error::Error for AireError {}

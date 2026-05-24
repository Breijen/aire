/// Errors returned by aire.
#[derive(Debug)]
pub enum AireError {
    /// The command buffer between the main thread and the audio thread is full.
    ///
    /// This typically means commands are being sent faster than the audio
    /// thread can process them. The buffer holds 256 commands.
    CommandBufferFull,
    /// Sound file extension is not supported (yet).
    FileExtNotSupported(String)
}

impl std::fmt::Display for AireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AireError::CommandBufferFull => write!(f, "command buffer full"),
            AireError::FileExtNotSupported(ext) => write!(f, "unsupported format: .{}", ext),
        }
    }
}

impl std::error::Error for AireError {}

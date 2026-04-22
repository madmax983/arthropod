//! Testing and debugging utilities for Arthropod

pub(crate) mod frame_capture;
pub use frame_capture::*;
pub(crate) mod visual_test;
pub use visual_test::*;

use anyhow::Result;

/// Initialize observability for tests
pub fn init_test_tracing() {
    use tracing_subscriber::EnvFilter;

    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_target(true)
        .with_line_number(true)
        .with_test_writer()
        .try_init();
}

/// RenderDoc instance wrapper
pub struct RenderDocCapture {
    #[allow(dead_code)]
    rd: Option<renderdoc::RenderDoc<renderdoc::V141>>,
    capturing: bool,
}

impl Default for RenderDocCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderDocCapture {
    /// Try to initialize RenderDoc API
    pub fn new() -> Self {
        match renderdoc::RenderDoc::<renderdoc::V141>::new() {
            Ok(rd) => {
                tracing::info!("RenderDoc API initialized successfully");
                Self {
                    rd: Some(rd),
                    capturing: false,
                }
            }
            Err(e) => {
                tracing::warn!("RenderDoc not available: {:?}", e);
                Self {
                    rd: None,
                    capturing: false,
                }
            }
        }
    }

    /// Start capturing a frame
    pub fn start_capture(&mut self) -> Result<()> {
        if let Some(ref mut rd) = self.rd
            && !self.capturing
        {
            rd.start_frame_capture(std::ptr::null(), std::ptr::null());
            self.capturing = true;
            tracing::info!("Started RenderDoc frame capture");
        }
        Ok(())
    }

    /// End capturing and save the frame
    pub fn end_capture(&mut self) -> Result<()> {
        if let Some(ref mut rd) = self.rd
            && self.capturing
        {
            rd.end_frame_capture(std::ptr::null(), std::ptr::null());
            self.capturing = false;
            tracing::info!("Ended RenderDoc frame capture");
        }
        Ok(())
    }

    /// Check if RenderDoc is available
    pub fn is_available(&self) -> bool {
        self.rd.is_some()
    }
}

impl Drop for RenderDocCapture {
    fn drop(&mut self) {
        if self.capturing {
            let _ = self.end_capture();
        }
    }
}

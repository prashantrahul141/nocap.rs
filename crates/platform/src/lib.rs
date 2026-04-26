use thiserror::Error;

pub trait Platform: Sized + Default {
    fn create_screencapture_session(
        &mut self,
    ) -> impl std::future::Future<Output = Result<ScreenCaptureContext, ScreenCaptureError>> + Send;

    fn start_screencapture(&mut self) -> impl std::future::Future<Output = Result<(), ()>> + Send;

    fn take_screenshot(
        &self,
    ) -> impl std::future::Future<Output = Result<ScreenShot, ScreenShotError>> + Send;
}

#[derive(Debug, Error)]
pub enum ScreenCaptureError {
    #[error("Unknown error")]
    Unknown,

    #[cfg(target_os = "linux")]
    #[error("DBus error")]
    DBus(ashpd::Error),
}

#[derive(Debug)]
pub struct ScreenCaptureContext {}

#[derive(Debug, Error)]
pub enum ScreenShotError {
    #[error("Unknown error")]
    Unknown,

    #[cfg(target_os = "linux")]
    #[error("DBus error")]
    DBus(ashpd::Error),
}

#[derive(Debug)]
pub enum ScreenShot {
    Filepath(String),
}

use thiserror::Error;

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

pub trait Platform: Sized + Default {
    fn take_screenshot(
        &self,
    ) -> impl std::future::Future<Output = Result<ScreenShot, ScreenShotError>> + Send;
}

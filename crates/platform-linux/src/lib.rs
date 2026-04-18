use ashpd::desktop::screenshot::Screenshot;
use nocaprs_platform::{Platform, ScreenShot, ScreenShotError};

#[derive(Default, Debug)]
pub struct Linux {}

impl Platform for Linux {
    async fn take_screenshot(&self) -> Result<ScreenShot, ScreenShotError> {
        let req = Screenshot::request()
            .interactive(true)
            .modal(true)
            .send()
            .await
            .map_err(ScreenShotError::DBus)?;
        let resp = req.response().map_err(ScreenShotError::DBus)?;
        Ok(ScreenShot::Filepath(resp.uri().to_string()))
    }
}

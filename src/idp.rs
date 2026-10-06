use std::time::Duration;
use thirtyfour::prelude::*;

#[tracing::instrument(name = "idp.login", skip_all)]
pub async fn login(
    driver: &WebDriver,
    login: &str,
    password: &str,
    totp_code: impl FnOnce() -> WebDriverResult<String>,
) -> WebDriverResult<()> {
    driver
        .query(By::Id("core:loginuserpass"))
        .and_displayed()
        .first()
        .await?;

    let url = driver.current_url().await?;
    if url.scheme() != "https" || url.host_str() != Some("idp.zid.tuwien.ac.at")
    {
        return Err(WebDriverError::ParseError(
            "Refusing to submit credentials outside the TU Wien IdP".into(),
        ));
    }

    for (id, value) in [("username", login), ("password", password)] {
        let input = driver.query(By::Id(id)).and_displayed().first().await?;
        input.clear().await?;
        input.send_keys(value).await?;
    }

    let totp_code = totp_code()?;
    if !totp_code.is_empty() {
        let input =
            driver.query(By::Id("totp")).and_displayed().first().await?;
        input.clear().await?;
        input.send_keys(&totp_code).await?;
    }

    tracing::info!("Submitting TU Wien IdP login");
    driver
        .query(By::Id("samlloginbutton"))
        .and_displayed()
        .first()
        .await?
        .click()
        .await?;

    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let (redirected, error): (bool, String) = driver
                .execute(
                    r#"return [
                        location.hostname !== 'idp.zid.tuwien.ac.at',
                        document.querySelector('.message-box.error h3')
                            ?.textContent?.trim() || ''
                    ];"#,
                    vec![],
                )
                .await?
                .convert()?;

            if !error.is_empty() {
                return Err(WebDriverError::IoError(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("TU Wien IdP login failed: {error}"),
                )));
            }
            if redirected {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| {
        WebDriverError::Timeout("waiting for TU Wien IdP login result".into())
    })?
}

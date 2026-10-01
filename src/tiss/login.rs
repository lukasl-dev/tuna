use std::time::Duration;
use thirtyfour::prelude::*;

#[tracing::instrument(name = "tiss.login", skip_all)]
pub async fn login(
    driver: &WebDriver,
    login: &str,
    password: &str,
    totp_code: &str,
) -> WebDriverResult<()> {
    tracing::debug!("Opening login page");
    driver
        .goto("https://tiss.tuwien.ac.at/admin/authentifizierung")
        .await?;
    driver
        .query(By::Id("core:loginuserpass"))
        .and_displayed()
        .first()
        .await?;

    for (id, value) in [("username", login), ("password", password)] {
        let input = driver.query(By::Id(id)).and_displayed().first().await?;
        input.clear().await?;
        input.send_keys(value).await?;
    }

    if !totp_code.is_empty() {
        let input =
            driver.query(By::Id("totp")).and_displayed().first().await?;
        input.clear().await?;
        input.send_keys(totp_code).await?;
    }

    tracing::debug!("Submitting login form");
    driver
        .query(By::Id("samlloginbutton"))
        .and_displayed()
        .first()
        .await?
        .click()
        .await?;

    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let (logged_in, error): (bool, String) = driver
                .execute(
                    r#"return [
                        Boolean(document.querySelector('#logoutLink, .toolLogout')),
                        document.querySelector('.message-box.error h3')
                            ?.textContent?.trim() || ''
                    ];"#,
                    vec![],
                )
                .await?
                .convert()?;

            if logged_in {
                tracing::debug!("Login succeeded");
                return Ok(());
            }
            if !error.is_empty() {
                return Err(WebDriverError::IoError(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("TISS login failed: {error}"),
                )));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| WebDriverError::Timeout("waiting for TISS login result".into()))?
}

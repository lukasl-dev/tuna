use thirtyfour::prelude::*;

#[tracing::instrument(name = "tiss.login", skip_all)]
pub async fn login(
    driver: &WebDriver,
    login: &str,
    password: &str,
    totp_code: &str,
) -> WebDriverResult<()> {
    ensure_authenticated(driver, login, password, || Ok(totp_code.to_owned()))
        .await
}

#[tracing::instrument(name = "tiss.ensure_authenticated", skip_all)]
pub async fn ensure_authenticated(
    driver: &WebDriver,
    login: &str,
    password: &str,
    totp_code: impl FnOnce() -> WebDriverResult<String>,
) -> WebDriverResult<()> {
    driver
        .goto("https://tiss.tuwien.ac.at/admin/authentifizierung")
        .await?;
    driver
        .query(By::Css(
            "#logoutLink, .toolLogout, [id='core:loginuserpass']",
        ))
        .first()
        .await?;

    let logged_in: bool = driver
        .execute(
            "return location.origin === 'https://tiss.tuwien.ac.at' \
             && Boolean(document.querySelector('#logoutLink, .toolLogout'));",
            vec![],
        )
        .await?
        .convert()?;

    if logged_in {
        tracing::debug!("Reusing authenticated browser session");
        return Ok(());
    }

    tracing::info!("Authenticating with TISS");
    crate::idp::login(driver, login, password, totp_code).await?;

    driver
        .query(By::Css("#logoutLink, .toolLogout"))
        .first()
        .await?;

    if driver.current_url().await?.origin().ascii_serialization()
        != "https://tiss.tuwien.ac.at"
    {
        return Err(WebDriverError::ParseError(
            "TISS login returned an unexpected origin".into(),
        ));
    }

    Ok(())
}

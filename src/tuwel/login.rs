use thirtyfour::prelude::*;

const AUTHENTICATED: &str =
    "body:not(.notloggedin) a[href*='/login/logout.php']";
const LOGIN_PROVIDER: &str =
    "a.login-identityprovider-btn[href*='/auth/saml2/login.php']";

#[tracing::instrument(name = "tuwel.ensure_authenticated", skip_all)]
pub async fn ensure_authenticated(
    driver: &WebDriver,
    login: &str,
    password: &str,
    totp_code: impl FnOnce() -> WebDriverResult<String>,
) -> WebDriverResult<()> {
    driver.goto("https://tuwel.tuwien.ac.at/").await?;
    driver
        .query(By::Css(format!(
            "{AUTHENTICATED}, [id='core:loginuserpass'], {LOGIN_PROVIDER}"
        )))
        .first()
        .await?;

    let logged_in: bool = driver
        .execute(
            "return location.origin === 'https://tuwel.tuwien.ac.at' \
             && Boolean(document.querySelector(arguments[0]));",
            vec![AUTHENTICATED.into()],
        )
        .await?
        .convert()?;
    if logged_in {
        tracing::debug!("Reusing authenticated TUWEL session");
        return Ok(());
    }

    tracing::info!("Authenticating with TUWEL");
    if driver.current_url().await?.host_str() != Some("idp.zid.tuwien.ac.at") {
        driver
            .query(By::Css(LOGIN_PROVIDER))
            .and_displayed()
            .first()
            .await?
            .click()
            .await?;
    }

    driver
        .query(By::Css(format!(
            "{AUTHENTICATED}, [id='core:loginuserpass']"
        )))
        .first()
        .await?;

    if driver.current_url().await?.host_str() == Some("idp.zid.tuwien.ac.at") {
        crate::idp::login(driver, login, password, totp_code).await?;
    }

    driver.query(By::Css(AUTHENTICATED)).first().await?;

    if driver.current_url().await?.origin().ascii_serialization()
        != "https://tuwel.tuwien.ac.at"
    {
        return Err(WebDriverError::ParseError(
            "TUWEL login returned an unexpected origin".into(),
        ));
    }

    Ok(())
}

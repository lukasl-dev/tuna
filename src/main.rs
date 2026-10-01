use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    driver.goto("https://wikipedia.org").await?;
    let elem_form = driver
        .query(By::Id("search-form"))
        .desc("Wikipedia search form")
        .single()
        .await?;

    let elem_text = elem_form
        .query(By::Id("searchInput"))
        .desc("Wikipedia search input")
        .single()
        .await?;
    elem_text.send_keys("selenium").await?;

    let elem_button = elem_form
        .query(By::Css("button[type='submit']"))
        .desc("Wikipedia search button")
        .single()
        .await?;
    elem_button.click().await?;

    driver
        .query(By::ClassName("firstHeading"))
        .desc("Wikipedia article heading")
        .single()
        .await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    driver.quit().await?;

    Ok(())
}

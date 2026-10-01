use clap::Parser;
use thirtyfour::prelude::*;

#[derive(Parser)]
#[command(version, about = "Search Wikipedia using browser automation")]
struct Cli {
    /// Search terms
    query: String,

    /// WebDriver server URL
    #[arg(long, default_value = "http://localhost:9515")]
    webdriver: String,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let cli = Cli::parse();

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(&cli.webdriver, caps).await?;

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
    elem_text.send_keys(&cli.query).await?;

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
    println!("{}", driver.title().await?);

    driver.quit().await?;

    Ok(())
}

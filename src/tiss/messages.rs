use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub title: String,
    pub author: Option<String>,
    pub url: String,
    pub body: String,
    #[serde(deserialize_with = "parse_published_at")]
    pub published_at: NaiveDateTime,
    pub tags: Vec<String>,
}

fn parse_published_at<'de, D>(
    deserializer: D,
) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&text, "%d.%m.%Y %H:%M")
        .map_err(serde::de::Error::custom)
}

#[tracing::instrument(name = "tiss.messages", skip(driver))]
pub async fn messages(driver: &WebDriver) -> WebDriverResult<Vec<Message>> {
    tracing::debug!("Opening messages page");
    driver
        .goto("https://tiss.tuwien.ac.at/education/messages.xhtml")
        .await?;
    driver
        .query(By::Id("messagesForm"))
        .and_displayed()
        .first()
        .await?;

    driver
        .execute(
            r#"const form = document.getElementById('messagesForm');
            const list = form.querySelector(':scope > ol');
            if (!list) throw new Error('TISS message list not found');
            const directText = element => Array.from(element.childNodes)
                .filter(node => node.nodeType === Node.TEXT_NODE)
                .map(node => node.textContent).join('').trim();

            return Array.from(list.querySelectorAll(':scope > li.clearfix'), item => {
                const heading = item.querySelector(':scope > h2');
                const link = heading?.querySelector('a');
                const body = item.querySelector('.encode');
                const metadata = item.querySelector('.light');
                if (!heading || !link || !body || !metadata)
                    throw new Error('Unexpected TISS message structure');

                return {
                    title: link.textContent.trim(),
                    author: directText(heading).replace(/:\s*$/, '').trim() || null,
                    url: link.href,
                    body: body.innerText.trim(),
                    published_at: directText(metadata),
                    tags: Array.from(metadata.querySelectorAll('a'), tag => tag.textContent.trim())
                };
            });"#,
            vec![],
        )
        .await?
        .convert()
}

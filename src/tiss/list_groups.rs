use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};
use thirtyfour::prelude::*;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub participants: u32,
    pub max_participants: u32,
    #[serde(deserialize_with = "parse_application_begin")]
    pub application_begin: Option<NaiveDateTime>,
}

fn parse_application_begin<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    if text.trim().is_empty() {
        return Ok(None);
    }
    NaiveDateTime::parse_from_str(text.trim(), "%d.%m.%Y, %H:%M")
        .map(Some)
        .map_err(serde::de::Error::custom)
}

#[tracing::instrument(name = "tiss.groups.list", skip(driver))]
pub async fn list_groups(
    driver: &WebDriver,
    semester: &str,
    course: &str,
) -> WebDriverResult<Vec<Group>> {
    let mut url = Url::parse(
        "https://tiss.tuwien.ac.at/education/course/groupList.xhtml",
    )
    .map_err(WebDriverError::InvalidUrl)?;
    url.query_pairs_mut()
        .append_pair("semester", semester)
        .append_pair("courseNr", course);

    tracing::debug!("Opening group list");
    driver.goto(url.as_str()).await?;
    driver
        .query(By::Id("groupContentForm:groupListPanel"))
        .and_displayed()
        .first()
        .await?;

    driver
        .execute(
            r#"const panel = document.getElementById('groupContentForm:groupListPanel');
            return Array.from(panel.querySelectorAll('.groupWrapper'), group => {
                const items = Array.from(group.querySelectorAll('li'));
                const item = label => items.find(li =>
                    li.querySelector('label')?.textContent?.trim() === label);
                const counts = item('Participants')?.textContent?.match(/\d+/g) || [];

                return {
                    name: group.querySelector('.groupHeaderWrapper .titleColStudent span.bold')
                        ?.textContent?.trim() || '',
                    participants: Number(counts[0] || 0),
                    max_participants: Number(counts[1] || 0),
                    application_begin: item('Application begin')?.querySelector('span')
                        ?.textContent?.trim() || ''
                };
            });"#,
            vec![],
        )
        .await?
        .convert()
}

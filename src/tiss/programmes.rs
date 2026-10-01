use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Programme {
    pub category: String,
    pub code: Option<String>,
    pub title: String,
    pub url: String,
    pub note: Option<String>,
}

#[tracing::instrument(name = "tiss.programmes", skip(driver))]
pub async fn programmes(driver: &WebDriver) -> WebDriverResult<Vec<Programme>> {
    tracing::debug!("Opening programmes page");
    driver
        .goto("https://tiss.tuwien.ac.at/curriculum/studyCodes.xhtml")
        .await?;
    driver
        .query(By::Id("studyCodeListForm"))
        .and_displayed()
        .first()
        .await?;

    driver
        .execute(
            r#"const form = document.getElementById('studyCodeListForm');
            return Array.from(form.querySelectorAll(':scope > table.standard')).flatMap(table => {
                const heading = table.previousElementSibling;
                if (heading?.tagName !== 'H2' || !table.tBodies[0])
                    throw new Error('Unexpected TISS programme category structure');

                return Array.from(table.tBodies[0].rows, row => {
                    const code = row.querySelector('.studyCodeColumn');
                    const title = row.querySelector('.studyCodeNameColumn');
                    const link = title?.querySelector('a');
                    if (!code || !title || !link)
                        throw new Error('Unexpected TISS programme row structure');

                    return {
                        category: heading.textContent.trim(),
                        code: code.textContent.trim() || null,
                        title: link.textContent.trim(),
                        url: link.href,
                        note: title.querySelector('.smallText')?.textContent.trim() || null
                    };
                });
            });"#,
            vec![],
        )
        .await?
        .convert()
}

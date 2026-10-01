pub mod exams;

use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub code: String,
    pub title: String,
    pub semester: String,
    pub course_type: String,
    pub semester_hours: f64,
    pub credits: f64,
    pub url: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Section {
    pub title: String,
    pub body: String,
}

#[tracing::instrument(name = "tiss.courses.get", skip(driver))]
pub async fn get(
    driver: &WebDriver,
    semester: &str,
    course: &str,
) -> WebDriverResult<Course> {
    let mut url =
        Url::parse("https://tiss.tuwien.ac.at/course/educationDetails.xhtml")
            .map_err(WebDriverError::InvalidUrl)?;
    url.query_pairs_mut()
        .append_pair("semester", semester)
        .append_pair("courseNr", course);

    tracing::debug!("Opening course description");
    driver.goto(url.as_str()).await?;
    driver
        .query(By::Id("detailLink"))
        .and_displayed()
        .first()
        .await?;

    driver
        .execute(
            r#"const content = document.getElementById('contentInner');
            const heading = content.querySelector(':scope > h1');
            const form = content.querySelector(':scope > form');
            const summary = document.getElementById('subHeader').textContent.trim()
                .match(/^(\d{4}[WS]),\s*([^,]+),\s*([\d.]+)h,\s*([\d.]+)EC$/);
            const code = heading?.querySelector('.light')?.textContent.trim();
            const link = document.querySelector('#detailLink a');
            if (!heading || !form || !summary || !code || !link)
                throw new Error('Unexpected TISS course structure');
            if (summary[1] !== arguments[0] || code.replace(/[.\s]/g, '') !== arguments[1].replace(/[.\s]/g, ''))
                throw new Error('TISS returned a different course or semester');

            const sections = Array.from(form.querySelectorAll(':scope > h2'), heading => {
                const parts = [];
                for (let node = heading.nextSibling; node; node = node.nextSibling) {
                    if (node.nodeType === Node.ELEMENT_NODE && node.tagName === 'H2') break;
                    if (node.nodeType === Node.TEXT_NODE) {
                        parts.push(node.textContent.trim());
                    } else if (node.nodeType === Node.ELEMENT_NODE &&
                        !node.matches('script, style, input, .ui-dialog') && node.getClientRects().length) {
                        parts.push(node.innerText.trim());
                    }
                }
                return {title: heading.textContent.trim(), body: parts.filter(Boolean).join('\n\n')};
            });

            return {
                code,
                title: Array.from(heading.childNodes).filter(node => node.nodeType === Node.TEXT_NODE)
                    .map(node => node.textContent).join('').trim(),
                semester: summary[1],
                course_type: summary[2].trim(),
                semester_hours: Number(summary[3]),
                credits: Number(summary[4]),
                url: link.href,
                sections
            };"#,
            vec![semester.into(), course.into()],
        )
        .await?
        .convert()
}

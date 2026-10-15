use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Quiz {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub course: Option<Course>,
    pub description: String,
    pub dates: Vec<Detail>,
    pub conditions: Vec<String>,
    pub notices: Vec<String>,
    pub result: Option<String>,
    pub attempts: Vec<Attempt>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Detail {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attempt {
    pub title: Option<String>,
    pub details: Vec<Detail>,
    pub review_url: Option<String>,
}

#[tracing::instrument(name = "tuwel.quizzes.get", skip(driver))]
pub async fn get(driver: &WebDriver, id: u64) -> WebDriverResult<Quiz> {
    if id == 0 || id > 9_007_199_254_740_991 {
        return Err(WebDriverError::ParseError(
            "Quiz course-module id must be a positive safe integer".into(),
        ));
    }

    tracing::debug!("Opening quiz overview");
    driver
        .goto(format!(
            "https://tuwel.tuwien.ac.at/mod/quiz/view.php?id={id}"
        ))
        .await?;
    driver
        .query(By::Css("body#page-mod-quiz-view #region-main"))
        .first()
        .await?;

    driver
        .execute(
            r#"const expected = arguments[0];
            const page = new URL(location.href);
            if (page.origin !== 'https://tuwel.tuwien.ac.at' ||
                page.pathname !== '/mod/quiz/view.php' ||
                page.searchParams.get('id') !== String(expected))
                throw new Error('TUWEL returned a different quiz overview');

            const main = document.getElementById('region-main');
            const text = element => element?.innerText.trim() || '';
            const visible = element => element.getClientRects().length > 0;
            const activityName = main.querySelector('[data-region="activity-information"]')
                ?.dataset.activityname?.trim();
            const header = main.querySelector('[data-for="page-activity-header"], .activity-header');
            const heading = header?.previousElementSibling;
            const breadcrumb = document.querySelector('#page-navbar .breadcrumb li:last-child');
            const breadcrumbLink = breadcrumb?.querySelector('a[href]');
            const breadcrumbUrl = breadcrumbLink ? new URL(breadcrumbLink.href) : null;
            const currentBreadcrumb = breadcrumbUrl
                ? breadcrumbUrl.origin === page.origin && breadcrumbUrl.pathname === page.pathname
                    && breadcrumbUrl.searchParams.get('id') === String(expected)
                : breadcrumb?.matches('.active, [aria-current="page"]');
            const title = activityName || (heading?.matches('h2') ? text(heading)
                : currentBreadcrumb ? text(breadcrumb) : '');
            if (!title) throw new Error('TUWEL quiz title is missing');

            const courseLink = Array.from(document.querySelectorAll('.breadcrumb a')).find(link => {
                const url = new URL(link.href);
                return url.origin === page.origin && url.pathname === '/course/view.php';
            });
            let course = null;
            if (courseLink) {
                const url = new URL(courseLink.href);
                const id = Number(url.searchParams.get('id'));
                if (!Number.isSafeInteger(id) || id <= 0)
                    throw new Error('Unexpected TUWEL course id');
                course = {id, name: text(courseLink), url: `${page.origin}/course/view.php?id=${id}`};
            }

            const reviewUrl = container => {
                const link = Array.from(container.querySelectorAll('a[href]')).filter(visible).find(link => {
                    const url = new URL(link.href);
                    return url.origin === page.origin && url.pathname === '/mod/quiz/review.php';
                });
                const form = Array.from(container.querySelectorAll('form[action]')).filter(visible).find(form => {
                    const url = new URL(form.action);
                    return url.origin === page.origin && url.pathname === '/mod/quiz/review.php';
                });
                const raw = link ? new URL(link.href).searchParams.get('attempt')
                    : form?.querySelector('input[name="attempt"]')?.value;
                if (!raw) return null;
                if (!/^\d+$/.test(raw) || Number(raw) <= 0)
                    throw new Error('Unexpected TUWEL quiz attempt id');
                return `${page.origin}/mod/quiz/review.php?attempt=${raw}`;
            };

            const attempts = Array.from(main.querySelectorAll('.card')).filter(visible).map(card => {
                const table = card.querySelector('table.quizreviewsummary');
                if (!table) return null;
                const details = Array.from(table.querySelectorAll('tbody tr')).filter(visible).map(row => {
                    const label = row.querySelector('th');
                    const value = row.querySelector('td');
                    if (!label || !value) throw new Error('Unexpected TUWEL attempt summary');
                    return {label: text(label), value: text(value)};
                });
                return {title: text(card.querySelector('.card-title')) || null,
                    details, review_url: reviewUrl(card)};
            }).filter(Boolean);

            for (const table of main.querySelectorAll('table.quizattemptsummary')) {
                if (!visible(table)) continue;
                const headers = Array.from(table.querySelectorAll('thead th'), text);
                for (const row of table.querySelectorAll('tbody tr')) {
                    if (!visible(row)) continue;
                    const cells = Array.from(row.cells);
                    if (cells.length !== headers.length)
                        throw new Error('Unexpected TUWEL attempt table');
                    attempts.push({title: null,
                        details: cells.map((cell, index) => ({label: headers[index], value: text(cell)})),
                        review_url: reviewUrl(row)});
                }
            }

            const dates = Array.from(main.querySelectorAll('[data-region="activity-dates"] > div'))
                .filter(visible).map(date => {
                const label = date.querySelector('strong');
                if (!label) throw new Error('Unexpected TUWEL quiz date');
                const labelText = text(label);
                return {label: labelText, value: text(date).slice(labelText.length).trim()};
            });
            const paragraphs = selector => Array.from(main.querySelectorAll(selector))
                .filter(visible).map(text).filter(Boolean);

            return {
                id: expected,
                title,
                url: `${page.origin}/mod/quiz/view.php?id=${expected}`,
                course,
                description: text(main.querySelector('#intro')),
                dates,
                conditions: paragraphs('.quizinfo p'),
                notices: paragraphs('.quizattempt p, .alert:not(.d-none)'),
                result: text(main.querySelector('#feedback')) || null,
                attempts
            };"#,
            vec![id.into()],
        )
        .await?
        .convert()
}

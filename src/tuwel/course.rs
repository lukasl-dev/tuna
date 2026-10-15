use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub format: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Section {
    pub id: u64,
    pub number: u32,
    pub name: Option<String>,
    pub url: String,
    pub summary: String,
    pub links: Vec<Link>,
    pub availability: Option<String>,
    pub activities: Vec<Activity>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub id: u64,
    pub activity_type: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: String,
    pub links: Vec<Link>,
    pub availability: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
enum Response {
    Ok(Course),
    Error(String),
}

#[tracing::instrument(name = "tuwel.course.get", skip(driver))]
pub async fn get(driver: &WebDriver, id: u64) -> WebDriverResult<Course> {
    if id == 0 || id > 9_007_199_254_740_991 {
        return Err(WebDriverError::ParseError(
            "Course id must be a positive safe integer".into(),
        ));
    }

    tracing::debug!("Opening course overview");
    driver
        .goto(format!(
            "https://tuwel.tuwien.ac.at/course/view.php?id={id}"
        ))
        .await?;
    driver.query(By::Css("#region-main")).first().await?;

    let response: Response = driver
        .execute_async(
            r#"const expected = arguments[0];
            const done = arguments[arguments.length - 1];
            const load = async () => {
                const page = new URL(location.href);
                if (page.origin !== 'https://tuwel.tuwien.ac.at' ||
                    page.pathname !== '/course/view.php' ||
                    page.searchParams.get('id') !== String(expected) ||
                    !document.body.classList.contains(`course-${expected}`))
                    throw new Error('TUWEL returned a different course overview');

                const content = document.querySelector('#region-main .course-content');
                if (!content) throw new Error('TUWEL course content is missing or unavailable');
                const ignored = 'script,style,noscript,svg,button,input,select,textarea,form,' +
                    '.accesshide,.sr-only,.visually-hidden,.d-none,[hidden]';
                const clean = element => {
                    const copy = element?.cloneNode(true) || document.createElement('div');
                    copy.querySelectorAll(ignored).forEach(element => element.remove());
                    return copy;
                };
                const text = element => {
                    const copy = clean(element);
                    copy.querySelectorAll('br').forEach(element => element.replaceWith('\n'));
                    copy.querySelectorAll('p,div,li,h1,h2,h3,h4,h5,h6,tr,pre')
                        .forEach(element => element.append('\n'));
                    return copy.textContent.replace(/\u00a0/g, ' ').replace(/[ \t]+/g, ' ')
                        .split('\n').map(line => line.trim()).join('\n')
                        .replace(/\n{3,}/g, '\n\n').trim();
                };
                const name = text(document.querySelector('#page-header h1'));
                const format = Array.from(document.body.classList)
                    .find(value => value.startsWith('format-'))?.slice(7);
                if (!name || !format) throw new Error('TUWEL course metadata is missing');

                const integer = (raw, label, minimum = 1) => {
                    const value = Number(raw);
                    if (!/^\d+$/.test(raw || '') || !Number.isSafeInteger(value) || value < minimum)
                        throw new Error(`Unexpected TUWEL ${label}`);
                    return value;
                };
                const safeUrl = raw => {
                    if (!raw?.trim() || raw.trim().startsWith('#')) return null;
                    const url = new URL(raw, page.href);
                    if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password)
                        return null;
                    if (Array.from(url.searchParams.keys()).some(key =>
                        /sesskey|token|password|secret|auth|key/i.test(key))) return null;
                    if (url.origin === page.origin &&
                        /\/(login|enrol)\/|\/(edit|delete|startattempt|processattempt|set)\.php$/.test(url.pathname))
                        return null;
                    return url.href;
                };
                const links = element => Array.from(clean(element).querySelectorAll('a[href]'), link =>
                    ({label: text(link), url: safeUrl(link.getAttribute('href'))}))
                    .filter(link => link.url !== null);
                const extract = (node, id, number, fallbackName) => {
                    const summary = node.querySelector('.summarytext, .summary');
                    const activities = Array.from(node.querySelectorAll('.activity:not(.spacer)'))
                        .filter(activity => activity.closest('.course-section') === node)
                        .map(activity => {
                            const id = integer(activity.dataset.id || activity.dataset.cmid ||
                                activity.id.replace(/^module-/, ''), 'activity id');
                            const type = Array.from(activity.classList)
                                .find(value => value.startsWith('modtype_'))?.slice(8);
                            if (!type) throw new Error('TUWEL activity type is missing');
                            const title = activity.querySelector('[data-activityname]')?.dataset.activityname?.trim() ||
                                text(activity.querySelector('.instancename, .activityname')) ||
                                activity.dataset.title?.trim() || null;
                            const target = activity.querySelector('.activityname a[href], a.cm-link[href]');
                            let url = null;
                            if (target) {
                                const actual = new URL(target.getAttribute('href'), page.href);
                                if (actual.origin === page.origin &&
                                    actual.pathname === `/mod/${type}/view.php` &&
                                    actual.searchParams.get('id') === String(id))
                                    url = `${page.origin}/mod/${type}/view.php?id=${id}`;
                                else url = safeUrl(target.getAttribute('href'));
                            }
                            const description = activity.querySelector(
                                '.activity-altcontent, .contentwithoutlink, .modal-description');
                            return {id, activity_type: type, name: title, url,
                                description: text(description), links: links(description),
                                availability: text(activity.querySelector('.availabilityinfo')) || null};
                        });
                    return {id, number,
                        name: text(node.querySelector('.sectionname, .sectiontitle h2')) || fallbackName,
                        url: `${page.origin}/course/section.php?id=${id}`,
                        summary: text(summary), links: links(summary),
                        availability: text(node.querySelector('.section_availability')) || null,
                        activities};
                };

                const tiles = new Map(Array.from(content.querySelectorAll('.tile[data-true-sectionid]'), tile => {
                    const number = integer(tile.dataset.section, 'section number', 0);
                    return [number, {id: integer(tile.dataset.trueSectionid, 'section id'),
                        name: text(tile.querySelector('h3')) || null}];
                }));
                const sections = [];
                const ids = new Set();
                const numbers = new Set();
                const activityIds = new Set();
                const signal = AbortSignal.timeout(20000);
                for (let node of content.querySelectorAll('.course-section')) {
                    const number = integer(node.dataset.number ?? node.dataset.section ??
                        node.id.replace(/^section-/, ''), 'section number', 0);
                    const id = integer(node.dataset.id || node.dataset.sectionid, 'section id');
                    if (ids.has(id) || numbers.has(number))
                        throw new Error('Duplicate TUWEL course section');
                    ids.add(id);
                    numbers.add(number);
                    const tile = tiles.get(number);
                    if (tile && tile.id !== id) throw new Error('TUWEL tile section id mismatch');
                    if (tile && node.childElementCount === 0) {
                        const url = `${page.origin}/course/section.php?id=${id}`;
                        const response = await fetch(url, {credentials: 'same-origin', redirect: 'error', signal});
                        if (!response.ok) throw new Error('Could not load TUWEL course section');
                        const template = document.createElement('template');
                        template.innerHTML = await response.text();
                        const candidates = Array.from(template.content.querySelectorAll(
                            '#region-main .course-content .course-section'));
                        node = candidates.find(section => section.dataset.sectionid === String(id) &&
                            section.dataset.section === String(number));
                        if (!node || node.childElementCount === 0)
                            throw new Error('TUWEL returned a missing or different course section');
                    }
                    const section = extract(node, id, number, tile?.name || null);
                    for (const activity of section.activities) {
                        if (activityIds.has(activity.id)) throw new Error('Duplicate TUWEL course activity');
                        activityIds.add(activity.id);
                    }
                    sections.push(section);
                }
                if (!sections.length || Array.from(tiles.keys()).some(number => !numbers.has(number)))
                    throw new Error('Unsupported or incomplete TUWEL course section layout');
                sections.sort((a, b) => a.number - b.number);
                return {id: expected, name, url: `${page.origin}/course/view.php?id=${expected}`, format, sections};
            };
            load().then(value => done({status: 'ok', value}),
                error => done({status: 'error', value: error.message || 'Could not read TUWEL course'}));"#,
            vec![id.into()],
        )
        .await?
        .convert()?;

    match response {
        Response::Ok(course) => Ok(course),
        Response::Error(message) => Err(WebDriverError::ParseError(message)),
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: u64,
    pub title: String,
    pub event_type: String,
    pub activity_type: Option<String>,
    pub at: DateTime<Utc>,
    pub overdue: bool,
    pub url: String,
    pub course: Option<Course>,
    pub action: Option<Action>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Action {
    pub label: String,
    pub available: bool,
    pub item_count: u32,
}

#[derive(Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
enum Response {
    Ok(Vec<Event>),
    Error(String),
}

#[tracing::instrument(name = "tuwel.timeline", skip(driver))]
pub async fn timeline(driver: &WebDriver) -> WebDriverResult<Vec<Event>> {
    tracing::debug!("Opening dashboard timeline");
    driver.goto("https://tuwel.tuwien.ac.at/my/").await?;
    driver
        .query(By::Css(
            ".block_timeline [data-region='event-list-container'][data-midnight]",
        ))
        .first()
        .await?;

    let response: Response = driver
        .execute_async(
            r#"const done = arguments[arguments.length - 1];
            const fail = error => done({status: 'error', value:
                error?.message || 'Could not fetch TUWEL timeline'});
            if (location.origin !== 'https://tuwel.tuwien.ac.at') {
                fail(new Error('Unexpected TUWEL timeline origin'));
                return;
            }

            const block = document.querySelector('.block_timeline');
            const list = block?.querySelector('[data-region="event-list-container"][data-midnight]');
            const all = block?.querySelector('[data-filtername="all"][data-from]');
            const midnight = Number(list?.dataset.midnight);
            const offset = Number(all?.dataset.from);
            if (!list || !all || !Number.isSafeInteger(midnight) || midnight <= 0 ||
                !Number.isSafeInteger(offset)) {
                fail(new Error('Unexpected TUWEL timeline structure'));
                return;
            }

            const id = value => {
                if (!Number.isSafeInteger(value) || value <= 0)
                    throw new Error('Unexpected TUWEL timeline id');
                return value;
            };
            const text = html => {
                if (typeof html !== 'string')
                    throw new Error('Unexpected TUWEL timeline text');
                const template = document.createElement('template');
                template.innerHTML = html;
                template.content.querySelectorAll('script, style').forEach(element => element.remove());
                return template.content.textContent.trim();
            };
            const timestamp = seconds => {
                if (!Number.isSafeInteger(seconds))
                    throw new Error('Unexpected TUWEL timeline timestamp');
                return new Date(seconds * 1000).toISOString();
            };

            require(['core/ajax'], Ajax => {
                const load = async () => {
                    const events = [];
                    const cursors = new Set([0]);
                    let aftereventid = 0;
                    for (;;) {
                        const result = await Ajax.call([{
                            methodname: 'core_calendar_get_action_events_by_timesort',
                            args: {
                                timesortfrom: midnight + offset * 86400,
                                aftereventid,
                                limitnum: 50,
                                limittononsuspendedevents: true
                            }
                        }])[0];
                        if (!Array.isArray(result.events))
                            throw new Error('Unexpected TUWEL timeline response');

                        for (const event of result.events) {
                            if (event.eventtype === 'open' || event.eventtype === 'opensubmission') {
                                if (!Number.isSafeInteger(event.timeusermidnight))
                                    throw new Error('Unexpected TUWEL timeline day timestamp');
                                if (event.timeusermidnight <= midnight) continue;
                            }
                            if (typeof event.overdue !== 'boolean')
                                throw new Error('Unexpected TUWEL timeline overdue status');

                            events.push({
                                id: id(event.id),
                                title: text(event.activityname || event.name),
                                event_type: event.eventtype,
                                activity_type: event.modulename || null,
                                at: timestamp(event.timesort),
                                overdue: event.overdue,
                                url: event.url,
                                course: event.course ? {
                                    id: id(event.course.id),
                                    name: text(event.course.fullnamedisplay),
                                    url: event.course.viewurl
                                } : null,
                                action: event.action ? {
                                    label: text(event.action.name),
                                    available: event.action.actionable,
                                    item_count: event.action.itemcount
                                } : null
                            });
                        }

                        if (result.events.length < 50) break;
                        aftereventid = id(result.lastid);
                        if (cursors.has(aftereventid))
                            throw new Error('TUWEL timeline pagination did not advance');
                        cursors.add(aftereventid);
                    }

                    return events;
                };
                load().then(events => done({status: 'ok', value: events}), fail);
            }, fail);"#,
            vec![],
        )
        .await?
        .convert()?;

    match response {
        Response::Ok(events) => Ok(events),
        Response::Error(error) => Err(WebDriverError::ParseError(error)),
    }
}

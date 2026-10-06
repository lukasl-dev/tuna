use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub url: Option<String>,
    pub url_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
enum Response {
    Ok(Vec<Notification>),
    Error(String),
}

#[tracing::instrument(name = "tuwel.notifications.list", skip(driver))]
pub async fn list(driver: &WebDriver) -> WebDriverResult<Vec<Notification>> {
    tracing::debug!("Opening notifications page");
    driver
        .goto(
            "https://tuwel.tuwien.ac.at/message/output/popup/notifications.php",
        )
        .await?;
    driver
        .query(By::Css("[data-region='notification-area']"))
        .first()
        .await?;

    let response: Response = driver
        .execute_async(
            r#"const done = arguments[arguments.length - 1];
            const fail = error => done({status: 'error',
                value: error.message || 'Could not fetch TUWEL notifications'});
            if (location.origin !== 'https://tuwel.tuwien.ac.at') {
                fail(new Error('Unexpected TUWEL notifications origin'));
                return;
            }

            require(['core/ajax'], Ajax => {
                Ajax.call([{
                    methodname: 'message_popup_get_popup_notifications',
                    args: {useridto: 0, newestfirst: true, limit: 0, offset: 0}
                }])[0].then(result => {
                    try {
                        if (!Array.isArray(result.notifications))
                            throw new Error('Unexpected TUWEL notifications response');

                        const timestamp = seconds => {
                            if (!Number.isSafeInteger(seconds))
                                throw new Error('Unexpected TUWEL notification timestamp');
                            return new Date(seconds * 1000).toISOString();
                        };
                        const notifications = result.notifications.map(notification => {
                            if (!Number.isSafeInteger(notification.id) || notification.id < 0)
                                throw new Error('Unexpected TUWEL notification id');
                            if (typeof notification.read !== 'boolean')
                                throw new Error('Unexpected TUWEL notification read status');
                            if (typeof notification.text !== 'string')
                                throw new Error('Unexpected TUWEL notification body');
                            const body = document.createElement('template');
                            body.innerHTML = notification.text;
                            body.content.querySelectorAll('script, style').forEach(element => element.remove());
                            body.content.querySelectorAll('br').forEach(element => element.replaceWith('\n'));
                            body.content.querySelectorAll('p, div, li, blockquote, h1, h2, h3, h4, h5, h6')
                                .forEach(element => element.append('\n'));

                            return {
                                id: notification.id,
                                title: notification.subject,
                                body: body.content.textContent.trim(),
                                url: notification.contexturl || null,
                                url_name: notification.contexturlname || null,
                                created_at: timestamp(notification.timecreated),
                                read: notification.read,
                                read_at: notification.timeread ? timestamp(notification.timeread) : null
                            };
                        });

                        done({status: 'ok', value: notifications});
                    } catch (error) {
                        fail(error);
                    }
                }, fail);
            }, fail);"#,
            vec![],
        )
        .await?
        .convert()?;

    match response {
        Response::Ok(notifications) => Ok(notifications),
        Response::Error(error) => Err(WebDriverError::ParseError(error)),
    }
}

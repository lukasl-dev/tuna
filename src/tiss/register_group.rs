use thirtyfour::prelude::*;
use url::Url;

#[tracing::instrument(name = "tiss.groups.register", skip(driver))]
pub async fn register_group(
    driver: &WebDriver,
    semester: &str,
    course: &str,
    group: &str,
) -> WebDriverResult<String> {
    let mut url = Url::parse(
        "https://tiss.tuwien.ac.at/education/course/groupList.xhtml",
    )
    .map_err(WebDriverError::InvalidUrl)?;
    url.query_pairs_mut()
        .append_pair("semester", semester)
        .append_pair("courseNr", course);

    tracing::debug!("Opening group registration page");
    driver.goto(url.as_str()).await?;
    driver
        .query(By::Id("groupContentForm:groupListPanel"))
        .and_displayed()
        .first()
        .await?;

    tracing::debug!("Submitting group registration");
    driver
        .execute(
            r#"const normalize = text => text.replace(/\s+/g, ' ').trim().toLowerCase();
            const target = normalize(arguments[0]);
            if (!target) throw new Error('Group name must not be empty');

            const code = document.querySelector('#contentInner h1 .light')?.textContent.trim();
            const semester = document.querySelector('#semesterForm select')?.value;
            if (!code || semester !== arguments[1] ||
                code.replace(/[.\s]/g, '') !== arguments[2].replace(/[.\s]/g, ''))
                throw new Error('TISS returned a different course or semester');

            const panel = document.getElementById('groupContentForm:groupListPanel');
            const groups = Array.from(panel.querySelectorAll('.groupWrapper'), wrapper => ({
                wrapper,
                name: wrapper.querySelector('.groupHeaderWrapper .titleColStudent span.bold')
                    ?.textContent.trim() || ''
            }));
            const matches = groups.filter(group => normalize(group.name) === target);
            if (matches.length === 0) {
                const available = groups.map(group => group.name).filter(Boolean);
                throw new Error(available.length
                    ? `Group not found. Available groups: ${available.join(', ')}`
                    : `No groups available: ${panel.textContent.replace(/\s+/g, ' ').trim()}`);
            }
            if (matches.length !== 1) throw new Error('Group name is ambiguous');

            const button = matches[0].wrapper.querySelector(
                'input[type="submit"][name^="groupContentForm:"]');
            if (!button) throw new Error('Group has no registration button');
            if (button.disabled) throw new Error('Group registration is disabled');
            if (/\b(deregister|unregister|abmelden|abmeldung|cancel|withdraw)\b/i.test(button.value))
                throw new Error('Refusing to click a deregistration button');

            button.click();"#,
            vec![group.into(), semester.into(), course.into()],
        )
        .await?;

    let confirmation = driver
        .query(By::Css(
            "form[id='regForm'] ul.styledCommandBox li:first-child \
             input[type='submit'][name^='regForm:']",
        ))
        .and_displayed()
        .and_enabled()
        .first()
        .await?;

    tracing::debug!("Confirming group registration");
    confirmation.click().await?;

    let message = driver
        .query(By::Css("form[id='confirmForm'] div.staticInfoMessage"))
        .and_displayed()
        .first()
        .await?
        .text()
        .await?
        .trim()
        .to_owned();
    if message.is_empty() {
        return Err(WebDriverError::ParseError(
            "Group registration success message is empty".to_owned(),
        ));
    }

    tracing::info!(%message, "Group registration completed");
    Ok(message)
}

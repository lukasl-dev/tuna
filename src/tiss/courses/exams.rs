use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Exam {
    pub id: String,
    pub name: String,
    pub dates: Vec<NaiveDate>,
    pub lecture_semester: Option<String>,
    pub mode: Option<String>,
    pub participants: Option<u32>,
    pub max_participants: Option<u32>,
    pub waiting_list: Option<u32>,
    pub application_begin: Option<NaiveDateTime>,
    pub application_end: Option<NaiveDateTime>,
    pub deregistration_end: Option<NaiveDateTime>,
    pub registration_type: Option<String>,
    pub registration_confirmation: Option<String>,
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub date: Option<NaiveDate>,
    pub begin: Option<NaiveTime>,
    pub end: Option<NaiveTime>,
    pub location: String,
    pub comment: String,
}

#[tracing::instrument(name = "tiss.courses.exams", skip(driver))]
pub async fn list(
    driver: &WebDriver,
    semester: &str,
    course: &str,
) -> WebDriverResult<Vec<Exam>> {
    let mut url = Url::parse(
        "https://tiss.tuwien.ac.at/education/course/examDateList.xhtml",
    )
    .map_err(WebDriverError::InvalidUrl)?;
    url.query_pairs_mut()
        .append_pair("semester", semester)
        .append_pair("courseNr", course);

    tracing::debug!("Opening course exams");
    driver.goto(url.as_str()).await?;
    driver
        .query(By::Id("examDateListForm:examDateListPanel"))
        .and_displayed()
        .first()
        .await?;

    driver
        .execute(
            r#"const panel = document.getElementById('examDateListForm:examDateListPanel');
            const code = document.querySelector('#contentInner h1 .light')?.textContent.trim();
            const semester = document.querySelector('#semesterForm select')?.value;
            if (!code || semester !== arguments[0] ||
                code.replace(/[.\s]/g, '') !== arguments[1].replace(/[.\s]/g, ''))
                throw new Error('TISS returned a different course or semester');

            const date = text => {
                if (!text) return null;
                const match = text.match(/^(\d{2})\.(\d{2})\.(\d{4})(?:,\s*(\d{2}):(\d{2}))?$/);
                if (!match) throw new Error('Unexpected TISS exam date format');
                const day = `${match[3]}-${match[2]}-${match[1]}`;
                return match[4] ? `${day}T${match[4]}:${match[5]}:00` : day;
            };

            const time = text => {
                if (!text) return null;
                if (!/^\d{2}:\d{2}$/.test(text))
                    throw new Error('Unexpected TISS exam time format');
                return `${text}:00`;
            };

            return Array.from(panel.querySelectorAll(':scope > .groupWrapper'), group => {
                const title = group.querySelector('.titleColStudent');
                const name = title?.querySelector('.bold');
                const details = group.querySelector('.toggleAll');
                const id = details?.id.match(/^toggleContent(\d+)$/)?.[1];
                if (!title || !name || !details || !id)
                    throw new Error('Unexpected TISS exam structure');

                const field = suffix => details.querySelector(`[id$=':${suffix}']`)
                    ?.textContent.trim() || null;
                const counts = field('members')?.match(/\d+/g) || [];
                const waiting = field('waitingList');
                const headerDates = Array.from(title.childNodes)
                    .filter(node => node.nodeType === Node.TEXT_NODE)
                    .map(node => node.textContent).join('').match(/\d{2}\.\d{2}\.\d{4}/g) || [];

                const events = Array.from(details.querySelectorAll('table.standard tbody tr'), row => {
                    const cells = Array.from(row.cells, cell => cell.textContent.trim());
                    if (cells.length !== 5) throw new Error('Unexpected TISS exam event structure');
                    return {
                        date: date(cells[0]),
                        begin: time(cells[1]),
                        end: time(cells[2]),
                        location: cells[3],
                        comment: cells[4]
                    };
                });

                return {
                    id,
                    name: name.textContent.trim(),
                    dates: headerDates.map(date),
                    lecture_semester: field('examSemesterCode'),
                    mode: field('examMode'),
                    participants: counts[0] === undefined ? null : Number(counts[0]),
                    max_participants: counts[1] === undefined ? null : Number(counts[1]),
                    waiting_list: waiting === null ? null : Number(waiting),
                    application_begin: date(field('appBeginn')),
                    application_end: date(field('appEnd')),
                    deregistration_end: date(field('deregEnd')),
                    registration_type: field('appMode'),
                    registration_confirmation: field('grpAuto'),
                    events
                };
            });"#,
            vec![semester.into(), course.into()],
        )
        .await?
        .convert()
}

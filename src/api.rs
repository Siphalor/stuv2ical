use chrono::{DateTime, Timelike, Utc};
use getset::{CopyGetters, Getters};
use serde::Deserialize;

mod internal {
    use chrono::{DateTime, Utc};
    use getset::Getters;
    use serde::Deserialize;

    #[derive(Deserialize, Getters)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Lecture {
        pub id: u32,
        pub date: DateTime<Utc>,
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
        pub name: String,
        pub lecturer: String,
        pub rooms: Vec<String>,
    }
}

#[derive(Deserialize, Getters, CopyGetters)]
#[serde(from = "internal::Lecture")]
pub struct Lecture {
    #[getset(get_copy = "pub")]
    id: u32,
    #[getset(get = "pub")]
    date: DateTime<Utc>,
    #[getset(get = "pub")]
    start_time: DateTime<Utc>,
    #[getset(get = "pub")]
    end_time: DateTime<Utc>,
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    lecturers: Vec<String>,
    #[getset(get = "pub")]
    rooms: Vec<String>,
    #[getset(get_copy = "pub")]
    online: bool,
    #[getset(get = "pub")]
    event_type: EventType,
}

pub enum EventType {
    Lecture,
    Exam,
    Holiday,
}

impl From<internal::Lecture> for Lecture {
    fn from(base: internal::Lecture) -> Self {
        // LECTURERS
        let mut lecturers_text: &str = &base.lecturer;
        let mut lecturers = Vec::new();
        while !lecturers_text.is_empty() {
            if let Some(first) = lecturers_text.find(',') {
                let after_comma = &lecturers_text[first + 1..];
                if let Some(pos) = after_comma.find(',') {
                    lecturers.push(String::from((&lecturers_text[..first + 1 + pos]).trim()));
                    lecturers_text = &after_comma[pos + 1..];
                } else {
                    lecturers.push(String::from(lecturers_text.trim()));
                    break;
                }
            } else {
                lecturers.push(String::from(lecturers_text.trim()));
                break;
            }
        }

        // NAME, ROOMS, ONLINE
        let mut name = base.name;
        let mut rooms = base.rooms;
        let mut online = false;
        if rooms.is_empty() {
            const ROOM_DELIMITER: &str = " - Raum: ";
            if let Some(index) = name.rfind(ROOM_DELIMITER) {
                let room = name.split_off(index);
                rooms = vec![room[ROOM_DELIMITER.len()..].to_string()];
            }
        } else {
            online = rooms
                .iter()
                .any(|room| room.to_lowercase().contains("online"));
        }

        // EVENT TYPE
        let mut event_type = EventType::Lecture;
        {
            if base.end_time.signed_duration_since(base.start_time) == chrono::Duration::hours(10)
                && rooms.is_empty()
            {
                event_type = EventType::Holiday;
            }
            if let Some(name_begin) = name.split(" ").next() {
                if name_begin.ends_with("klausur")
                    || name_begin == "prüfung"
                    || name_begin == "prüfungswahl"
                {
                    event_type = EventType::Exam;
                }
            }
        }

        Lecture {
            id: base.id,
            name,
            date: base.date,
            start_time: base.start_time,
            end_time: base.end_time,
            rooms,
            lecturers,
            online,
            event_type,
        }
    }
}

pub async fn get_courses(
    client: &awc::Client,
    base_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(client
        .get(format!("{}/rapla/courses", base_url))
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_lectures(
    client: &awc::Client,
    base_url: &str,
    course: &str,
    archived: bool,
) -> Result<Vec<Lecture>, Box<dyn std::error::Error>> {
    let request = if archived {
        client.get(format!(
            "{}/rapla/lectures/{}?archived=true",
            base_url, course
        ))
    } else {
        client.get(format!("{}/rapla/lectures/{}", base_url, course))
    };
    Ok(request.send().await?.json().await?)
}

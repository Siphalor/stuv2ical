use chrono::{DateTime, Utc};
use getset::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Lecture {
    #[getset(get = "pub")]
    id: u32,
    #[getset(get = "pub")]
    date: DateTime<Utc>,
    #[getset(get = "pub")]
    start_time: DateTime<Utc>,
    #[getset(get = "pub")]
    end_time: DateTime<Utc>,
    #[getset(get = "pub")]
    name: String,
    lecturer: String,
    #[getset(get = "pub")]
    rooms: Vec<String>,
}

impl Lecture {
    pub fn lecturers(&self) -> Vec<String> {
        let mut base_string = self.lecturer.as_str();
        let mut lecturers = Vec::new();
        while !base_string.is_empty() {
            if let Some(first) = base_string.find(',') {
                let after_comma = &base_string[first+1..];
                if let Some(pos) = after_comma.find(',') {
                    lecturers.push(String::from((&base_string[..first+1+pos]).trim()));
                    base_string = &after_comma[pos+1..];
                } else {
                    lecturers.push(String::from(base_string.trim()));
                    break;
                }
            } else {
                lecturers.push(String::from(base_string.trim()));
                break;
            }
        }
        return lecturers;
    }

    pub fn is_exam(&self) -> bool {
        let name_lower = self.name.to_lowercase();
        name_lower.starts_with("klausur ") || name_lower.starts_with("prüfung ") || name_lower.starts_with("prüfungswahl ")
    }

    pub fn is_online(&self) -> bool {
        self.rooms.iter().any(|loc| loc.to_lowercase().contains("online"))
    }
}

pub async fn get_courses(client: &awc::Client, base_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(
        client.get(format!("{}/rapla/courses", base_url))
            .send()
            .await?
            .json()
            .await?
    )
}

pub async fn get_lectures(client: &awc::Client, base_url: &str, course: &str, archived: bool) -> Result<Vec<Lecture>, Box<dyn std::error::Error>> {
    let request = if archived {
        client.get(format!("{}/rapla/lectures/{}?archived=true", base_url, course))
    } else {
        client.get(format!("{}/rapla/lectures/{}", base_url, course))
    };
    Ok(request.send().await?.json().await?)
}

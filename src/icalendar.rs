use chrono::Utc;
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::api::Lecture;

const ICALENDAR_DATE_TIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";

pub async fn write_icalendar(file: &mut File, lectures: Vec<Lecture>)
                             -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = BufWriter::new(file);
    write_short_line(&mut writer, "BEGIN:VCALENDAR").await?;
    write_short_line(&mut writer, "VERSION:2.0").await?;
    write_short_line(&mut writer, "PRODID:-//Siphalor//StuV2iCal//DE").await?;
    write_short_line(&mut writer, format!("X-STUV2ICAL-CREATION:{}", Utc::now().format("%d.%m.%Y %H:%M")).as_str()).await?;

    for lecture in &lectures {
        write_lecture(&mut writer, lecture).await?;
    }

    write_short_line(&mut writer, "END:VCALENDAR").await?;

    writer.flush().await?;
    Ok(())
}

async fn write_lecture<W: AsyncWrite + std::marker::Unpin>(writer: &mut W, lecture: &Lecture)
                                                           -> Result<(), Box<dyn std::error::Error>> {
    write_short_line(writer, "BEGIN:VEVENT").await?;
    write_short_line(writer, format!("UID:{}@mosbach.dhbw.de", lecture.id()).as_str()).await?;
    write_short_line(writer, format!("DTSTAMP:{}", Utc::now().format(ICALENDAR_DATE_TIME_FORMAT)).as_str()).await?;
    write_short_line(writer, format!("DTSTART:{}", lecture.start_time().format(ICALENDAR_DATE_TIME_FORMAT)).as_str()).await?;
    write_short_line(writer, format!("DTEND:{}", lecture.end_time().format(ICALENDAR_DATE_TIME_FORMAT)).as_str()).await?;
    write_field(writer, "SUMMARY", lecture.name()).await?;
    write_field(writer, "LOCATION", &lecture.rooms().join(", ")).await?;
    write_field(writer, "DESCRIPTION", format!("Dozent:innen: {}", lecture.lecturers().join(", "))).await?;

    for lecturer in lecture.lecturers() {
        write_line(writer, format!("ATTENDEE;CN=\"{}\":noreply@mosbach.dhbw.de", lecturer).as_str()).await?;
    }
    write_short_line(writer, "END:VEVENT").await?;
    Ok(())
}

/// Safely writes a field with necessary escaping
async fn write_field<W: AsyncWrite + std::marker::Unpin, K, V>(writer: &mut W, key: K, value: V)
                                                               -> Result<(), Box<dyn std::error::Error>>
    where K: Into<String>, V: Into<String> {
    let key = key.into();
    let value = value.into().replace(",", "\\,");
    let line = format!("{}:{}", &key, &value);
    write_line(writer, line.as_str()).await?;
    Ok(())
}

/// Writes a short line with should not exceed the maximum line length
async fn write_short_line<W: AsyncWrite + std::marker::Unpin>(writer: &mut W, line: &str)
                                                              -> Result<(), Box<dyn std::error::Error>> {
    writer.write(format!("{}\r\n", line).as_bytes()).await?;
    Ok(())
}

/// Writes a variable length line with the correct splitting of long lines.
async fn write_line<W: AsyncWrite + std::marker::Unpin>(writer: &mut W, line: &str)
                                                        -> Result<(), Box<dyn std::error::Error>> {
    let mut line_rest = line;

    let mut first = true;
    let mut current_line_length = 0;
    while !line_rest.is_empty() {
        for line_char in line_rest.chars() {
            current_line_length += line_char.len_utf8();
            if current_line_length > 72 {
                current_line_length -= line_char.len_utf8();
                break;
            }
        }

        let parts = line_rest.split_at(current_line_length);
        line_rest = parts.1;
        current_line_length = 0;

        if first {
            writer.write(format!("{}\r\n", parts.0).as_bytes()).await?;
            first = false;
        } else {
            writer.write(format!(" {}\r\n", parts.0).as_bytes()).await?;
        }
    }
    Ok(())
}

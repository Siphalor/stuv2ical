mod api;
mod icalendar;

use std::fs::create_dir_all;
use std::path::Path;
use clap::{App, Arg};
use tokio::fs::File;
use crate::api::{get_courses, get_lectures};
use crate::icalendar::write_icalendar;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let arg_matcher = App::new("StuV2iCal")
        .version("0.1")
        .about("An unofficial program that uses the StuV API (api.stuv.app) to generate iCalendar files.")
        .author("Siphalor <info@siphalor.de>")
        .arg(Arg::new("output-directory")
            .short('o')
            .takes_value(true)
            .about("The output directory for the iCalendar files"))
        .arg(Arg::new("api-base-url")
            .short('a')
            .takes_value(true)
            .about("The base url of the Stuv API"));

    let matches = arg_matcher.get_matches();
    let api_base_url = matches.value_of("api-base-url")
        .map(String::from)
        .or_else(|| dotenv::var("API_BASE_URL").ok())
        .expect("No API base url specified!");
    let output_dir = Path::new(matches.value_of("output-directory").unwrap_or(""));
    create_dir_all(output_dir).expect("Failed to create output directory!");

    let client = create_client();
    println!("Starting request for courses");
    let courses = get_courses(&client, api_base_url.as_str())
        .await
        .expect("Failed to get courses from API");
    println!("Got {} courses", courses.len());

    for course in courses {
        let result = process_course(&client, &output_dir, api_base_url.as_str(), course.as_str())
            .await;
        if let Err(err) = result {
            println!("Failed to write calendar file for course {}: {:?}", course, err);
        }
    }

    println!("Done.");
}

fn create_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .user_agent("StuV2iCal")
        .build().expect("Failed to create client!")
}

async fn process_course(client: &reqwest::Client, output_directory: &Path, base_url: &str, course: &str)
    -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading lecture data for course {}", course);
    let lectures = get_lectures(client, base_url, course).await?;

    println!("Writing calendar file for course {} with {} lectures", course, lectures.len());
    let mut file = File::create(output_directory.join(format!("{}.ics", course))).await?;
    write_icalendar(&mut file, lectures).await?;

    Ok(())
}

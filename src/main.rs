mod api;
mod icalendar;

use std::fs::create_dir_all;
use std::path::Path;
use clap::Parser;
use tokio::fs::File;
use crate::api::{get_courses, get_lectures};
use crate::icalendar::write_icalendar;

#[derive(Parser)]
#[clap(
    version = "0.2",
    author = "Siphalor <info@siphalor.de>",
    rename_all = "kebab",
    about = "An unofficial program that uses the StuV API to generate iCalendar files.",
)]
struct Opts {
    /// The output directory for the iCalendar files
    #[clap(short, long, default_value = "")]
    output_directory: String,

    /// The base url of the StuV API
    #[clap(short, long, env="API-BASE-URL")]
    api_base_url: Option<String>,

    /// A course to query. Querying for all courses if none is specified.
    #[clap(short, long = "course", multiple_occurrences = true)]
    courses: Vec<String>,

    /// Request and include archived events
    #[clap(long)]
    request_archived: bool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let opts: Opts = Opts::parse();

    let api_base_url = opts.api_base_url
        .or_else(|| dotenv::var("API_BASE_URL").ok())
        .expect("No API base url specified!");
    let output_dir = Path::new(&opts.output_directory);
    create_dir_all(output_dir).expect("Failed to create output directory!");

    let client = create_client();
    println!("Starting request for courses");

    let courses = if !opts.courses.is_empty() {
        opts.courses
    } else {
        get_courses(&client, api_base_url.as_str())
            .await
            .expect("Failed to get courses from API")
    };

    println!("Got {} courses", courses.len());

    for course in courses {
        let result = process_course(&client, &output_dir, api_base_url.as_str(), course.as_str(), opts.request_archived)
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

async fn process_course(client: &reqwest::Client, output_directory: &Path, base_url: &str, course: &str, archived: bool)
    -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading lecture data for course {}", course);
    let lectures = get_lectures(client, base_url, course, archived).await?;

    println!("Writing calendar file for course {} with {} lectures", course, lectures.len());
    let mut file = File::create(output_directory.join(format!("{}.ics", course))).await?;
    write_icalendar(&mut file, lectures).await?;

    Ok(())
}

use crate::moodle_client::*;
use crate::utils;
use crate::moodle_client::error::MoodleClientError;
use crate::moodle_client::models::{MoodleCourseActivity, MoodleCourse};
use std::collections::HashMap;

static SLEEP_BETWEEN_SCRAPES: u64 = 3 * 60 * 1000;
static CHECK_SPEED: u64 = 2000;

pub fn start_scrape()
{
	println!("Hello, Moodle!");

	let mut client = MoodleClient::new().unwrap();
	println!("Scraping for {}",dotenv!("BME_USERNAME"));
	client.login().expect("Couldn't login to moodle !");

	let ld = client.get_loaded_state().expect("Should log in.");
	println!("Logged in as {}, course count {}", ld.username, ld.courses.len());

	let mut all_activities = Vec::new();

	let mut first_run = true;

	loop
	{
		println!("Doing scrape loop!");
		let scrape_res = do_scrape(&mut client, &mut all_activities, first_run);
		match scrape_res
		{
			Ok(_) => {}
			Err(ee) => println!("{:?} error while scraping.", ee)
		}

		println!("Finished scrape loop run.");
		first_run = false;
		utils::blocking_sleep(SLEEP_BETWEEN_SCRAPES);
	}
}

pub fn do_scrape(client: &mut MoodleClient, all_activities: &mut Vec<MoodleCourseActivity>, first_run: bool) -> Result<(), MoodleClientError>
{
	let logged_state = client.get_loaded_state().unwrap();
	for course in logged_state.courses.iter()
	{
		let actv = client.scrape_course_page(course)?;
		for ac in actv
		{
			if !all_activities.iter().any(|el| el.url == ac.url)
			{
				if !first_run {
					handle_new_activity(&ac, course)?;
				}
				all_activities.push(ac);
			}
		}

		utils::blocking_sleep(CHECK_SPEED);
	}

	Ok(())
}

pub fn handle_new_activity(actvity: &MoodleCourseActivity, course: &MoodleCourse) -> Result<(), MoodleClientError>
{
	println!("Found new course activity in course : {} - {}", course.name, actvity.name);

	let disc = reqwest::blocking::Client::new();
	let mut dc_form = HashMap::new();
	dc_form.insert("content", format!("**NEW MOODLE ACTIVITY {} - in:{}**\nLink: {}", actvity.name, course.name, actvity.url));

	disc.post(dotenv!("DC_HOOK"))
		.form(&dc_form)
		.send()
		.map_err(|_| MoodleClientError::RequestError)
		.map(|_| ())
}
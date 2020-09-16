use crate::moodle_client::*;
use crate::utils;
use crate::moodle_client::error::MoodleClientError;
use crate::moodle_client::models::{MoodleCourseActivity, MoodleCourse};
use std::collections::HashMap;
use crate::course_prefs::get_course_prefs;

static SLEEP_BETWEEN_SCRAPES: u64 = 20 * 1000;
static CHECK_SPEED: u64 = 2000;

pub fn start_scrape()
{
	println!("Hello, Moodle!");

	let mut client = MoodleClient::new().unwrap();
	println!("Scraping for {}", dotenv!("BME_USERNAME"));
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
			pick_activity_option(client, &ac,&course);

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

pub fn pick_activity_option(client: &MoodleClient, activity: &MoodleCourseActivity, course: &MoodleCourse)
{
	let prefs = get_course_prefs();
	let ac_id: &str = &activity.get_id();
	if prefs.contains_key(ac_id)
	{
		let pref_order = prefs[ac_id].iter();
		for pref_choice in pref_order
		{
			let res = client.select_option_on_activity(activity, pref_choice.to_owned());
			match res
			{
				Ok(res_v) => {
					println!("Choiceres : {:?} - ac: {}/{} - for {}", res_v, activity.name,course.name, pref_choice);

					if res_v == ChoiceResult::Success
					{
						send_discord_hook(format!("**Sent choice for {} :: {} - in:{}**\nLink: {}", activity.name,pref_choice,course.name, activity.url));
					}

					if res_v != ChoiceResult::Full
					{
						return;
					}
				}
				Err(res_e) =>
				{
					println!("{:?} error on {:?} / {}",res_e,activity,course.name);
				}
			}
		}
	}
}

pub fn handle_new_activity(actvity: &MoodleCourseActivity, course: &MoodleCourse) -> Result<(), MoodleClientError>
{
	println!("Found new course activity in course : {} - {}", course.name, actvity.name);

	send_discord_hook(format!("**NEW MOODLE ACTIVITY {} - in:{}**\nLink: {}", actvity.name, course.name, actvity.url))
}

pub fn send_discord_hook(content:String) -> Result<(),MoodleClientError>
{
	let disc = reqwest::blocking::Client::new();
	let mut dc_form = HashMap::new();
	dc_form.insert("content", content);

	disc.post(dotenv!("DC_HOOK"))
		.form(&dc_form)
		.send()
		.map_err(|_| MoodleClientError::RequestError)
		.map(|_| ())
}
pub mod error;
pub mod models;

use reqwest::blocking::Client;
use self::error::MoodleClientError;
use reqwest::{Url, redirect};
use reqwest::blocking::{Response};
use std::collections::HashMap;
use crate::utils;
use scraper;
use std::fs::{OpenOptions};
use std::io::Write;
use crate::moodle_client::models::{MoodleCourse, MoodleCourseActivity};

pub enum MoodleState
{
	LoggedIn(MoodleLoadedState),
	NotLoggedIn,
}

pub struct MoodleClient
{
	client: Client,
	pub state: MoodleState
}

#[derive(Debug)]
pub struct MoodleLoadedState
{
	pub username: String,
	pub courses: Vec<MoodleCourse>,
}

static SLEEP_BETWEEN_PAGES: u64 = 1200;

impl MoodleClient
{
	pub fn new() -> Result<Self, MoodleClientError>
	{
		let client = reqwest::blocking::Client::builder()
			.user_agent(dotenv!("USER_AGENT"))
			.cookie_store(true)
			.redirect(redirect::Policy::limited(10))
			.build().map_err(|_| MoodleClientError::FailedToCreateClient)?;

		Ok(Self {
			client,
			state: MoodleState::NotLoggedIn
		})
	}

	pub fn get_loaded_state(&self) -> Option<&MoodleLoadedState>
	{
		match &self.state
		{
			MoodleState::NotLoggedIn => None,
			MoodleState::LoggedIn(stat) => Some(stat)
		}
	}

	fn get_element_attr_val_by_selector(dom: &scraper::Html, selector: &str, attr: &str) -> Result<String, MoodleClientError>
	{
		dom.select(&scraper::Selector::parse(selector).unwrap())
			.last()
			.ok_or(MoodleClientError::ElementNotFound)?
			.value()
			.attr(attr)
			.ok_or(MoodleClientError::DataNotFound)
			.map(|val| val.to_string())
	}

	fn get_to_bme_login(&self) -> Result<(), MoodleClientError>
	{
		self.client.get(Url::parse("https://edu.vik.bme.hu/login/index.php").unwrap())
			.send()
			.map_err(|_| MoodleClientError::RequestError)?;

		utils::blocking_sleep(SLEEP_BETWEEN_PAGES);

		self.client.get(Url::parse("https://edu.vik.bme.hu/auth/shibboleth/index.php").unwrap())
			.send()
			.map_err(|_| MoodleClientError::RequestError)?;

		Ok(())
	}

	fn get_dom_from_response(resp: Response) -> Result<scraper::Html, MoodleClientError>
	{
		let body_txt = resp.text().map_err(|_| MoodleClientError::LoadBodyError)?;
		Ok(scraper::Html::parse_document(&body_txt))
	}

	fn do_bme_login_nojs_post(&mut self, login_dom: &scraper::Html) -> Result<(), MoodleClientError>
	{
		let relay_state = Self::get_element_attr_val_by_selector(&login_dom, "input[name=\"RelayState\"]", "value")?;
		let saml_response = Self::get_element_attr_val_by_selector(&login_dom, "input[name=\"SAMLResponse\"]", "value")?;

		let mut login_second_form = HashMap::new();
		login_second_form.insert("RelayState", relay_state);
		login_second_form.insert("SAMLResponse", saml_response);

		let login_second_url = Self::get_element_attr_val_by_selector(&login_dom, "form", "action")?;

		let login_second_resp = self.client.post(Url::parse(&login_second_url).unwrap())
			.form(&login_second_form)
			.send()
			.map_err(|_| MoodleClientError::RequestError)?;

		if !login_second_resp.status().is_success()
		{
			return Err(MoodleClientError::LoginError);
		}

		let login_second_dom = Self::get_dom_from_response(login_second_resp)?;
		let username = Self::get_current_username(&login_second_dom)?;
		let courses = Self::get_courses(&login_second_dom)?;

		self.state = MoodleState::LoggedIn(MoodleLoadedState {
			username,
			courses,
		});

		Ok(())
	}

	pub fn do_bme_login(&mut self) -> Result<(), MoodleClientError>
	{
		let mut login_bme_form = HashMap::new();
		login_bme_form.insert("j_username", dotenv!("BME_USERNAME"));
		login_bme_form.insert("j_password", dotenv!("BME_PASSWORD"));

		let login_resp = self.client.post(Url::parse("https://login.bme.hu/idp/Authn/UserPassword").unwrap())
			.form(&login_bme_form)
			.send()
			.map_err(|_| MoodleClientError::RequestError)?;

		if !login_resp.status().is_success()
		{
			return Err(MoodleClientError::LoginError);
		}

		let login_dom = Self::get_dom_from_response(login_resp)?;

		self.do_bme_login_nojs_post(&login_dom)
	}

	pub fn login(&mut self) -> Result<(), MoodleClientError>
	{
		self.get_to_bme_login()?;
		utils::blocking_sleep(SLEEP_BETWEEN_PAGES);
		self.do_bme_login()?;
		utils::blocking_sleep(SLEEP_BETWEEN_PAGES);

		Ok(())
	}

	fn dump_html_for_test(html: String, name: &str)
	{
		let mut file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(format!("{}.html", name)).unwrap();
		file.write_all(html.as_bytes()).expect("Couldn't write to dummy file !");
	}

	fn get_current_username(dom: &scraper::Html) -> Result<String, MoodleClientError>
	{
		for user_spans in dom.select(&scraper::Selector::parse("span.usertext.mr-1").unwrap())
		{
			return Ok(user_spans.inner_html());
		}

		Err(MoodleClientError::DataNotFound)
	}

	fn get_courses(dom: &scraper::Html) -> Result<Vec<MoodleCourse>, MoodleClientError>
	{
		let course_selector = scraper::Selector::parse("h3.coursename>.aalink").unwrap();
		let course_links = dom.select(&course_selector);
		let mut res = Vec::new();

		for course_link in course_links
		{
			let name: String = course_link.text().collect::<Vec<_>>().join("");
			let href = course_link.value().attr("href").ok_or(MoodleClientError::DataNotFound)?;
			res.push(MoodleCourse {
				name,
				url: href.to_string(),
			});
		}

		Ok(res)
	}

	pub fn scrape_course_page(&self,course:&MoodleCourse) -> Result<Vec<MoodleCourseActivity>,MoodleClientError>
	{
		let resp = self.client.get(&course.url).send().map_err(|_| MoodleClientError::RequestError)?;
		let dom = Self::get_dom_from_response(resp)?;

		let choice_links_selector = scraper::Selector::parse(".activityinstance>.aalink[href*=\"/mod/choice\"]").unwrap();
		let choice_instancenames_selector = scraper::Selector::parse(".activityinstance>.aalink[href*=\"/mod/choice\"]>.instancename").unwrap();

		let choice_links = dom.select(&choice_links_selector).collect::<Vec<_>>();
		let choice_names = dom.select(&choice_instancenames_selector).collect::<Vec<_>>();

		if choice_names.len() != choice_links.len()
		{
			return Err(MoodleClientError::DataNotFound);
		}

		let mut actv = Vec::new();

		for link_index in 0..choice_links.len()
		{
			let link = choice_links[link_index].value().attr("href").ok_or(MoodleClientError::DataNotFound)?.to_string();
			let name:String = choice_names[link_index].text().collect::<Vec<_>>().join("");
			actv.push(MoodleCourseActivity{
				name,
				url:link
			})
		}

		Ok(actv)
	}
}

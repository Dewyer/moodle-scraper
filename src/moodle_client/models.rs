
#[derive(Debug,Clone)]
pub struct MoodleCourse
{
	pub name:String,
	pub url: String
}

#[derive(Debug,Clone)]
pub struct MoodleCourseActivity
{
	pub name:String,
	pub url:String
}

impl MoodleCourseActivity
{
	pub fn get_id(&self) -> String
	{
		self.url.split("id=").last().unwrap().trim().to_string()
	}
}
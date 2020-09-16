use std::collections::HashMap;

pub fn get_course_prefs() -> HashMap<String, Vec<i32>>
{
	let mut hm = HashMap::new();
	hm.insert("23045".to_string(), vec![1,2,3]);
	hm.insert("23200".to_string(),vec![1,2,3]);

	hm
}
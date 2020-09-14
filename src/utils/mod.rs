use std::{thread, time};

pub fn blocking_sleep(milliseconds:u64)
{
	let ten_millis = time::Duration::from_millis(milliseconds);
	thread::sleep(ten_millis);
}
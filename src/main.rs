pub mod moodle_client;
pub mod utils;
pub mod scraper;

#[macro_use]
extern crate dotenv_codegen;

fn main()
{
    scraper::start_scrape();
}

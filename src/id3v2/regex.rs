extern crate regex;
use self::regex::Regex;

pub fn get_track_number(input: &str) -> (String, String) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d+)(/\d+)?$").unwrap();
    }

    match RE.captures(input) {
        None => ("".to_string(), "".to_string()),
        Some(c) => (
            match c.get(1) {
                None => "".to_string(),
                Some(s) => s.as_str().trim_start_matches('0').to_string(),
            },
            match c.get(2) {
                None => "".to_string(),
                Some(s) => (&s.as_str()[1..]).trim_start_matches('0').to_string(),
            },
        ),
    }
}

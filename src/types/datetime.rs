#[derive(PartialEq, Debug, Clone, Default)]
pub struct DateTime {
    pub year: Option<i64>,
    pub month: Option<u8>,
    pub day: Option<u8>, 
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    // utc only
}

extern crate regex;
use self::regex::Regex;


macro_rules! parse {
    ($c:ident, $i:ident, $t:ty) => {{
        match $c.name(stringify!($i)) {
            None => None,
            Some(m) => {
                match m.as_str().parse::<$t>() {
                    Ok(i) => Some(i),
                    Err(_) => None,
                }},
        }
    }}
}

impl DateTime {
    pub fn from_iso_8601(string: &str) -> Option<DateTime> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                ^
                (?P<year>\d{4})
                  (?:-(?P<month>\d{2})
                    (?:-(?P<day>\d{2})
                      (?:T(?P<hour>\d{2})
                        (?::(?P<minute>\d{2})
                          (?::(?P<second>\d{2})Z?
                        )?
                      )?
                    )?
                  )?
                )?
            ").unwrap();
        }
        if let Some(captures) = RE.captures(string) {
            Some(DateTime {
                year: parse!(captures, year, i64),
                month: parse!(captures, month, u8),
                day: parse!(captures, day, u8),
                hour: parse!(captures, hour, u8),
                minute: parse!(captures, minute, u8),
                second: parse!(captures, second, u8),
            })
        } else {
            None
        }
    }
    pub fn to_iso_8601(&self) -> String {
        let mut s = String::with_capacity(19);
        if let Some(x) = self.year {
            s.push_str(format!("{:04}", x).as_str());
            if let Some(x) = self.month {
                s.push_str(format!("-{:02}", x).as_str());
                if let Some(x) = self.day {
                    s.push_str(format!("-{:02}", x).as_str());
                    if let Some(x) = self.hour {
                        s.push_str(format!("T{:02}", x).as_str());
                        if let Some(x) = self.minute {
                            s.push_str(format!(":{:02}", x).as_str());
                            if let Some(x) = self.second {
                                s.push_str(format!(":{:02}", x).as_str());
                            }
                        }
                    }
                }
            }
        }
        s
    }
}

use std::fmt;
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_iso_8601())
    }
}

#[test]
fn test() {
    assert_eq!(DateTime::from_iso_8601("2000").unwrap(), 
               DateTime { year: Some(2000), ..Default::default() });
    assert_eq!(DateTime::from_iso_8601("2000-01").unwrap(), 
               DateTime { 
                   year: Some(2000), 
                   month: Some(1),
                   ..Default::default() 
               });
    assert_eq!(DateTime::from_iso_8601("2000-01-01").unwrap(), 
               DateTime { 
                   year: Some(2000), 
                   month: Some(1),
                   day: Some(1),
                   ..Default::default() 
               });
    assert_eq!(DateTime::from_iso_8601("2000-01-01T00").unwrap(), 
               DateTime { 
                   year: Some(2000), 
                   month: Some(1),
                   day: Some(1),
                   hour: Some(0),
                   ..Default::default() 
               });
    assert_eq!(DateTime::from_iso_8601("2000-01-01T00:00").unwrap(), 
               DateTime { 
                   year: Some(2000), 
                   month: Some(1),
                   day: Some(1),
                   hour: Some(0),
                   minute: Some(0),
                   ..Default::default() 
               });
    assert_eq!(DateTime::from_iso_8601("2000-01-01T00:00:00").unwrap(), 
               DateTime { 
                   year: Some(2000), 
                   month: Some(1),
                   day: Some(1),
                   hour: Some(0),
                   minute: Some(0),
                   second: Some(0),
                   ..Default::default() 
               });

    let s = "2000-01-01T00:00:00";
    assert_eq!(DateTime::from_iso_8601(s).unwrap().to_iso_8601(), s.to_string());
}


#[cfg(feature = "chrono")]
mod chrono {
    use DateTime;
    extern crate chrono;
    use self::chrono::DateTime as ChronoDateTime;
    use self::chrono::{Utc, TimeZone, Datelike, Timelike};
    
    impl<Tz: TimeZone> From<ChronoDateTime<Tz>> for DateTime {
        fn from(cdt: ChronoDateTime<Tz>) -> DateTime {
            let cdt = cdt.with_timezone(&Utc);
            DateTime {
                year: Some(cdt.year() as i64),
                month: Some(cdt.month() as u8),
                day: Some(cdt.day() as u8),
                hour: Some(cdt.hour() as u8),
                minute: Some(cdt.minute() as u8),
                second: Some(cdt.second() as u8),
            }
        }
    }

    use self::chrono::{NaiveDateTime, NaiveDate};

    impl From<DateTime> for Option<NaiveDateTime> {
        fn from(dt: DateTime) -> Option<NaiveDateTime> {
            let ndt = NaiveDate::from_ymd_opt(dt.year.unwrap_or(0) as i32, 
                                              dt.month.unwrap_or(1) as u32, 
                                              dt.day.unwrap_or(1) as u32);
            if ndt.is_none() {
                None
            } else {
                ndt.unwrap().and_hms_opt(dt.hour.unwrap_or(0) as u32, 
                                         dt.minute.unwrap_or(0) as u32, 
                                         dt.second.unwrap_or(0) as u32)
            }
        }
    }
    impl From<DateTime> for Option<ChronoDateTime<Utc>> {
        fn from(dt: DateTime) -> Option<ChronoDateTime<Utc>> {
            let ndt: NaiveDateTime = 
                match dt.into() {
                    Some(x) => x,
                    None => return None,
                };
            Some(ChronoDateTime::<Utc>::from_utc(ndt, Utc))
        }
    }

    #[test]
    fn from_chrono() {
        let s = "2000-01-01T00:00:00-00:00";
        let cdt = self::chrono::DateTime::parse_from_rfc3339(s).unwrap();
        let dt = DateTime::from_iso_8601(s).unwrap();
        assert_eq!(dt, cdt.into());
    }
    #[test]
    fn into_chrono() {
        let s = "2000-01-01T00:00:00-00:00";
        let dt = DateTime::from_iso_8601(s).unwrap();
        let cdt = ChronoDateTime::parse_from_rfc3339(s).unwrap();
        assert_eq!(cdt, (Option::from(dt)).unwrap());
    }
}

#![allow(non_camel_case_types)]

use chrono::{DateTime, Utc, Local, NaiveDate, NaiveTime, Datelike, Timelike, Weekday};
use vbox_core::VBoxResult;

pub const RT_DATE_MIN_YEAR: i32 = 1900;
pub const RT_DATE_MAX_YEAR: i32 = 9999;
pub const RT_TIME_MIN_HOUR: i32 = 0;
pub const RT_TIME_MAX_HOUR: i32 = 23;
pub const RT_TIME_MIN_MINUTE: i32 = 0;
pub const RT_TIME_MAX_MINUTE: i32 = 59;
pub const RT_TIME_MIN_SECOND: i32 = 0;
pub const RT_TIME_MAX_SECOND: i32 = 59;

#[repr(C)]
pub struct RTDATE {
    pub i32Year: i32,
    pub i32Month: i32,
    pub i32Day: i32,
    pub i32WeekYear: i32,
    pub enmWeekDay: Weekday,
}

#[repr(C)]
pub struct RTTIME {
    pub i32Hour: i32,
    pub i32Min: i32,
    pub i32Sec: i32,
    pub i32NanoTS: i64,
}

#[repr(C)]
pub struct RTDATETIME {
    pub i32Year: i32,
    pub i32Month: i32,
    pub i32Day: i32,
    pub i32Hour: i32,
    pub i32Min: i32,
    pub i32Sec: i32,
    pub i32NanoTS: i64,
    pub enmWeekDay: Weekday,
}

#[repr(C)]
pub struct RTTIMEINTERVAL {
    pub cHour: i32,
    pub cMin: i32,
    pub cSec: i32,
    pub cNanoTS: i64,
}

#[repr(C)]
pub struct RTTIMESPEC {
    pub i64Year: i64,
    pub i64Month: i64,
    pub i64Day: i64,
    pub i64Hour: i64,
    pub i64Minute: i64,
    pub i64Second: i64,
    pub i64Nanosecond: i64,
}

impl RTDATE {
    pub fn new(year: i32, month: u32, day: u32) -> VBoxResult<Self> {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| vbox_core::VBoxError::BadParam)?;
        Ok(Self {
            i32Year: date.year(),
            i32Month: date.month() as i32,
            i32Day: date.day(),
            i32WeekYear: date.iso_week().year(),
            enmWeekDay: date.weekday(),
        })
    }

    pub fn today() -> Self {
        let today = Utc::today().naive_utc();
        Self {
            i32Year: today.year(),
            i32Month: today.month() as i32,
            i32Day: today.day(),
            i32WeekYear: today.iso_week().year(),
            enmWeekDay: today.weekday(),
        }
    }

    pub fn is_leap_year(year: i32) -> bool {
        NaiveDate::from_ymd_opt(year, 1, 1).map_or(false, |d| {
            let _ = d;
            (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
        })
    }

    pub fn days_in_month(year: i32, month: u32) -> u32 {
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        let days = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .and_then(|d| d.pred_opt())
            .map(|d| d.day())
            .unwrap_or(28);
        days
    }

    pub fn day_of_year(&self) -> u32 {
        NaiveDate::from_ymd_opt(self.i32Year, self.i32Month as u32, self.i32Day as u32)
            .map(|d| d.ordinal() as u32)
            .unwrap_or(0)
    }

    pub fn day_of_week(&self) -> Weekday {
        self.enmWeekDay
    }

    pub fn day_of_week_index(&self) -> u32 {
        self.enmWeekDay.num_days_from_monday()
    }

    pub fn week_number(&self) -> u32 {
        self.i32WeekYear as u32
    }

    pub fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.i32Year, self.i32Month, self.i32Day)
    }
}

impl Default for RTDATE {
    fn default() -> Self {
        Self::today()
    }
}

impl RTTIME {
    pub fn new(hour: u32, minute: u32, second: u32) -> VBoxResult<Self> {
        if hour > 23 || minute > 59 || second > 59 {
            return Err(vbox_core::VBoxError::BadParam);
        }
        Ok(Self {
            i32Hour: hour as i32,
            i32Min: minute as i32,
            i32Sec: second as i32,
            i32NanoTS: 0,
        })
    }

    pub fn now() -> Self {
        let now = Local::now();
        Self {
            i32Hour: now.hour() as i32,
            i32Min: now.minute() as i32,
            i32Sec: now.second() as i32,
            i32NanoTS: now.timestamp_subsec_nanos() as i64,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.i32Hour, self.i32Min, self.i32Sec)
    }
}

impl Default for RTTIME {
    fn default() -> Self {
        Self::now()
    }
}

impl RTDATETIME {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> VBoxResult<Self> {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| vbox_core::VBoxError::BadParam)?;
        let time = NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32)
            .ok_or_else(|| vbox_core::VBoxError::BadParam)?;
        let datetime = date.and_time(time);
        let dt = DateTime::<Utc>::from_utc(datetime, Utc);
        Ok(Self {
            i32Year: dt.year(),
            i32Month: dt.month() as i32,
            i32Day: dt.day(),
            i32Hour: dt.hour() as i32,
            i32Min: dt.minute() as i32,
            i32Sec: dt.second() as i32,
            i32NanoTS: dt.timestamp_subsec_nanos() as i64,
            enmWeekDay: dt.weekday(),
        })
    }

    pub fn now() -> Self {
        let now = Local::now();
        Self {
            i32Year: now.year(),
            i32Month: now.month() as i32,
            i32Day: now.day(),
            i32Hour: now.hour() as i32,
            i32Min: now.minute() as i32,
            i32Sec: now.second() as i32,
            i32NanoTS: now.timestamp_subsec_nanos() as i64,
            enmWeekDay: now.weekday(),
        }
    }

    pub fn add(&self, interval: &RTTIMEINTERVAL) -> Self {
        let dt = NaiveDate::from_ymd_opt(self.i32Year, self.i32Month as u32, self.i32Day as u32)
            .and_then(|d| d.and_time_opt(NaiveTime::from_hms_opt(self.i32Hour as u32, self.i32Min as u32, self.i32Sec as u32).unwrap()));
        let new_dt = dt.map(|d| {
            d.checked_add_signed(chrono::Duration::hours(interval.cHour as i64)
                + chrono::Duration::minutes(interval.cMin as i64)
                + chrono::Duration::seconds(interval.cSec as i64)
                + chrono::Duration::nanoseconds(interval.cNanoTS))
        }).flatten();
        match new_dt {
            Some(nd) => Self {
                i32Year: nd.year(),
                i32Month: nd.month() as i32,
                i32Day: nd.day(),
                i32Hour: nd.hour() as i32,
                i32Min: nd.minute() as i32,
                i32Sec: nd.second() as i32,
                i32NanoTS: nd.timestamp_subsec_nanos() as i64,
                enmWeekDay: nd.weekday(),
            },
            None => self.clone(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                self.i32Year, self.i32Month, self.i32Day,
                self.i32Hour, self.i32Min, self.i32Sec)
    }
}

impl Default for RTDATETIME {
    fn default() -> Self {
        Self::now()
    }
}

impl RTTIMEINTERVAL {
    pub fn new(hour: i32, minute: i32, second: i32, nano: i64) -> Self {
        Self {
            cHour: hour,
            cMin: minute,
            cSec: second,
            cNanoTS: nano,
        }
    }

    pub fn zero() -> Self {
        Self { cHour: 0, cMin: 0, cSec: 0, cNanoTS: 0 }
    }

    pub fn total_milliseconds(&self) -> i64 {
        (self.cHour * 3600 + self.cMin * 60 + self.cSec) as i64 * 1000
            + self.cNanoTS / 1_000_000
    }
}

pub fn rt_date_from_epoch_seconds(epoch: i64) -> RTDATETIME {
    let dt = DateTime::<Utc>::from_timestamp(epoch, 0).unwrap_or_default();
    RTDATETIME {
        i32Year: dt.year(),
        i32Month: dt.month() as i32,
        i32Day: dt.day(),
        i32Hour: dt.hour() as i32,
        i32Min: dt.minute() as i32,
        i32Sec: dt.second() as i32,
        i32NanoTS: dt.timestamp_subsec_nanos() as i64,
        enmWeekDay: dt.weekday(),
    }
}

pub fn rt_date_to_epoch_seconds(dt: &RTDATETIME) -> i64 {
    let naive = NaiveDate::from_ymd_opt(dt.i32Year, dt.i32Month as u32, dt.i32Day as u32)
        .and_then(|d| d.and_time_opt(NaiveTime::from_hms_opt(dt.i32Hour as u32, dt.i32Min as u32, dt.i32Sec as u32).unwrap()));
    match naive {
        Some(nd) => DateTime::<Utc>::from_utc(nd, Utc).timestamp(),
        None => 0,
    }
}

pub fn rt_date_difference(a: &RTDATETIME, b: &RTDATETIME) -> i64 {
    a.i32Year * 365 + a.i32Month * 30 + a.i32Day - (b.i32Year * 365 + b.i32Month * 30 + b.i32Day)
}

pub fn rt_date_is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn rt_date_days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

use chrono::prelude::*;
use chrono::{Duration, NaiveDate};
pub struct DayIter {
    date: chrono::NaiveDate,
}

impl DayIter {
    pub fn new(date: chrono::NaiveDate) -> Self {
        DayIter { date }
    }
    pub fn clone(&mut self) -> &chrono::NaiveDate {
        &self.date
    }
    pub fn next_day(&mut self) -> &chrono::NaiveDate {
        self.date = self.date.succ_opt().unwrap();
        &self.date
    }

    pub fn previous_day(&mut self) -> &chrono::NaiveDate {
        self.date = self.date.pred_opt().unwrap();
        &self.date
    }

    pub fn next_month(&mut self) -> &chrono::NaiveDate {
        let start = self.date;

        while self.next_day().month() == start.month() {
            continue;
        }
        &self.date
    }

    pub fn previous_month(&mut self) -> &chrono::NaiveDate {
        let start = self.date;

        while self.previous_day().month() == start.month() {
            continue;
        }
        &self.date
    }

    pub fn last_day_month(&mut self) -> &chrono::NaiveDate {
        self.date = self.next_month().pred_opt().unwrap();
        &self.date
    }

    pub fn first_day_month(&mut self) -> &chrono::NaiveDate {
        self.date = self.previous_month().succ_opt().unwrap();
        &self.date
    }

    pub fn day_module(&mut self) -> u32 {
        let day = self.date.day();
        if &day % 7 != 0 {
            // uncomment for regular count
            &day / 7 + 1
        } else {
            &day / 7
        }
    }

    pub fn leap_year(&mut self) -> bool {
        let from_ymd_opt = NaiveDate::from_ymd_opt;
        from_ymd_opt(self.date.year(), 2, 29).is_some()
    }
    pub fn previous_module(&mut self) -> u32 {
        self.date = if self.date.clone().day() > 7 {
            self.date - Duration::days(7)
        } else {
            self.date - Duration::days(i64::from(self.date.clone().day()) + 1)
        };
        println!("{:#?}", self.date);
        self.day_module()
    }

    pub fn weeks(&mut self) -> Vec<u32> {
        let mut weeks = vec![self.day_module(), self.previous_module()];
        println!("{:?}", weeks);
        weeks.reverse();
        weeks
    }

    pub fn first_day_module(&mut self) -> NaiveDate {
        let day_module = self.day_module();
        if day_module == 1 {
            NaiveDate::from_ymd_opt(self.date.year(), self.date.month(), 1).unwrap()
        } else if day_module * 7 > self.last_day_month().day() {
            let day = (day_module - 1) * 7 + 1;
            // NaiveDate::from_ymd(year, month, current_date.day())
            NaiveDate::from_ymd_opt(self.date.year(), self.date.month(), day).unwrap()
        } else {
            NaiveDate::from_ymd_opt(
                self.date.year(),
                self.date.month(),
                (day_module - 1) * 7 + 1,
            )
            .unwrap()
        }
    }

    pub fn last_day_module(&mut self) -> NaiveDate {
        let day_module = self.day_module();
        let last_day_month = *self.last_day_month();
        if day_module * 7 < last_day_month.day() {
            NaiveDate::from_ymd_opt(
                self.date.clone().year(),
                self.date.clone().month(),
                (day_module) * 7,
            )
            .unwrap()
        } else {
            NaiveDate::from_ymd_opt(
                self.date.clone().year(),
                self.date.clone().month(),
                last_day_month.day(),
            )
            .unwrap()
        }
    }
}
pub fn first_day_week(
    last_day_month: &u32,
    year: &i32,
    month: &u32,
    day_module: &u32,
) -> NaiveDate {
    if day_module == &1_u32 {
        NaiveDate::from_ymd_opt(*year, *month, 1).unwrap()
    } else if day_module * 7 > *last_day_month {
        let day = (day_module - 1) * 7 + 1;
        NaiveDate::from_ymd_opt(*year, *month, day).unwrap()
    } else {
        NaiveDate::from_ymd_opt(*year, *month, (day_module - 1) * 7 + 1).unwrap()
    }
}

pub fn last_day_week(last_day_month: &u32, year: &i32, month: &u32, day_module: &u32) -> NaiveDate {
    if day_module * 7 < *last_day_month {
        NaiveDate::from_ymd_opt(*year, *month, (day_module) * 7).unwrap()
    } else {
        NaiveDate::from_ymd_opt(*year, *month, *last_day_month).unwrap()
    }
}

use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct TurnAroundTime {
    pub id: i32,
    pub turnaroundable_id: Option<i32>,
    pub turnaroundable_type: Option<String>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub display_units: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    // pub origin: Option<String>,
    // pub access: Option<Vec<String>>,
}

impl TurnAroundTime {
    pub fn adjusted_min(&self) -> Option<i64> {
        self.min.map(|m| m / self.turn_around_time_denominator())
    }

    pub fn adjusted_max(&self) -> Option<i64> {
        self.max.map(|m| m / self.turn_around_time_denominator())
    }

    fn turn_around_time_denominator(&self) -> i64 {
        match self.display_units.as_deref() {
            Some("hours") => 3600,
            Some("weeks") => 3600 * 24 * 7,
            Some("months") => 3600 * 24 * 30,
            _ => 3600 * 24,
        }
    }
}

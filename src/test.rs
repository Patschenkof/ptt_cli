#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::*, utils::get_activity_hours};

    fn test_time_record() -> TimeRecord {
        let t_for_test = TimeRecord {
            date: chrono::NaiveDate::from_ymd_opt(2025, 11, 9).unwrap(),
            start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            end_time: chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(), // 10 H 
            pause_minutes: 0.5, // 9.5 H
            project_entries: vec![
                ProjectEntry {
                    project_name: Project { code: String::from("INEK"), allocation: 1.0 },
                    hours: 3.5,
                    activity: String::from("I ran a test")

                }
            ],
        };
        return t_for_test;
    }

    #[test]
    fn test_get_net_hours() {
        let t_for_test = test_time_record();
        let net_hours = t_for_test.get_net_hours();
        assert_eq!(net_hours, 9.5);
    }

    #[test]
    fn test_allocated_hours() {
        let t_for_test = test_time_record();
        let allocated_hours = t_for_test.allocated_hours();
        assert_eq!(allocated_hours, 3.5)
    }

    #[test]
    fn test_remaining_hours() {
        let t_for_test = test_time_record();
        let remaining = t_for_test.remaining_hours();
        assert_eq!(6.0, remaining)
    }
}
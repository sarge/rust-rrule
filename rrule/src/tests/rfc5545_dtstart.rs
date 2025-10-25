mod rfc5545_dtstart_tests {
    use crate::tests::common;
    use crate::{RRule, RRuleSet, Unvalidated};
    use std::str::FromStr;

    #[test]
    fn test_include_dtstart_when_enabled() {
        let dates = "DTSTART:20230101T100000Z\n\
            RRULE:FREQ=DAILY;INTERVAL=2;COUNT=3;X-INCLUDE-DTSTART=TRUE"
            .parse::<RRuleSet>()
            .unwrap()
            .all(u16::MAX)
            .dates;

        common::check_occurrences(
            &dates,
            &[
                "2023-01-01T10:00:00+00:00", // DTSTART included as first occurrence (doesn't count towards COUNT)
                "2023-01-03T10:00:00+00:00", // First recurrence (2 days later)
                "2023-01-05T10:00:00+00:00", // Second recurrence (4 days from start)
                "2023-01-07T10:00:00+00:00", // Third recurrence (6 days from start)
            ],
        );
    }

    #[test]
    fn test_exclude_dtstart_when_disabled() {
        // Use a case where DTSTART doesn't match the pattern
        // DTSTART on Monday but rule is every Tuesday
        let dates = "DTSTART:20230102T100000Z\n\
            RRULE:FREQ=WEEKLY;BYDAY=TU;COUNT=3;X-INCLUDE-DTSTART=FALSE"
            .parse::<RRuleSet>()
            .unwrap()
            .all(u16::MAX)
            .dates;

        common::check_occurrences(
            &dates,
            &[
                "2023-01-03T10:00:00+00:00", // First Tuesday
                "2023-01-10T10:00:00+00:00", // Second Tuesday  
                "2023-01-17T10:00:00+00:00", // Third Tuesday
            ],
        );
    }

    #[test]
    fn test_default_behavior_includes_dtstart() {
        let dates = "DTSTART:20230101T100000Z\n\
            RRULE:FREQ=DAILY;INTERVAL=2;COUNT=3"
            .parse::<RRuleSet>()
            .unwrap()
            .all(u16::MAX)
            .dates;

        common::check_occurrences(
            &dates,
            &[
                "2023-01-01T10:00:00+00:00", // DTSTART (naturally included)
                "2023-01-03T10:00:00+00:00", // First recurrence (2 days from DTSTART)
                "2023-01-05T10:00:00+00:00", // Second recurrence
            ],
        );
    }

    #[test]
    fn test_parse_x_include_dtstart_parameter() {
        // Test parsing X-INCLUDE-DTSTART=TRUE
        let rrule_str = "FREQ=DAILY;INTERVAL=2;COUNT=3;X-INCLUDE-DTSTART=TRUE";
        let rrule: RRule<Unvalidated> = RRule::from_str(rrule_str).unwrap();
        assert_eq!(rrule.get_include_dtstart(), Some(&true));

        // Test parsing X-INCLUDE-DTSTART=FALSE
        let rrule_str = "FREQ=DAILY;INTERVAL=2;COUNT=3;X-INCLUDE-DTSTART=FALSE";
        let rrule: RRule<Unvalidated> = RRule::from_str(rrule_str).unwrap();
        assert_eq!(rrule.get_include_dtstart(), Some(&false));

        // Test parsing X-INCLUDE-DTSTART=1
        let rrule_str = "FREQ=DAILY;INTERVAL=2;COUNT=3;X-INCLUDE-DTSTART=1";
        let rrule: RRule<Unvalidated> = RRule::from_str(rrule_str).unwrap();
        assert_eq!(rrule.get_include_dtstart(), Some(&true));

        // Test parsing X-INCLUDE-DTSTART=0
        let rrule_str = "FREQ=DAILY;INTERVAL=2;COUNT=3;X-INCLUDE-DTSTART=0";
        let rrule: RRule<Unvalidated> = RRule::from_str(rrule_str).unwrap();
        assert_eq!(rrule.get_include_dtstart(), Some(&false));
    }

    #[test]
    fn test_invalid_x_include_dtstart_parameter() {
        // Test invalid value
        let rrule_str = "FREQ=DAILY;X-INCLUDE-DTSTART=MAYBE";
        let result: Result<RRule<Unvalidated>, _> = RRule::from_str(rrule_str);
        assert!(result.is_err());

        if let Err(crate::RRuleError::ParserError(parse_err)) = result {
            assert!(matches!(
                parse_err,
                crate::parser::ParseError::InvalidXIncludeDtstart(_)
            ));
        }
    }

    #[test]
    fn test_display_with_x_include_dtstart() {
        let rrule = RRule::new(crate::Frequency::Daily)
            .count(3)
            .include_dtstart(true);

        let rrule_str = rrule.to_string();
        assert!(rrule_str.contains("X-INCLUDE-DTSTART=TRUE"));

        let rrule = RRule::new(crate::Frequency::Daily)
            .count(3)
            .include_dtstart(false);

        let rrule_str = rrule.to_string();
        assert!(rrule_str.contains("X-INCLUDE-DTSTART=FALSE"));
    }

    #[test]
    fn test_weekly_recurrence_with_dtstart() {
        let dates = "DTSTART:20230101T100000Z\n\
            RRULE:FREQ=WEEKLY;COUNT=3;X-INCLUDE-DTSTART=TRUE"
            .parse::<RRuleSet>()
            .unwrap()
            .all(u16::MAX)
            .dates;

        common::check_occurrences(
            &dates,
            &[
                "2023-01-01T10:00:00+00:00", // DTSTART included (Sunday, doesn't count towards COUNT)
                "2023-01-08T10:00:00+00:00", // Next Sunday
                "2023-01-15T10:00:00+00:00", // Following Sunday
                "2023-01-22T10:00:00+00:00", // Third Sunday
            ],
        );
    }
}

#[cfg(test)]
mod local_tzid_integration_tests {
    use crate::{RRuleSet, Tz};
    use chrono::Timelike;

    #[test]
    fn local_tzid_replaces_force_utc_behavior() {
        // Test that LOCAL-TZID causes floating datetimes to be interpreted in the specified timezone
        // This replaces the behavior that was previously controlled by the force-utc feature

        // Test case 1: RRULE with LOCAL-TZID=UTC should convert floating datetime output to UTC
        // Using floating DTSTART (no TZID) so it will be affected by LOCAL-TZID
        let rrule_with_local_tzid = "DTSTART:19970902T090000\n\
            RRULE:FREQ=DAILY;LOCAL-TZID=UTC;UNTIL=19970904T090000Z;COUNT=3";

        let rrule_set = rrule_with_local_tzid.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        // Should generate dates successfully (validates that UNTIL was parsed correctly)
        assert!(dates.len() <= 3); // Limited by COUNT=3
        assert!(!dates.is_empty());

        // All dates should be converted to UTC timezone due to LOCAL-TZID=UTC
        // For floating datetime, LOCAL-TZID=UTC means the output should be in UTC
        // The floating 09:00 is parsed in system local time, then converted to UTC for output
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
            // The exact hour depends on system timezone, but it should be UTC
            // We just verify it's in UTC timezone (the key behavior we're testing)
        }

        // Test case 2: Just test RRULE parsing directly
        let rrule_str = "FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=2";
        let rrule = rrule_str
            .parse::<crate::RRule<crate::Unvalidated>>()
            .unwrap();
        assert_eq!(rrule.local_tzid, Some(Tz::America__New_York));
    }

    #[test]
    fn local_tzid_serialization_roundtrip() {
        // Test that LOCAL-TZID is preserved during parsing and serialization
        let original = "FREQ=DAILY;COUNT=3;LOCAL-TZID=Europe/London";
        let parsed = original
            .parse::<crate::RRule<crate::Unvalidated>>()
            .unwrap();
        let serialized = format!("{}", parsed);

        // LOCAL-TZID should be preserved
        assert!(serialized.contains("LOCAL-TZID=Europe/London"));
    }

    #[test]
    fn local_tzid_with_date_values() {
        // Test that LOCAL-TZID with DATE values (all-day events) works correctly
        // DATE values should always be at 00:00:00 in the LOCAL-TZID timezone

        let rrule_with_date_utc = "DTSTART;VALUE=DATE:20201214\n\
            RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=2";

        let rrule_set = rrule_with_date_utc.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 2);

        // All dates should be at 00:00:00 in UTC timezone
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
            assert_eq!(date.hour(), 0);
            assert_eq!(date.minute(), 0);
            assert_eq!(date.second(), 0);
        }

        // Test with different timezone
        let rrule_with_date_ny = "DTSTART;VALUE=DATE:20201214\n\
            RRULE:FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=2";

        let rrule_set_ny = rrule_with_date_ny.parse::<RRuleSet>().unwrap();
        let dates_ny = rrule_set_ny.all(u16::MAX).dates;

        assert_eq!(dates_ny.len(), 2);

        // All dates should be at 00:00:00 in America/New_York timezone
        for date in &dates_ny {
            assert_eq!(date.timezone(), Tz::America__New_York);
            assert_eq!(date.hour(), 0);
            assert_eq!(date.minute(), 0);
            assert_eq!(date.second(), 0);
        }
    }

    #[test]
    fn local_tzid_different_timezones() {
        // Test various timezone identifiers work with LOCAL-TZID
        let test_cases = [
            ("UTC", Some(Tz::UTC)),
            ("America/New_York", Some(Tz::America__New_York)),
            ("Europe/London", Some(Tz::Europe__London)),
            ("Asia/Tokyo", Some(Tz::Asia__Tokyo)),
        ];

        for (tz_str, expected_tz) in test_cases {
            let rrule_str = format!("FREQ=WEEKLY;COUNT=2;LOCAL-TZID={}", tz_str);
            let parsed = rrule_str
                .parse::<crate::RRule<crate::Unvalidated>>()
                .unwrap();
            assert_eq!(parsed.local_tzid, expected_tz);
        }
    }

    #[test]
    fn local_tzid_affects_output_timezone() {
        // Test that LOCAL-TZID converts the output timezone of generated dates

        // Test with floating DTSTART (no explicit timezone)
        let rrule_with_floating_start = "DTSTART:20120201T093000\n\
            RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=2";

        let rrule_set = rrule_with_floating_start.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 2);

        // All dates should be in UTC timezone
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
        }

        // Compare with the same RRULE but without LOCAL-TZID (should be in local timezone)
        let rrule_without_local_tzid = "DTSTART:20120201T093000\n\
            RRULE:FREQ=DAILY;COUNT=2";

        let rrule_set_local = rrule_without_local_tzid.parse::<RRuleSet>().unwrap();
        let dates_local = rrule_set_local.all(u16::MAX).dates;

        assert_eq!(dates_local.len(), 2);

        // These should be in the local timezone (not UTC)
        for date in &dates_local {
            assert_ne!(date.timezone(), Tz::UTC); // Should NOT be UTC
        }
    }
}

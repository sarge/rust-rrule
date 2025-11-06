#[cfg(test)]
mod local_tzid_integration_tests {
    use crate::{RRuleSet, Tz};
    use chrono::{Datelike, Timelike};

    #[test]
    fn local_tzid_replaces_force_utc_behavior() {
        // Test that LOCAL-TZID causes floating datetimes to be interpreted in the specified timezone
        // This replaces the behavior that was previously controlled by the force-utc feature

        // Test case 1: RRULE with LOCAL-TZID=UTC should convert floating datetime output to UTC
        // Using floating DTSTART (no TZID) so it will be affected by LOCAL-TZID
        let rrule_with_local_tzid = "RRULE:FREQ=DAILY;LOCAL-TZID=UTC;UNTIL=19970904T090000Z;COUNT=3\n\
            DTSTART:19970902T090000";

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

        let rrule_with_date_utc = "RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=2\n\
            DTSTART;VALUE=DATE:20201214";

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
        let rrule_with_date_ny = "RRULE:FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=2\n\
            DTSTART;VALUE=DATE:20201214";

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
    fn local_tzid_floating_datetime_detection() {
        // Test that both DATE and floating DATE-TIME values are correctly detected as floating
        
        // Test 1: DATE value should be detected as floating and converted to LOCAL-TZID
        let date_rrule = "RRULE:FREQ=DAILY;COUNT=2;BYHOUR=9;LOCAL-TZID=America/New_York\n\
            DTSTART;VALUE=DATE:20201214";
        
        let rrule_set = date_rrule.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;
        
        assert_eq!(dates.len(), 2);
        // All dates should be in America/New_York timezone due to floating datetime conversion
        for date in &dates {
            assert_eq!(date.timezone(), Tz::Tz(chrono_tz::America::New_York));
            assert_eq!(date.hour(), 9);
        }
        
        // Test 2: Floating DATE-TIME (no timezone) should be detected as floating and converted
        let floating_datetime_rrule = "RRULE:FREQ=DAILY;COUNT=2;LOCAL-TZID=America/New_York\n\
            DTSTART:20201214T120000";
        
        let rrule_set = floating_datetime_rrule.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;
        
        assert_eq!(dates.len(), 2);
        // All dates should be in America/New_York timezone due to floating datetime conversion
        for date in &dates {
            assert_eq!(date.timezone(), Tz::Tz(chrono_tz::America::New_York));
        }
        
        // Test 3: Non-floating DATE-TIME (with TZID) should NOT be converted by LOCAL-TZID
        let non_floating_rrule = "RRULE:FREQ=DAILY;COUNT=2;LOCAL-TZID=America/New_York\n\
            DTSTART;TZID=UTC:20201214T120000";
        
        let rrule_set = non_floating_rrule.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;
        
        assert_eq!(dates.len(), 2);
        // All dates should remain in UTC timezone, not converted to America/New_York
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
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
        let rrule_with_floating_start = "RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=2\n\
            DTSTART:20120201T093000";

        let rrule_set = rrule_with_floating_start.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 2);

        // All dates should be in UTC timezone
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
        }

        // Compare with the same RRULE but without LOCAL-TZID (should be in local timezone)
        let rrule_without_local_tzid = "RRULE:FREQ=DAILY;COUNT=2\n\
            DTSTART:20120201T093000";

        let rrule_set_local = rrule_without_local_tzid.parse::<RRuleSet>().unwrap();
        let dates_local = rrule_set_local.all(u16::MAX).dates;

        assert_eq!(dates_local.len(), 2);

        // These should be in the local timezone (not UTC)
        for date in &dates_local {
            assert_ne!(date.timezone(), Tz::UTC); // Should NOT be UTC
        }
    }

    #[test]
    fn local_tzid_with_date_values_and_byhour() {
        // Test with DATE values with BYHOUR parameters work correctly with LOCAL-TZID
        // This ensures that when DTSTART is a DATE (00:00:00), the BYHOUR times
        // are correctly interpreted in the LOCAL-TZID timezone

        // Test with UTC timezone
        let rrule_with_byhour_utc = "RRULE:FREQ=DAILY;BYHOUR=9,12,15;LOCAL-TZID=UTC;COUNT=6\n\
            DTSTART;VALUE=DATE:20201214";

        let rrule_set = rrule_with_byhour_utc.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 6);

        // First day should have three times: 09:00, 12:00, 15:00 UTC
        assert_eq!(dates[0].year(), 2020);
        assert_eq!(dates[0].month(), 12);
        assert_eq!(dates[0].day(), 14);
        assert_eq!(dates[0].hour(), 9);
        assert_eq!(dates[0].timezone(), Tz::UTC);

        assert_eq!(dates[1].hour(), 12);
        assert_eq!(dates[1].timezone(), Tz::UTC);

        assert_eq!(dates[2].hour(), 15);
        assert_eq!(dates[2].timezone(), Tz::UTC);

        // Second day should start at 09:00 UTC again
        assert_eq!(dates[3].year(), 2020);
        assert_eq!(dates[3].month(), 12);
        assert_eq!(dates[3].day(), 15);
        assert_eq!(dates[3].hour(), 9);
        assert_eq!(dates[3].timezone(), Tz::UTC);

        // Test with America/New_York timezone
        let rrule_with_byhour_ny = "RRULE:FREQ=DAILY;BYHOUR=9,12,15;LOCAL-TZID=America/New_York;COUNT=3\n\
            DTSTART;VALUE=DATE:20201214";

        let rrule_set_ny = rrule_with_byhour_ny.parse::<RRuleSet>().unwrap();
        let dates_ny = rrule_set_ny.all(u16::MAX).dates;

        assert_eq!(dates_ny.len(), 3);

        // All dates should be in America/New_York timezone with correct hours
        for (i, expected_hour) in [9, 12, 15].iter().enumerate() {
            assert_eq!(dates_ny[i].timezone(), Tz::America__New_York);
            assert_eq!(dates_ny[i].hour(), *expected_hour);
            assert_eq!(dates_ny[i].year(), 2020);
            assert_eq!(dates_ny[i].month(), 12);
            assert_eq!(dates_ny[i].day(), 14);
        }
    }

    #[test]
    fn local_tzid_with_floating_datetime() {
        // Test that floating datetimes (no explicit timezone) are converted to LOCAL-TZID
        // Floating datetimes are initially parsed as local timezone but should be converted
        // to LOCAL-TZID when specified.

        // Test with floating datetime (no Z suffix, no TZID parameter)
        // LOCAL-TZID must appear before DTSTART
        let rrule_with_floating = "RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=20\n\
            DTSTART:20201214T093000";

        let rrule_set = rrule_with_floating.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 20);

        // All dates should be in UTC timezone (converted from floating/local)
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
            // The time should be preserved from the original floating datetime
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
            assert_eq!(date.second(), 0);
        }

        // Test with floating datetime and different LOCAL-TZID
        let rrule_with_floating_ny = "RRULE:FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=3\n\
            DTSTART:20201214T093000";

        let rrule_set_ny = rrule_with_floating_ny.parse::<RRuleSet>().unwrap();
        let dates_ny = rrule_set_ny.all(u16::MAX).dates;

        assert_eq!(dates_ny.len(), 3);

        // All dates should be in America/New_York timezone
        for date in &dates_ny {
            assert_eq!(date.timezone(), Tz::America__New_York);
            // The time should be preserved from the original floating datetime
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
            assert_eq!(date.second(), 0);
        }

        // Compare with explicit UTC datetime (should behave differently)
        let rrule_with_utc = "RRULE:FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=2\n\
            DTSTART:20201214T093000Z";

        let rrule_set_utc = rrule_with_utc.parse::<RRuleSet>().unwrap();
        let dates_utc = rrule_set_utc.all(u16::MAX).dates;

        // UTC datetimes should NOT be affected by LOCAL-TZID
        for date in &dates_utc {
            assert_eq!(date.timezone(), Tz::UTC); // Should remain UTC, not converted
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
        }
    }

    #[test]
    fn local_tzid_with_floating_datetime_and_byhour() {
        // Test floating datetime with BYHOUR parameter to ensure it works correctly
        let rrule_floating_byhour = "RRULE:FREQ=DAILY;BYHOUR=9,15;LOCAL-TZID=UTC;COUNT=4\n\
            DTSTART:20201214T120000";

        let rrule_set = rrule_floating_byhour.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;



        assert_eq!(dates.len(), 4);

        // Should generate times at 09:00 and 15:00 UTC for each day
        // Note: DTSTART is 12:00, so on Dec 14 only 15:00 matches BYHOUR=9,15 
        // (since 12:00 is after 09:00 but before 15:00)
        let expected_hours = [15, 9, 15, 9]; // Dec 14: 15; Dec 15: 9,15; Dec 16: 9
        for (i, date) in dates.iter().enumerate() {
            assert_eq!(date.timezone(), Tz::UTC);
            assert_eq!(date.hour(), expected_hours[i]);
            assert_eq!(date.minute(), 0);
            assert_eq!(date.second(), 0);
        }

        // Dates should be spread across Dec 14, 15, 16
        assert_eq!(dates[0].day(), 14); // Dec 14 15:00
        assert_eq!(dates[1].day(), 15); // Dec 15 09:00  
        assert_eq!(dates[2].day(), 15); // Dec 15 15:00
        assert_eq!(dates[3].day(), 16); // Dec 16 09:00
    }

    #[test]
    fn local_tzid_with_floating_datetime_and_x_include_dtstart() {
        // Test X-INCLUDE-DTSTART behavior with floating datetimes and LOCAL-TZID
        // This ensures that both DTSTART and generated recurrences are correctly 
        // handled with the LOCAL-TZID conversion

        // Test 1: X-INCLUDE-DTSTART=TRUE with floating datetime
        let rrule_include_true = "RRULE:FREQ=DAILY;LOCAL-TZID=UTC;COUNT=2;X-INCLUDE-DTSTART=TRUE\n\
            DTSTART:20201214T093000";

        let rrule_set = rrule_include_true.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 3); // DTSTART + 2 recurrences (X-INCLUDE-DTSTART doesn't count towards COUNT)
        
        // All dates should be in UTC (converted from floating)
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC);
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
            assert_eq!(date.second(), 0);
        }

        // Check specific dates
        assert_eq!(dates[0].day(), 14); // DTSTART: Dec 14
        assert_eq!(dates[1].day(), 15); // First recurrence: Dec 15  
        assert_eq!(dates[2].day(), 16); // Second recurrence: Dec 16

        // Test 2: X-INCLUDE-DTSTART=FALSE with floating datetime 
        // NOTE: DTSTART naturally matches DAILY pattern, so X-INCLUDE-DTSTART=FALSE should exclude it
        let rrule_include_false = "RRULE:FREQ=WEEKLY;BYDAY=TU;LOCAL-TZID=America/New_York;COUNT=2;X-INCLUDE-DTSTART=FALSE\n\
            DTSTART:20201214T093000";  // Dec 14, 2020 is a Monday

        let rrule_set = rrule_include_false.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        assert_eq!(dates.len(), 2); // Only 2 Tuesday recurrences, DTSTART (Monday) naturally excluded
        
        // All dates should be in America/New_York (converted from floating)
        for date in &dates {
            assert_eq!(date.timezone(), Tz::America__New_York);
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
            assert_eq!(date.second(), 0);
            // Should be Tuesdays
            assert_eq!(date.weekday(), chrono::Weekday::Tue);
        }

        // Check that we get Tuesdays (Dec 15, 22 are Tuesdays in 2020)
        assert_eq!(dates[0].day(), 15); // First Tuesday: Dec 15
        assert_eq!(dates[1].day(), 22); // Second Tuesday: Dec 22
        
        // Test 3: Compare with explicit UTC datetime (should NOT be affected by LOCAL-TZID)
        let rrule_explicit_utc = "RRULE:FREQ=DAILY;LOCAL-TZID=America/New_York;COUNT=2;X-INCLUDE-DTSTART=TRUE\n\
            DTSTART:20201214T093000Z";

        let rrule_set = rrule_explicit_utc.parse::<RRuleSet>().unwrap();
        let dates = rrule_set.all(u16::MAX).dates;

        // UTC DTSTART should remain in UTC, not converted to LOCAL-TZID
        for date in &dates {
            assert_eq!(date.timezone(), Tz::UTC); // Should stay UTC
            assert_eq!(date.hour(), 9);
            assert_eq!(date.minute(), 30);
        }
    }
}

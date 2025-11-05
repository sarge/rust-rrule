use std::{collections::HashMap, marker::PhantomData, str::FromStr};

use chrono::Weekday;

use crate::{
    parser::{
        content_line::parameters::parse_parameters, datetime::parse_weekdays, str_to_weekday,
        utils::parse_str_to_vec, ParseError,
    },
    Frequency, RRule, Unvalidated,
};

use super::content_line_parts::ContentLineCaptures;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum RRuleProperty {
    Freq,
    Until,
    Count,
    Interval,
    BySecond,
    ByMinute,
    ByHour,
    ByDay,
    ByMonthDay,
    ByYearDay,
    ByWeekNo,
    ByMonth,
    BySetPos,
    Wkst,
    #[cfg(feature = "by-easter")]
    ByEaster,
    XIncludeDtstart,
    LocalTzid,
}

impl FromStr for RRuleProperty {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prop = match &s.to_uppercase()[..] {
            "FREQ" => Self::Freq,
            "UNTIL" => Self::Until,
            "COUNT" => Self::Count,
            "INTERVAL" => Self::Interval,
            "BYSECOND" => Self::BySecond,
            "BYMINUTE" => Self::ByMinute,
            "BYHOUR" => Self::ByHour,
            "BYWEEKDAY" | "BYDAY" => Self::ByDay,
            "BYMONTHDAY" => Self::ByMonthDay,
            "BYYEARDAY" => Self::ByYearDay,
            "BYWEEKNO" => Self::ByWeekNo,
            "BYMONTH" => Self::ByMonth,
            "BYSETPOS" => Self::BySetPos,
            "WKST" => Self::Wkst,
            #[cfg(feature = "by-easter")]
            "BYEASTER" => Self::ByEaster,
            "X-INCLUDE-DTSTART" => Self::XIncludeDtstart,
            "LOCAL-TZID" => Self::LocalTzid,
            _ => return Err(ParseError::UnrecognizedParameter(s.into())),
        };
        Ok(prop)
    }
}

impl TryFrom<ContentLineCaptures<'_>> for RRule<Unvalidated> {
    type Error = ParseError;

    fn try_from(value: ContentLineCaptures) -> Result<Self, Self::Error> {
        if let Some(parameters) = value.parameters {
            if !parameters.is_empty() {
                return Err(ParseError::PropertyParametersNotSupported(
                    parameters.into(),
                ));
            }
        }

        let properties: HashMap<RRuleProperty, String> = parse_parameters(value.value)?;

        props_to_rrule(&properties)
    }
}

/// Takes a map of [`RRuleProperty`] and returns an [`RRule`].
#[allow(clippy::too_many_lines)]
fn props_to_rrule(
    props: &HashMap<RRuleProperty, String>,
) -> Result<RRule<Unvalidated>, ParseError> {
    let freq = props
        .get(&RRuleProperty::Freq)
        .map(|freq| Frequency::from_str(freq))
        .transpose()?
        .ok_or_else(|| ParseError::MissingProperty("FREQ".into()))?;
    let interval = props
        .get(&RRuleProperty::Interval)
        .map(|interval| {
            interval
                .parse()
                .map_err(|_| ParseError::InvalidInterval(interval.into()))
        })
        .transpose()?
        .unwrap_or(1);
    let count = props
        .get(&RRuleProperty::Count)
        .map(|count| {
            count
                .parse()
                .map_err(|_| ParseError::InvalidCount(count.into()))
        })
        .transpose()?;

    // Parse LOCAL-TZID first as it may be needed for other datetime parsing
    let local_tzid = props
        .get(&RRuleProperty::LocalTzid)
        .map(|tzid_str: &String| {
            use crate::parser::datetime::parse_timezone;
            parse_timezone(tzid_str)
        })
        .transpose()?;

    let until = props
        .get(&RRuleProperty::Until)
        .map(|until| {
            use crate::parser::datetime::datestring_to_date_with_local_tzid;
            datestring_to_date_with_local_tzid(until, None, "UNTIL", local_tzid)
        })
        .transpose()?;
    let week_start = props
        .get(&RRuleProperty::Wkst)
        .map(|week_start| {
            str_to_weekday(week_start)
                .map_err(|_| ParseError::InvalidWeekdayStart(week_start.into()))
        })
        .transpose()?
        .unwrap_or(Weekday::Mon);
    let by_set_pos = props
        .get(&RRuleProperty::BySetPos)
        .map(|by_set_pos| {
            parse_str_to_vec(by_set_pos, |_| true)
                .map_err(|_| ParseError::InvalidBySetPos(by_set_pos.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_month = props
        .get(&RRuleProperty::ByMonth)
        .map(|by_month| {
            parse_str_to_vec(by_month, |month| (1..=12).contains(&month))
                .map_err(|_| ParseError::InvalidByMonth(by_month.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_month_day = props
        .get(&RRuleProperty::ByMonthDay)
        .map(|by_month_day| {
            parse_str_to_vec(by_month_day, |monthday| (-31..=31).contains(&monthday))
                .map_err(|_| ParseError::InvalidByMonthDay(by_month_day.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_year_day = props
        .get(&RRuleProperty::ByYearDay)
        .map(|by_year_day| {
            parse_str_to_vec(by_year_day, |yearday| (-366..=366).contains(&yearday))
                .map_err(|_| ParseError::InvalidByYearDay(by_year_day.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_week_no = props
        .get(&RRuleProperty::ByWeekNo)
        .map(|by_week_no| {
            parse_str_to_vec(by_week_no, |weekno| (-53..=53).contains(&weekno))
                .map_err(|_| ParseError::InvalidByWeekNo(by_week_no.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_weekday = props
        .get(&RRuleProperty::ByDay)
        .map(|by_weekday| parse_weekdays(by_weekday))
        .transpose()?
        .unwrap_or_default();
    let by_hour = props
        .get(&RRuleProperty::ByHour)
        .map(|by_hour| {
            parse_str_to_vec(by_hour, |hour| hour < 24)
                .map_err(|_| ParseError::InvalidByHour(by_hour.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_minute = props
        .get(&RRuleProperty::ByMinute)
        .map(|by_minute| {
            parse_str_to_vec(by_minute, |minute| minute < 60)
                .map_err(|_| ParseError::InvalidByMinute(by_minute.into()))
        })
        .transpose()?
        .unwrap_or_default();
    let by_second = props
        .get(&RRuleProperty::BySecond)
        .map(|by_second| {
            parse_str_to_vec(by_second, |second| second < 60)
                .map_err(|_| ParseError::InvalidBySecond(by_second.into()))
        })
        .transpose()?
        .unwrap_or_default();

    #[cfg(not(feature = "by-easter"))]
    let by_easter = None;
    #[cfg(feature = "by-easter")]
    let by_easter = props
        .get(&RRuleProperty::ByEaster)
        .map(|new_by_easter: &String| {
            i16::from_str(new_by_easter)
                .map_err(|_| ParseError::InvalidByEaster(new_by_easter.into()))
        })
        .transpose()?;

    let include_dtstart = props
        .get(&RRuleProperty::XIncludeDtstart)
        .map(
            |include_str: &String| match include_str.to_uppercase().as_str() {
                "TRUE" | "1" | "YES" => Ok(true),
                "FALSE" | "0" | "NO" => Ok(false),
                _ => Err(ParseError::InvalidXIncludeDtstart(include_str.clone())),
            },
        )
        .transpose()?;

    // Check if mandatory fields are set
    Ok(RRule {
        freq,
        interval,
        count,
        until,
        week_start,
        by_set_pos,
        by_month,
        by_month_day,
        by_n_month_day: vec![],
        by_year_day,
        by_week_no,
        by_weekday,
        by_hour,
        by_minute,
        by_second,
        by_easter,
        include_dtstart,
        local_tzid,
        stage: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::content_line::{ContentLineCaptures, PropertyName};

    use super::*;

    #[test]
    fn parses_rrule_content_line() {
        let tests = [
            (
                ContentLineCaptures {
                    property_name: PropertyName::RRule,
                    parameters: None,
                    value: "FREQ=DAILY",
                },
                RRule {
                    freq: Frequency::Daily,
                    ..Default::default()
                },
            ),
            (
                ContentLineCaptures {
                    property_name: PropertyName::RRule,
                    parameters: None,
                    value: "BYHOUR=4;FREQ=DAILY",
                },
                RRule {
                    by_hour: vec![4],
                    freq: Frequency::Daily,
                    ..Default::default()
                },
            ),
            (
                ContentLineCaptures {
                    property_name: PropertyName::RRule,
                    parameters: None,
                    // Testing case insensitivity
                    value: "byhour=4;freQ=DAILY",
                },
                RRule {
                    by_hour: vec![4],
                    freq: Frequency::Daily,
                    ..Default::default()
                },
            ),
        ];

        for (input, expected_output) in tests {
            let output = RRule::try_from(input);
            assert_eq!(output, Ok(expected_output));
        }
    }

    #[test]
    fn parses_local_tzid_parameter() {
        let tests = [
            (
                ContentLineCaptures {
                    property_name: PropertyName::RRule,
                    parameters: None,
                    value: "FREQ=DAILY;LOCAL-TZID=America/New_York",
                },
                RRule {
                    freq: Frequency::Daily,
                    local_tzid: Some(crate::Tz::America__New_York),
                    ..Default::default()
                },
            ),
            (
                ContentLineCaptures {
                    property_name: PropertyName::RRule,
                    parameters: None,
                    value: "FREQ=WEEKLY;LOCAL-TZID=UTC",
                },
                RRule {
                    freq: Frequency::Weekly,
                    local_tzid: Some(crate::Tz::UTC),
                    ..Default::default()
                },
            ),
        ];

        for (input, expected_output) in tests {
            let output = RRule::try_from(input);
            assert_eq!(output, Ok(expected_output));
        }
    }

    #[test]
    fn rejects_invalid_local_tzid() {
        let tests = [(
            ContentLineCaptures {
                property_name: PropertyName::RRule,
                parameters: None,
                value: "FREQ=DAILY;LOCAL-TZID=Invalid/Timezone",
            },
            ParseError::InvalidTimezone("Invalid/Timezone".into()),
        )];

        for (input, expected_output) in tests {
            let output = RRule::try_from(input);
            assert_eq!(output, Err(expected_output));
        }
    }

    #[test]
    fn local_tzid_affects_until_parsing() {
        use crate::Tz;

        // Test that LOCAL-TZID affects UNTIL datetime parsing
        let tests = [
            (
                // Without LOCAL-TZID, floating datetime uses local time
                "FREQ=DAILY;UNTIL=19970904T090000",
                None, // no local_tzid
            ),
            (
                // With LOCAL-TZID, floating datetime uses specified timezone
                "FREQ=DAILY;LOCAL-TZID=America/New_York;UNTIL=19970904T090000",
                Some(Tz::America__New_York),
            ),
        ];

        for (input_str, expected_local_tzid) in tests {
            let input = ContentLineCaptures {
                property_name: PropertyName::RRule,
                parameters: None,
                value: input_str,
            };
            let output = RRule::try_from(input).unwrap();
            assert_eq!(output.local_tzid, expected_local_tzid);

            // Verify UNTIL was parsed (would fail if LOCAL-TZID caused parsing error)
            assert!(output.until.is_some());
        }
    }

    #[test]
    fn rejects_property_parameters_in_rrule_line() {
        let tests = [(
            ContentLineCaptures {
                property_name: PropertyName::RRule,
                parameters: Some("TZID=Europe/London"),
                value: "BYHOUR=4",
            },
            ParseError::PropertyParametersNotSupported("TZID=Europe/London".into()),
        )];

        for (input, expected_output) in tests {
            let output = RRule::try_from(input);
            assert_eq!(output, Err(expected_output));
        }
    }

    #[test]
    fn rejects_invalid_freq() {
        let mut props = HashMap::new();
        props.insert(RRuleProperty::Freq, "DAIL".into());
        let res = props_to_rrule(&props);
        assert_eq!(
            res.unwrap_err(),
            ParseError::InvalidFrequency("DAIL".into())
        );
    }

    #[test]
    fn rejects_invalid_byhour() {
        let mut props = HashMap::new();
        props.insert(RRuleProperty::Freq, "DAILY".into());
        props.insert(RRuleProperty::ByHour, "24".into());
        let res = props_to_rrule(&props);
        assert_eq!(res.unwrap_err(), ParseError::InvalidByHour("24".into()));

        props.insert(RRuleProperty::ByHour, "5,6,25".into());
        let res = props_to_rrule(&props);
        assert_eq!(res.unwrap_err(), ParseError::InvalidByHour("5,6,25".into()));
    }

    #[test]
    fn rejects_invalid_byminute() {
        let mut props = HashMap::new();
        props.insert(RRuleProperty::Freq, "DAILY".into());
        props.insert(RRuleProperty::ByMinute, "60".into());
        let res = props_to_rrule(&props);
        assert_eq!(res.unwrap_err(), ParseError::InvalidByMinute("60".into()));

        props.insert(RRuleProperty::ByMinute, "4,5,64".into());
        let res = props_to_rrule(&props);
        assert_eq!(
            res.unwrap_err(),
            ParseError::InvalidByMinute("4,5,64".into())
        );
    }
}

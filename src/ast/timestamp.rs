use rowan::NodeOrToken;

use super::{filter_token, Timestamp};
use crate::syntax::SyntaxKind;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeUnit {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RepeaterType {
    Cumulate,
    CatchUp,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DelayType {
    All,
    First,
}

impl Timestamp {
    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let ts = Org::parse("<2003-09-16 Tue 09:39-10:39>").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_active());
    /// let ts = Org::parse("<2003-09-16 Tue 09:39>--<2003-09-16 Tue 10:39>").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_active());
    /// let ts = Org::parse("<2003-09-16 Tue 09:39>").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_ACTIVE
    }

    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// ```
    pub fn is_inactive(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_INACTIVE
    }

    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let ts = Org::parse("<%%(org-calendar-holiday)>").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_diary());
    /// ```
    pub fn is_diary(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_DIARY
    }

    /// Returns `true` if this timestamp has a range
    ///
    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_range());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.is_range());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]").first_node::<Timestamp>().unwrap();
    /// assert!(!ts.is_range());
    /// ```
    pub fn is_range(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::MINUS))
            .count()
            > 2
    }

    /// ```rust
    /// use orgize::{Org, ast::{Timestamp, RepeaterType}};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), Some(RepeaterType::Cumulate));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), Some(RepeaterType::Restart));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), None);
    /// ```
    pub fn repeater_type(&self) -> Option<RepeaterType> {
        self.syntax
            .children_with_tokens()
            .find_map(filter_token(SyntaxKind::TIMESTAMP_REPEATER_MARK))
            .map(|t| match t.text() {
                "++" => RepeaterType::CatchUp,
                "+" => RepeaterType::Cumulate,
                ".+" => RepeaterType::Restart,
                _ => {
                    debug_assert!(false);
                    RepeaterType::CatchUp
                }
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), Some(1));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), Some(10));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), None);
    /// ```
    pub fn repeater_value(&self) -> Option<u32> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::TIMESTAMP_REPEATER_MARK)
            .nth(1)
            .and_then(|e| match e {
                NodeOrToken::Token(t) => {
                    debug_assert!(t.kind() == SyntaxKind::TIMESTAMP_VALUE);
                    t.text().parse().ok()
                }
                _ => None,
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::{Timestamp, TimeUnit}};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), Some(TimeUnit::Week));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), Some(TimeUnit::Day));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), None);
    /// ```
    pub fn repeater_unit(&self) -> Option<TimeUnit> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::TIMESTAMP_REPEATER_MARK)
            .nth(2)
            .and_then(|e| match e {
                NodeOrToken::Token(t) => {
                    debug_assert!(t.kind() == SyntaxKind::TIMESTAMP_UNIT);
                    match t.text() {
                        "h" => Some(TimeUnit::Hour),
                        "d" => Some(TimeUnit::Day),
                        "w" => Some(TimeUnit::Week),
                        "m" => Some(TimeUnit::Month),
                        "y" => Some(TimeUnit::Year),
                        _ => None,
                    }
                }
                _ => None,
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::{Timestamp, DelayType}};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::All));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::All));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::First));
    /// ```
    pub fn warning_type(&self) -> Option<DelayType> {
        self.syntax
            .children_with_tokens()
            .find_map(filter_token(SyntaxKind::TIMESTAMP_DELAY_MARK))
            .map(|t| match t.text() {
                "-" => DelayType::All,
                "--" => DelayType::First,
                _ => {
                    debug_assert!(false);
                    DelayType::All
                }
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(3));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(5));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(10));
    /// ```
    pub fn warning_value(&self) -> Option<u32> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::TIMESTAMP_DELAY_MARK)
            .nth(1)
            .and_then(|e| match e {
                NodeOrToken::Token(t) => {
                    debug_assert!(t.kind() == SyntaxKind::TIMESTAMP_VALUE);
                    t.text().parse().ok()
                }
                _ => None,
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::{Timestamp, TimeUnit}};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(TimeUnit::Year));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(TimeUnit::Week));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(TimeUnit::Month));
    /// ```
    pub fn warning_unit(&self) -> Option<TimeUnit> {
        self.syntax
            .children_with_tokens()
            .skip_while(|n| n.kind() != SyntaxKind::TIMESTAMP_DELAY_MARK)
            .nth(2)
            .and_then(|e| match e {
                NodeOrToken::Token(t) => {
                    debug_assert!(t.kind() == SyntaxKind::TIMESTAMP_UNIT);
                    match t.text() {
                        "h" => Some(TimeUnit::Hour),
                        "d" => Some(TimeUnit::Day),
                        "w" => Some(TimeUnit::Week),
                        "m" => Some(TimeUnit::Month),
                        "y" => Some(TimeUnit::Year),
                        _ => None,
                    }
                }
                _ => None,
            })
    }

    /// Converts timestamp start to chrono NaiveDateTime
    ///
    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    /// use chrono::NaiveDateTime;
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(ts.start_to_chrono().unwrap(), "2003-09-16T09:39:00".parse::<NaiveDateTime>().unwrap());
    ///
    /// let ts = Org::parse("[2003-13-00 Tue 09:39-10:39]").first_node::<Timestamp>().unwrap();
    /// assert!(ts.start_to_chrono().is_none());
    /// ```
    #[cfg(feature = "chrono")]
    pub fn start_to_chrono(&self) -> Option<chrono::NaiveDateTime> {
        Some(chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                self.year_start()?.text().parse().ok()?,
                self.month_start()?.text().parse().ok()?,
                self.day_start()?.text().parse().ok()?,
            )?,
            chrono::NaiveTime::from_hms_opt(
                self.hour_start()?.text().parse().ok()?,
                self.minute_start()?.text().parse().ok()?,
                0,
            )?,
        ))
    }

    /// Converts timestamp end to chrono NaiveDateTime
    ///
    /// ```rust
    /// use orgize::{Org, ast::Timestamp};
    /// use chrono::NaiveDateTime;
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<Timestamp>().unwrap();
    /// assert_eq!(ts.end_to_chrono().unwrap(), "2003-09-16T10:39:00".parse::<NaiveDateTime>().unwrap());
    /// ```
    #[cfg(feature = "chrono")]
    pub fn end_to_chrono(&self) -> Option<chrono::NaiveDateTime> {
        Some(chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                self.year_end()?.text().parse().ok()?,
                self.month_end()?.text().parse().ok()?,
                self.day_end()?.text().parse().ok()?,
            )?,
            chrono::NaiveTime::from_hms_opt(
                self.hour_end()?.text().parse().ok()?,
                self.minute_end()?.text().parse().ok()?,
                0,
            )?,
        ))
    }
}

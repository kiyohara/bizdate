//! 利用者指定休日と業務日を扱う。CLI の option 配線や exit code は扱わない。

use std::{collections::BTreeSet, error::Error, fmt};

use jiff::{
    Timestamp,
    civil::{Date, Weekday},
};

use crate::{
    date::{DateError, parse_date},
    holidays::{HolidayData, HolidayError},
};

/// 不正な要素を元の入力のまま保持する。
#[derive(Debug)]
pub struct DayOffError {
    pub input: String,
    pub source: DateError,
}

impl fmt::Display for DayOffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid day-off element {:?} (expected YYYY-MM-DD or MM-DD): {}",
            self.input, self.source
        )
    }
}

impl Error for DayOffError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// 利用者指定休日。option 未指定時は `default()` の空集合を使う。
#[derive(Debug, Default)]
pub struct DaysOff(BTreeSet<Date>);

impl DaysOff {
    /// `today` は `date::today` が返す採用 TZ の今日とし、判定対象日を渡さない。
    /// 空文字・空要素・空白を含む要素はエラーにし、重複は集合にまとめる。
    pub fn parse(input: &str, today: Date) -> Result<Self, DayOffError> {
        input
            .split(',')
            .map(|element| {
                let bytes = element.as_bytes();
                let date = if bytes.len() == 5
                    && bytes.iter().enumerate().all(|(i, b)| {
                        if i == 2 {
                            *b == b'-'
                        } else {
                            b.is_ascii_digit()
                        }
                    }) {
                    let month = element[..2].parse().expect("validated as two ASCII digits");
                    let day = element[3..].parse().expect("validated as two ASCII digits");
                    Date::new(today.year(), month, day).map_err(DateError::InvalidDate)
                } else {
                    parse_date(element)
                };
                date.map_err(|source| DayOffError {
                    input: element.to_owned(),
                    source,
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()
            .map(Self)
    }

    pub fn contains(&self, date: Date) -> bool {
        self.0.contains(&date)
    }
}

/// 一度取得した現在時刻と読み込み済みデータを使う、1 回の判定用カレンダー。
/// 新しい判定ではその時点の `now` を渡して作り直す。
pub struct BusinessCalendar<'a> {
    holidays: &'a HolidayData,
    days_off: &'a DaysOff,
    now: Timestamp,
}

impl<'a> BusinessCalendar<'a> {
    pub fn new(holidays: &'a HolidayData, days_off: &'a DaysOff, now: Timestamp) -> Self {
        Self {
            holidays,
            days_off,
            now,
        }
    }

    /// 土日や利用者指定休日でも、先に祝日データの期限と年カバーを検証する。
    pub fn is_business_day(&self, date: Date) -> Result<bool, HolidayError> {
        let holiday = self.holidays.is_holiday(date, self.now)?;
        Ok(!holiday
            && !matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
            && !self.days_off.contains(date))
    }

    /// 指定日の属する月の最初の業務日。業務日が 0 日なら `None`。
    pub fn first_business_day(&self, date: Date) -> Result<Option<Date>, HolidayError> {
        self.find_business_day(date, 1..=date.days_in_month())
    }

    /// 指定日の属する月の最後の業務日。業務日が 0 日なら `None`。
    pub fn last_business_day(&self, date: Date) -> Result<Option<Date>, HolidayError> {
        self.find_business_day(date, (1..=date.days_in_month()).rev())
    }

    fn find_business_day(
        &self,
        date: Date,
        days: impl Iterator<Item = i8>,
    ) -> Result<Option<Date>, HolidayError> {
        for day in days {
            // 同じ年・月の実在する日だけを列挙する。翌月への加算は不要。
            let candidate = Date::new(date.year(), date.month(), day)
                .expect("day is within the specified calendar month");
            if self.is_business_day(candidate)? {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;

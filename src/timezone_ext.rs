use chrono::{DateTime, Local, TimeZone, Utc};

pub trait TimeZoneExt
where
    Self: TimeZone,
    Self::Offset: Copy,
{
    fn current_datetime() -> DateTime<Self>;
}

impl TimeZoneExt for Utc {
    #[inline]
    fn current_datetime() -> DateTime<Self> {
        Utc::now()
    }
}

impl TimeZoneExt for Local {
    #[inline]
    fn current_datetime() -> DateTime<Self> {
        Local::now()
    }
}

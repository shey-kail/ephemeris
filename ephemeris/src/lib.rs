pub mod internal;
pub mod lunnar;
pub mod astronomy;


/// 暴露 AstroyDate 等结构调用
///
/// `JulianDate` 提供以下时间转换功能:
/// - `from_day(year, month, day)`: 从公历日期构造儒略日 (day 可用小数表示时分)
/// - `from_ymdhms_tt(year, month, day, hour, minute, second)`: 从公历年月日时分秒构造**力学时 (TT)** 儒略日
/// - `from_ymdhms_ut(year, month, day, hour, minute, second)`: 从公历年月日时分秒构造**UT1** 儒略日
/// - `jd2day(jd)`: 儒略日转为公历日期
///
/// # Example
///
/// ```
/// use rust_ephemeris::JulianDate;
///
/// // 方式 1: 使用 from_day (day 可用小数表示时间)
/// let jd1 = JulianDate::from_day(2023, 7, 23.5); // 2023-07-23 12:00
///
/// // 方式 2: 使用 from_ymdhms_tt (推荐，自动 UT1→TT 转换)
/// let jd2 = JulianDate::from_ymdhms_tt(2023, 7, 23, 12, 0, 0.0);
///
/// // 儒略日转公历
/// let (y, m, d) = JulianDate::jd2day(jd2.jd);
/// ```
pub  use crate::internal::lunnar::JulianDate;
pub  use crate::internal::math_utils;
pub use crate::internal::math_utils::Angle;

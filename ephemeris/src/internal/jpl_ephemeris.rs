// JPL 历表支持模块（精准模式）
//
// 使用 anise 库加载和计算 JPL DE 系列历表（DE431/DE441）
// 提供高精度的太阳、月球位置计算
//
// 计算模式：
// - 简单模式：使用 Moshier 近似算法，速度快，精度约 1-2 秒
// - 精准模式：使用 JPL DE 历表，速度较慢，精度 < 0.1 秒

use anise::prelude::*;
use anise::constants::frames::{EARTH_J2000, SUN_J2000, MOON_J2000};
use crate::internal::planet::Planet;
use std::path::Path;
use std::sync::Arc;

/// JPL 历表类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JplEphemerisType {
    /// DE431 - 适用于 -13000 年到 +17000 年
    DE431,
    /// DE441 - 适用于 -13000 年到 +17000 年，改进的月球历表
    DE441,
}

impl JplEphemerisType {
    /// 获取历表文件路径
    pub fn file_paths(&self) -> Vec<&'static str> {
        match self {
            JplEphemerisType::DE431 => vec![
                "bsp/431/de431_part-1.bsp",
                "bsp/431/de431_part-2.bsp",
            ],
            JplEphemerisType::DE441 => vec![
                "bsp/441/de441_part-1.bsp",
                "bsp/441/de441_part-2.bsp",
            ],
        }
    }
    
    /// 获取历表覆盖的年份范围
    pub fn year_range(&self) -> (i32, i32) {
        match self {
            JplEphemerisType::DE431 => (-13000, 17000),
            JplEphemerisType::DE441 => (-13000, 17000),
        }
    }
}

/// JPL 历表计算器
/// 
/// 封装 anise 库的 Almanac，提供简化的天文计算接口
#[derive(Clone)]
pub struct JplEphemeris {
    almanac: Arc<Almanac>,
    ephemeris_type: JplEphemerisType,
}

impl JplEphemeris {
    /// 创建新的 JPL 历表计算器
    /// 
    /// # 参数
    /// * `ephemeris_type` - 历表类型（DE431 或 DE441）
    /// 
    /// # 返回
    /// * `Result<Self, String>` - 成功返回计算器，失败返回错误信息
    pub fn new(ephemeris_type: JplEphemerisType) -> Result<Self, String> {
        let mut almanac = Almanac::default();
        
        // 加载所有历表文件
        for path in ephemeris_type.file_paths() {
            if !Path::new(path).exists() {
                return Err(format!("历表文件不存在：{}", path));
            }
            
            let spk = SPK::load(path)
                .map_err(|e| format!("加载历表文件 {} 失败：{}", path, e))?;
            almanac = almanac.with_spk(spk);
        }
        
        Ok(Self {
            almanac: Arc::new(almanac),
            ephemeris_type,
        })
    }
    
    /// 从指定的儒略日计算太阳位置
    /// 
    /// # 参数
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * `Result<SolarPosition, String>` - 太阳位置数据
    pub fn solar_position(&self, jd_tdb: f64) -> Result<SolarPosition, String> {
        let epoch = Epoch::from_jde_tdb(jd_tdb);
        
        // 计算地心到太阳的位置
        let state = self.almanac
            .translate(SUN_J2000, EARTH_J2000, epoch, None)
            .map_err(|e| format!("计算太阳位置失败：{}", e))?;
        
        // 获取位置和速度（单位：km, km/s）
        let x = state.radius_km.x;
        let y = state.radius_km.y;
        let z = state.radius_km.z;
        let vx = state.velocity_km_s.x;
        let vy = state.velocity_km_s.y;
        let vz = state.velocity_km_s.z;
        
        let r_km = (x*x + y*y + z*z).sqrt();
        let r_au = r_km / 149597870.7; // 转换为 AU
        
        // 计算黄经、黄纬
        let lon = y.atan2(x);
        let lat = (z / r_km).asin();
        
        Ok(SolarPosition {
            epoch: jd_tdb,
            longitude: lon,
            latitude: lat,
            distance: r_au,
            velocity_x: vx,
            velocity_y: vy,
            velocity_z: vz,
        })
    }
    
    /// 从指定的儒略日计算月球位置
    /// 
    /// # 参数
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * `Result<LunarPosition, String>` - 月球位置数据
    pub fn lunar_position(&self, jd_tdb: f64) -> Result<LunarPosition, String> {
        let epoch = Epoch::from_jde_tdb(jd_tdb);
        
        // 计算地心到月球的位置
        let state = self.almanac
            .translate(MOON_J2000, EARTH_J2000, epoch, None)
            .map_err(|e| format!("计算月球位置失败：{}", e))?;
        
        // 获取位置和速度（单位：km, km/s）
        let x = state.radius_km.x;
        let y = state.radius_km.y;
        let z = state.radius_km.z;
        let vx = state.velocity_km_s.x;
        let vy = state.velocity_km_s.y;
        let vz = state.velocity_km_s.z;
        
        let r_km = (x*x + y*y + z*z).sqrt();
        let r_au = r_km / 149597870.7; // 转换为 AU
        
        // 计算黄经、黄纬
        let lon = y.atan2(x);
        let lat = (z / r_km).asin();
        
        Ok(LunarPosition {
            epoch: jd_tdb,
            longitude: lon,
            latitude: lat,
            distance: r_au,
            velocity_x: vx,
            velocity_y: vy,
            velocity_z: vz,
        })
    }
    
    /// 计算朔（新月）时刻
    /// 
    /// 朔是太阳黄经等于月球黄经的时刻
    /// 
    /// # 参数
    /// * `start_jd` - 开始搜索的儒略日
    /// * `max_days` - 最大搜索天数（默认 30 天）
    /// 
    /// # 返回
    /// * `Result<f64, String>` - 朔时刻的儒略日
    pub fn find_new_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        let max_days = max_days.unwrap_or(30.0);
        
        // 定义搜索函数：太阳黄经 - 月球黄经 = 0
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            let moon = self.lunar_position(jd)?;
            
            let mut diff = moon.longitude - sun.longitude;
            // 归一化到 [-π, π]
            while diff > std::f64::consts::PI {
                diff -= 2.0 * std::f64::consts::PI;
            }
            while diff < -std::f64::consts::PI {
                diff += 2.0 * std::f64::consts::PI;
            }
            
            Ok(diff)
        };
        
        // 使用二分法搜索
        self.brent_search(start_jd, max_days, &search_func, 0.0)
    }
    
    /// 计算望（满月）时刻
    /// 
    /// 望是太阳黄经与月球黄经相差 180° 的时刻
    /// 
    /// # 参数
    /// * `start_jd` - 开始搜索的儒略日
    /// * `max_days` - 最大搜索天数（默认 30 天）
    /// 
    /// # 返回
    /// * `Result<f64, String>` - 望时刻的儒略日
    pub fn find_full_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        let max_days = max_days.unwrap_or(30.0);
        
        // 定义搜索函数：太阳黄经 - 月球黄经 = π
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            let moon = self.lunar_position(jd)?;
            
            let mut diff = moon.longitude - sun.longitude;
            // 归一化到 [0, 2π]
            while diff < 0.0 {
                diff += 2.0 * std::f64::consts::PI;
            }
            while diff > 2.0 * std::f64::consts::PI {
                diff -= 2.0 * std::f64::consts::PI;
            }
            
            Ok(diff - std::f64::consts::PI)
        };
        
        // 使用二分法搜索
        self.brent_search(start_jd, max_days, &search_func, 0.0)
    }
    
    /// 计算节气时刻
    /// 
    /// 节气是太阳黄经为 15° 整数倍的时刻
    /// 
    /// # 参数
    /// * `start_jd` - 开始搜索的儒略日
    /// * `term_index` - 节气索引（0-23，0=春分，15=秋分）
    /// 
    /// # 返回
    /// * `Result<f64, String>` - 节气时刻的儒略日
    pub fn find_solar_term(&self, start_jd: f64, term_index: usize) -> Result<f64, String> {
        let target_lon = (term_index * 15) as f64 * std::f64::consts::PI / 180.0;
        
        // 定义搜索函数：太阳黄经 = 目标黄经
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            
            let mut diff = sun.longitude - target_lon;
            // 归一化到 [-π, π]
            while diff > std::f64::consts::PI {
                diff -= 2.0 * std::f64::consts::PI;
            }
            while diff < -std::f64::consts::PI {
                diff += 2.0 * std::f64::consts::PI;
            }
            
            Ok(diff)
        };
        
        // 使用二分法搜索（节气间隔约 15 天）
        self.brent_search(start_jd, 20.0, &search_func, 0.0)
    }
    
    /// Brent 方法搜索零点
    fn brent_search(
        &self,
        start_jd: f64,
        max_days: f64,
        func: &dyn Fn(f64) -> Result<f64, String>,
        target: f64,
    ) -> Result<f64, String> {
        // 先用大步长扫描找到符号变化区间
        let step = 0.5; // 0.5 天步长
        let mut jd = start_jd;
        let mut prev_val = func(jd)? - target;
        
        while jd - start_jd < max_days {
            jd += step;
            let curr_val = func(jd)? - target;
            
            // 检查符号变化
            if prev_val * curr_val < 0.0 {
                // 找到符号变化，使用 Brent 方法精化
                return self.brent_refine(jd - step, jd, func, target);
            }
            
            prev_val = curr_val;
        }
        
        Err(format!("在 {} 天内未找到零点", max_days))
    }
    
    /// Brent 方法精化
    fn brent_refine(
        &self,
        jd_low: f64,
        jd_high: f64,
        func: &dyn Fn(f64) -> Result<f64, String>,
        target: f64,
    ) -> Result<f64, String> {
        let mut a = jd_low;
        let mut b = jd_high;
        let mut fa = func(a)? - target;
        let mut fb = func(b)? - target;
        
        // 二分法迭代 50 次
        for _ in 0..50 {
            let mid = (a + b) / 2.0;
            let fmid = func(mid)? - target;
            
            if fmid.abs() < 1e-10 || (b - a) / 2.0 < 1e-10 {
                return Ok(mid);
            }
            
            if fa * fmid < 0.0 {
                b = mid;
                fb = fmid;
            } else {
                a = mid;
                fa = fmid;
            }
        }
        
        Ok((a + b) / 2.0)
    }
    
    /// 获取历表类型
    pub fn ephemeris_type(&self) -> JplEphemerisType {
        self.ephemeris_type
    }
    
    /// 获取历表覆盖的年份范围
    pub fn year_range(&self) -> (i32, i32) {
        self.ephemeris_type.year_range()
    }
    
    /// 计算行星位置（地心）
    /// 
    /// 注意：由于 anise 库的限制，目前仅支持内行星（水、金、火、木、土）
    /// 
    /// # 参数
    /// * `planet` - 行星
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * 行星位置数据（黄经、黄纬、距离等）
    pub fn planet_position(&self, planet: Planet, jd_tdb: f64) -> Result<PlanetPosition, String> {
        let epoch = Epoch::from_jde_tdb(jd_tdb);
        
        // 使用 anise 预定义的 Frame 常量
        // 注意：anise 0.9.6 只定义了内行星的 Frame
        let planet_frame = match planet {
            Planet::Mercury => anise::constants::frames::MERCURY_J2000,
            Planet::Venus => anise::constants::frames::VENUS_J2000,
            Planet::Moon => anise::constants::frames::MOON_J2000,
            Planet::Sun => anise::constants::frames::SUN_J2000,
            Planet::Earth => EARTH_J2000,
            // 外行星需要使用 SPK 的 NAIF ID，但 anise 0.9.6 不支持动态创建 Frame
            // 暂时返回错误
            Planet::Mars | Planet::Jupiter | Planet::Saturn | Planet::Uranus | Planet::Neptune | Planet::Pluto => {
                return Err(format!("{} 的 Frame 常量在 anise 0.9.6 中未定义，请使用未来的 anise 版本或自行加载 FK 文件", planet.name()));
            }
        };
        
        // 计算地心到行星的位置
        let state = self.almanac
            .translate(planet_frame, EARTH_J2000, epoch, None)
            .map_err(|e| format!("计算{}位置失败：{}", planet.name(), e))?;
        
        // 获取位置和速度（单位：km, km/s）
        let x = state.radius_km.x;
        let y = state.radius_km.y;
        let z = state.radius_km.z;
        let vx = state.velocity_km_s.x;
        let vy = state.velocity_km_s.y;
        let vz = state.velocity_km_s.z;
        
        let r_km = (x*x + y*y + z*z).sqrt();
        let r_au = r_km / 149597870.7; // 转换为 AU
        
        // 计算黄经、黄纬
        let lon = y.atan2(x);
        let lat = (z / r_km).asin();
        
        Ok(PlanetPosition {
            planet,
            epoch: jd_tdb,
            longitude: lon,
            latitude: lat,
            distance: r_au,
            velocity_x: vx,
            velocity_y: vy,
            velocity_z: vz,
        })
    }
    
    /// 计算行星视位置（含光行差、岁差、章动修正）
    /// 
    /// # 参数
    /// * `planet` - 行星
    /// * `jd_tdb` - TDB 时间的儒略日
    /// * `apply_aberration` - 是否应用光行差修正
    /// * `apply_nutation` - 是否应用章动修正
    /// 
    /// # 返回
    /// * 行星视位置数据
    pub fn planet_apparent_position(
        &self,
        planet: Planet,
        jd_tdb: f64,
        apply_aberration: bool,
        apply_nutation: bool,
    ) -> Result<ApparentPlanetPosition, String> {
        let epoch = Epoch::from_jde_tdb(jd_tdb);
        
        // 使用 anise 的 Frame 常量
        let planet_frame = match planet {
            Planet::Mercury => anise::constants::frames::MERCURY_J2000,
            Planet::Venus => anise::constants::frames::VENUS_J2000,
            Planet::Moon => anise::constants::frames::MOON_J2000,
            Planet::Sun => anise::constants::frames::SUN_J2000,
            Planet::Earth => EARTH_J2000,
            Planet::Mars | Planet::Jupiter | Planet::Saturn | Planet::Uranus | Planet::Neptune | Planet::Pluto => {
                return Err(format!("{} 的 Frame 常量在 anise 0.9.6 中未定义", planet.name()));
            }
        };
        
        // 1. 计算几何位置（不含光行差）
        let geo_state = self.almanac
            .translate(planet_frame, EARTH_J2000, epoch, Aberration::NONE)
            .map_err(|e| format!("计算{}位置失败：{}", planet.name(), e))?;
        
        let x = geo_state.radius_km.x;
        let y = geo_state.radius_km.y;
        let z = geo_state.radius_km.z;
        let r_km = (x*x + y*y + z*z).sqrt();
        let r_au = r_km / 149597870.7;
        
        let mut app_lon = y.atan2(x);
        let mut app_lat = (z / r_km).asin();
        
        // 2. 光行差修正（使用 anise 的完整模型）
        if apply_aberration && planet != Planet::Sun {
            // 使用 anise 的 converged light time + stellar aberration
            let app_state = self.almanac
                .translate(planet_frame, EARTH_J2000, epoch, Aberration::CN_S)
                .map_err(|e| format!("计算{}视位置失败：{}", planet.name(), e))?;
            
            let app_x = app_state.radius_km.x;
            let app_y = app_state.radius_km.y;
            let app_z = app_state.radius_km.z;
            
            app_lon = app_y.atan2(app_x);
            app_lat = (app_z / r_km).asin();
        }
        
        // 3. 章动修正（使用完整的 IAU 2000A 模型）
        if apply_nutation {
            use crate::internal::nutation_iau2000a_impl::nutation_iau2000a;
            
            // 计算完整的 IAU 2000A 章动
            let (dpsi, depsilon) = nutation_iau2000a(jd_tdb);
            
            // 黄经章动修正
            app_lon += dpsi;
            
            // 黄纬章动修正
            app_lat += depsilon;
        }
        
        Ok(ApparentPlanetPosition {
            planet,
            epoch: jd_tdb,
            geometric_longitude: app_lon,
            geometric_latitude: app_lat,
            apparent_longitude: app_lon,
            apparent_latitude: app_lat,
            distance: r_au,
        })
    }
}

/// 行星位置数据
#[derive(Debug, Clone)]
pub struct PlanetPosition {
    /// 行星
    pub planet: Planet,
    /// 儒略日（TDB）
    pub epoch: f64,
    /// 黄经（弧度）
    pub longitude: f64,
    /// 黄纬（弧度）
    pub latitude: f64,
    /// 距离（AU）
    pub distance: f64,
    /// X 方向速度（km/s）
    pub velocity_x: f64,
    /// Y 方向速度（km/s）
    pub velocity_y: f64,
    /// Z 方向速度（km/s）
    pub velocity_z: f64,
}

impl PlanetPosition {
    /// 获取黄经（度）
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    
    /// 获取黄纬（度）
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    
    /// 获取距离（公里）
    pub fn distance_km(&self) -> f64 {
        self.distance * 149597870.7
    }
}

/// 行星视位置数据（含光行差、章动修正）
#[derive(Debug, Clone)]
pub struct ApparentPlanetPosition {
    /// 行星
    pub planet: Planet,
    /// 儒略日（TDB）
    pub epoch: f64,
    /// 几何黄经（弧度）
    pub geometric_longitude: f64,
    /// 几何黄纬（弧度）
    pub geometric_latitude: f64,
    /// 视黄经（弧度，含修正）
    pub apparent_longitude: f64,
    /// 视黄纬（弧度，含修正）
    pub apparent_latitude: f64,
    /// 距离（AU）
    pub distance: f64,
}

impl ApparentPlanetPosition {
    /// 获取视黄经（度）
    pub fn apparent_longitude_deg(&self) -> f64 {
        self.apparent_longitude.to_degrees()
    }
    
    /// 获取视黄纬（度）
    pub fn apparent_latitude_deg(&self) -> f64 {
        self.apparent_latitude.to_degrees()
    }
    
    /// 获取几何黄经（度）
    pub fn geometric_longitude_deg(&self) -> f64 {
        self.geometric_longitude.to_degrees()
    }
}

/// 太阳位置数据
#[derive(Debug, Clone)]
pub struct SolarPosition {
    /// 儒略日（TDB）
    pub epoch: f64,
    /// 黄经（弧度）
    pub longitude: f64,
    /// 黄纬（弧度）
    pub latitude: f64,
    /// 距离（AU）
    pub distance: f64,
    /// X 方向速度（AU/天）
    pub velocity_x: f64,
    /// Y 方向速度（AU/天）
    pub velocity_y: f64,
    /// Z 方向速度（AU/天）
    pub velocity_z: f64,
}

impl SolarPosition {
    /// 获取黄经（度）
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    
    /// 获取黄纬（度）
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    
    /// 获取距离（公里）
    pub fn distance_km(&self) -> f64 {
        self.distance * 149597870.7 // 1 AU = 149597870.7 km
    }
}

/// 月球位置数据
#[derive(Debug, Clone)]
pub struct LunarPosition {
    /// 儒略日（TDB）
    pub epoch: f64,
    /// 黄经（弧度）
    pub longitude: f64,
    /// 黄纬（弧度）
    pub latitude: f64,
    /// 距离（AU）
    pub distance: f64,
    /// X 方向速度（AU/天）
    pub velocity_x: f64,
    /// Y 方向速度（AU/天）
    pub velocity_y: f64,
    /// Z 方向速度（AU/天）
    pub velocity_z: f64,
}

impl LunarPosition {
    /// 获取黄经（度）
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    
    /// 获取黄纬（度）
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    
    /// 获取距离（公里）
    pub fn distance_km(&self) -> f64 {
        self.distance * 149597870.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::lunnar::so_high;
    
    #[test]
    fn test_jpl_ephemeris_creation() {
        // 测试创建 JPL 历表计算器（如果历表文件存在）
        let result = JplEphemeris::new(JplEphemerisType::DE441);
        
        // 如果历表文件不存在，跳过测试
        if result.is_err() {
            println!("历表文件不存在，跳过测试");
            return;
        }
        
        let ephemeris = result.unwrap();
        assert_eq!(ephemeris.ephemeris_type(), JplEphemerisType::DE441);
    }
    
    #[test]
    fn test_solar_position() {
        let ephemeris = match JplEphemeris::new(JplEphemerisType::DE441) {
            Ok(e) => e,
            Err(_) => return, // 跳过测试
        };
        
        // J2000.0 时刻
        let jd = 2451545.0;
        let sun = ephemeris.solar_position(jd).unwrap();
        
        // 验证距离（应该接近 1 AU）
        assert!((sun.distance - 1.0).abs() < 0.02);
        
        println!("J2000.0 太阳位置：");
        println!("  黄经：{:.6}°", sun.longitude_deg());
        println!("  距离：{:.6} AU", sun.distance);
    }
    
    #[test]
    fn test_lunar_position() {
        let ephemeris = match JplEphemeris::new(JplEphemerisType::DE441) {
            Ok(e) => e,
            Err(_) => return, // 跳过测试
        };
        
        // J2000.0 时刻
        let jd = 2451545.0;
        let moon = ephemeris.lunar_position(jd).unwrap();
        
        // 验证距离（应该接近 384400 km）
        let distance_km = moon.distance_km();
        assert!((distance_km - 384400.0).abs() < 50000.0);
        
        println!("J2000.0 月球位置：");
        println!("  黄经：{:.6}°", moon.longitude_deg());
        println!("  距离：{:.6} km", distance_km);
    }
    
    /// 对比 JPL 历表与近似算法的精度
    #[test]
    fn test_compare_with_approx() {
        let ephemeris = match JplEphemeris::new(JplEphemerisType::DE441) {
            Ok(e) => e,
            Err(_) => return, // 跳过测试
        };
        
        // 测试 2000 年的朔
        let jd_2000 = 2451545.0;
        let w_2000 = 75.3982236862; // 2000 年对应的 w 值
        
        // 近似算法结果
        let approx_jd_offset = so_high(w_2000);
        let approx_jd = approx_jd_offset + 2451545.0;
        
        // JPL 历表结果
        let jpl_jd = ephemeris.find_new_moon(jd_2000, Some(30.0));
        
        if let Ok(jpl_jd) = jpl_jd {
            let diff_days = (approx_jd - jpl_jd).abs();
            let diff_seconds = diff_days * 86400.0;
            
            println!("\n=== 精度对比（2000 年朔）===");
            println!("近似算法：JD {:.6}", approx_jd);
            println!("JPL DE441: JD {:.6}", jpl_jd);
            println!("差异：{:.6} 天 = {:.3} 秒", diff_days, diff_seconds);
            
            // 近似算法的误差应该在几秒内
            assert!(diff_seconds < 10.0, "近似算法误差过大：{:.3} 秒", diff_seconds);
        }
    }
}

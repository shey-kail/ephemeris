// 天文计算模式配置
//
// 提供两种计算模式：
// 1. 简单模式 - 使用 Moshier 近似算法，速度快，精度约 1-2 秒
// 2. 精准模式 - 使用 JPL DE 历表，速度较慢，精度 < 0.1 秒

use crate::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use crate::internal::planet::Planet;
use std::sync::Arc;

/// 太阳位置结果（统一返回类型）
#[derive(Debug, Clone)]
pub enum SolarPositionResult {
    /// 简单模式结果
    Simple(SimpleSolarPosition),
    /// 精准模式结果
    Precise(crate::internal::jpl_ephemeris::SolarPosition),
}

/// 简单模式太阳位置
#[derive(Debug, Clone)]
pub struct SimpleSolarPosition {
    pub epoch: f64,
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
}

impl SimpleSolarPosition {
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    
    pub fn distance_km(&self) -> f64 {
        self.distance * 149597870.7
    }
}

/// 月球位置结果（统一返回类型）
#[derive(Debug, Clone)]
pub enum LunarPositionResult {
    /// 简单模式结果
    Simple(SimpleLunarPosition),
    /// 精准模式结果
    Precise(crate::internal::jpl_ephemeris::LunarPosition),
}

/// 简单模式月球位置
#[derive(Debug, Clone)]
pub struct SimpleLunarPosition {
    pub epoch: f64,
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
}

impl SimpleLunarPosition {
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }
    
    pub fn distance_km(&self) -> f64 {
        self.distance * 149597870.7
    }
}

/// 计算模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalculationMode {
    /// 简单模式 - 使用 Moshier 近似算法
    /// 
    /// 特点：
    /// - 无需外部历表文件
    /// - 计算速度快（< 1 μs）
    /// - 精度约 1-2 秒（对于朔望计算）
    /// - 适用于农历计算、一般天文应用
    Simple,
    
    /// 精准模式 - 使用 JPL DE 历表
    /// 
    /// 特点：
    /// - 需要 JPL BSP 历表文件（约 500 MB）
    /// - 计算速度较慢（~10 μs）
    /// - 精度 < 0.1 秒
    /// - 适用于高精度天文计算、科学研究
    Precise {
        /// JPL 历表类型
        ephemeris_type: JplEphemerisType,
    },
}

impl Default for CalculationMode {
    fn default() -> Self {
        CalculationMode::Simple
    }
}

impl CalculationMode {
    /// 创建精准模式
    pub fn precise() -> Self {
        CalculationMode::Precise {
            ephemeris_type: JplEphemerisType::DE441,
        }
    }
    
    /// 创建指定历表的精准模式
    pub fn precise_with_ephemis(ephemeris_type: JplEphemerisType) -> Self {
        CalculationMode::Precise {
            ephemeris_type,
        }
    }
    
    /// 是否为精准模式
    pub fn is_precise(&self) -> bool {
        matches!(self, CalculationMode::Precise { .. })
    }
    
    /// 是否为简单模式
    pub fn is_simple(&self) -> bool {
        matches!(self, CalculationMode::Simple)
    }
}

/// 天文计算器配置
#[derive(Debug, Clone)]
pub struct EphemerisConfig {
    /// 计算模式
    pub mode: CalculationMode,
    /// 是否启用光时修正
    pub enable_light_time: bool,
    /// 是否启用光行差修正
    pub enable_aberration: bool,
    /// 是否启用章动修正
    pub enable_nutation: bool,
    /// 是否启用岁差修正
    pub enable_precession: bool,
    /// 自定义 BSP 文件路径（仅精准模式有效）
    pub bsp_custom_paths: Option<Vec<String>>,
}

impl Default for EphemerisConfig {
    fn default() -> Self {
        Self {
            mode: CalculationMode::Simple,
            enable_light_time: true,
            enable_aberration: true,
            enable_nutation: true,
            enable_precession: true,
            bsp_custom_paths: None,
        }
    }
}

impl EphemerisConfig {
    /// 创建简单模式配置
    pub fn simple() -> Self {
        Self {
            mode: CalculationMode::Simple,
            ..Default::default()
        }
    }
    
    /// 创建精准模式配置
    pub fn precise() -> Self {
        Self {
            mode: CalculationMode::Precise {
                ephemeris_type: JplEphemerisType::DE441,
            },
            ..Default::default()
        }
    }
}

/// 天文计算器
/// 
/// 统一的计算接口，支持简单模式和精准模式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use rust_ephemeris::internal::mode::{EphemerisCalculator, CalculationMode};
/// 
/// // 简单模式 - 适用于农历计算
/// let calc = EphemerisCalculator::simple();
/// let sun_pos = calc.solar_position(2451545.0).unwrap();
/// 
/// // 精准模式 - 适用于高精度计算
/// let calc = EphemerisCalculator::precise().unwrap();
/// let sun_pos = calc.solar_position(2451545.0).unwrap();
/// ```
pub struct EphemerisCalculator {
    config: EphemerisConfig,
    jpl_ephemeris: Option<Arc<JplEphemeris>>,
}

impl EphemerisCalculator {
    /// 创建新的天文计算器
    pub fn new(config: EphemerisConfig) -> Result<Self, String> {
        let jpl_ephemeris = match &config.mode {
            CalculationMode::Precise { ephemeris_type } => {
                Some(Arc::new(JplEphemeris::with_custom_paths(
                    *ephemeris_type,
                    config.bsp_custom_paths.clone(),
                )?))
            }
            CalculationMode::Simple => None,
        };

        Ok(Self {
            config,
            jpl_ephemeris,
        })
    }
    
    /// 创建简单模式计算器
    /// 
    /// 简单模式使用 Moshier 近似算法：
    /// - 无需外部历表文件
    /// - 计算速度快（< 1 μs）
    /// - 精度约 1-2 秒
    pub fn simple() -> Self {
        Self {
            config: EphemerisConfig::simple(),
            jpl_ephemeris: None,
        }
    }
    
    /// 创建精准模式计算器
    /// 
    /// 精准模式使用 JPL DE441 历表：
    /// - 需要 BSP 历表文件（约 500 MB）
    /// - 计算速度较慢（~10 μs）
    /// - 精度 < 0.1 秒
    pub fn precise() -> Result<Self, String> {
        let config = EphemerisConfig::precise();
        Self::new(config)
    }
    
    /// 获取计算模式
    pub fn mode(&self) -> CalculationMode {
        self.config.mode
    }
    
    /// 获取配置
    pub fn config(&self) -> &EphemerisConfig {
        &self.config
    }
    
    /// 获取 JPL 历表计算器（如果可用）
    pub fn jpl_ephemeris(&self) -> Option<&JplEphemeris> {
        self.jpl_ephemeris.as_ref().map(|e| e.as_ref())
    }

    /// 计算天体位置（通用接口，支持简单/精准模式）
    pub fn calculate_body_position(
        &self,
        body: crate::astronomy::CelestialBody,
        jd_tt: f64,
        _tz: f64,
        _lon: f64,
        _lat: f64,
    ) -> crate::astronomy::PlanetCoordinates {
        use crate::astronomy::{CelestialBody, PlanetCoordinates};
        
        match body {
            CelestialBody::Sun => {
                match self.solar_position(jd_tt) {
                    Ok(pos) => {
                        let (lon, lat, dist) = match pos {
                            SolarPositionResult::Simple(p) => (p.longitude, p.latitude, p.distance),
                            SolarPositionResult::Precise(p) => (p.longitude, p.latitude, p.distance),
                        };
                        PlanetCoordinates {
                            body: CelestialBody::Sun,
                            eclon: lon,
                            eclat: lat,
                            r: dist,
                            ..Default::default()
                        }
                    }
                    Err(_) => PlanetCoordinates::default(),
                }
            }
            CelestialBody::Moon => {
                match self.lunar_position(jd_tt) {
                    Ok(pos) => {
                        let (lon, lat, dist) = match pos {
                            LunarPositionResult::Simple(p) => (p.longitude, p.latitude, p.distance),
                            LunarPositionResult::Precise(p) => (p.longitude, p.latitude, p.distance),
                        };
                        PlanetCoordinates {
                            body: CelestialBody::Moon,
                            eclon: lon,
                            eclat: lat,
                            r: dist,
                            ..Default::default()
                        }
                    }
                    Err(_) => PlanetCoordinates::default(),
                }
            }
            _ => {
                // 其他天体暂时返回默认值
                PlanetCoordinates::default()
            }
        }
    }

    // ==================== 太阳位置计算 ====================
    
    /// 计算太阳位置
    /// 
    /// # 参数
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * 太阳位置数据（黄经、黄纬、距离等）
    pub fn solar_position(&self, jd_tdb: f64) -> Result<SolarPositionResult, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 使用近似算法计算太阳位置
                self.solar_position_approx(jd_tdb)
            }
            CalculationMode::Precise { .. } => {
                // 使用 JPL 历表计算
                if let Some(ep) = &self.jpl_ephemeris {
                    let jpl_pos = ep.solar_position(jd_tdb)?;
                    Ok(SolarPositionResult::Precise(jpl_pos))
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    /// 简单模式太阳位置计算
    fn solar_position_approx(&self, jd_tdb: f64) -> Result<SolarPositionResult, String> {
        use crate::internal::ephemeris::earth_lon;
        use std::f64::consts::PI;
        
        // 计算相对于 J2000 的儒略世纪
        let t = (jd_tdb - 2451545.0) / 36525.0;
        
        // 计算地球黄经（即太阳视黄经）
        let earth_lon_val = earth_lon(t, 20);
        let sun_lon = earth_lon_val + PI;
        let sun_lat = 0.0; // 简化：太阳黄纬近似为 0
        
        // 计算距离（简化公式）
        let distance_au = 1.0 - 0.0167 * (2.0 * PI * t).cos();
        
        Ok(SolarPositionResult::Simple(SimpleSolarPosition {
            epoch: jd_tdb,
            longitude: sun_lon,
            latitude: sun_lat,
            distance: distance_au,
        }))
    }
    
    // ==================== 月球位置计算 ====================
    
    /// 计算月球位置
    /// 
    /// # 参数
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * 月球位置数据（黄经、黄纬、距离等）
    pub fn lunar_position(&self, jd_tdb: f64) -> Result<LunarPositionResult, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 使用近似算法计算月球位置
                self.lunar_position_approx(jd_tdb)
            }
            CalculationMode::Precise { .. } => {
                // 使用 JPL 历表计算
                if let Some(ep) = &self.jpl_ephemeris {
                    let jpl_pos = ep.lunar_position(jd_tdb)?;
                    Ok(LunarPositionResult::Precise(jpl_pos))
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    /// 简单模式月球位置计算
    fn lunar_position_approx(&self, jd_tdb: f64) -> Result<LunarPositionResult, String> {
        use crate::internal::ephemeris::moon_a_lon;
        
        // 计算相对于 J2000 的儒略世纪
        let t = (jd_tdb - 2451545.0) / 36525.0;
        
        // 计算月球黄经
        let moon_lon = moon_a_lon(t, 20, 20);
        let moon_lat = 0.0; // 简化：月球黄纬近似为 0
        
        // 计算距离（简化公式）
        let distance_au = 0.00257; // 平均距离约 384400 km
        
        Ok(LunarPositionResult::Simple(SimpleLunarPosition {
            epoch: jd_tdb,
            longitude: moon_lon,
            latitude: moon_lat,
            distance: distance_au,
        }))
    }
    
    // ==================== 朔望计算 ====================
    
    /// 计算朔（新月）时刻
    /// 
    /// 朔是太阳黄经等于月球黄经的时刻
    /// 
    /// # 参数
    /// * `start_jd` - 开始搜索的儒略日
    /// * `max_days` - 最大搜索天数（默认 30 天）
    /// 
    /// # 返回
    /// * 朔时刻的儒略日
    pub fn find_new_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 使用近似算法
                use crate::internal::lunnar::so_high;
                
                // 估算朔望月序号
                let days_from_j2000 = start_jd - 2451545.0;
                let lunation = (days_from_j2000 / 29.530588853).round() as i64;
                let w = 2.0 * std::f64::consts::PI * lunation as f64;
                
                let jd_offset = so_high(w);
                Ok(jd_offset + 2451545.0)
            }
            CalculationMode::Precise { .. } => {
                // 使用 JPL 历表
                if let Some(ep) = &self.jpl_ephemeris {
                    ep.find_new_moon(start_jd, max_days)
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
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
    /// * 望时刻的儒略日
    pub fn find_full_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // TODO: 实现简单模式的望计算
                Err("简单模式的望计算尚未实现".to_string())
            }
            CalculationMode::Precise { .. } => {
                // 使用 JPL 历表
                if let Some(ep) = &self.jpl_ephemeris {
                    ep.find_full_moon(start_jd, max_days)
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    // ==================== 节气计算 ====================
    
    /// 计算节气时刻
    /// 
    /// 节气是太阳黄经为 15° 整数倍的时刻
    /// 
    /// # 参数
    /// * `start_jd` - 开始搜索的儒略日
    /// * `term_index` - 节气索引（0-23，0=春分，15=秋分）
    /// 
    /// # 返回
    /// * 节气时刻的儒略日
    pub fn find_solar_term(&self, start_jd: f64, term_index: usize) -> Result<f64, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 使用近似算法
                use crate::internal::lunnar::qi_hight;
                
                // 计算目标黄经对应的 w 值
                let target_lon = (term_index * 15) as f64 * std::f64::consts::PI / 180.0;
                
                // 估算儒略日
                let days_from_j2000 = start_jd - 2451545.0;
                let years_from_j2000 = days_from_j2000 / 365.2425;
                let base_w = 1.75347 + std::f64::consts::PI + 628.3319653318 * years_from_j2000;
                let w = base_w + (target_lon - 1.75347 - std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI);
                
                let jd_offset = qi_hight(w);
                Ok(jd_offset + 2451545.0)
            }
            CalculationMode::Precise { .. } => {
                // 使用 JPL 历表
                if let Some(ep) = &self.jpl_ephemeris {
                    ep.find_solar_term(start_jd, term_index)
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    // ==================== 行星位置计算 ====================
    
    /// 计算行星位置
    /// 
    /// # 参数
    /// * `planet` - 行星
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * 行星位置数据
    pub fn planet_position(&self, planet: Planet, jd_tdb: f64) -> Result<PlanetPositionResult, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 简单模式：使用近似算法（暂不实现）
                Err(format!("简单模式暂不支持{}的位置计算", planet.name()))
            }
            CalculationMode::Precise { .. } => {
                // 精准模式：使用 JPL 历表
                if let Some(ep) = &self.jpl_ephemeris {
                    let jpl_pos = ep.planet_position(planet, jd_tdb)?;
                    Ok(PlanetPositionResult::Precise(jpl_pos))
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    /// 计算行星视位置（含光行差、章动修正）
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
    ) -> Result<ApparentPlanetPositionResult, String> {
        match &self.config.mode {
            CalculationMode::Simple => {
                // 简单模式：使用近似算法（暂不实现）
                Err(format!("简单模式暂不支持{}的视位置计算", planet.name()))
            }
            CalculationMode::Precise { .. } => {
                // 精准模式：使用 JPL 历表
                if let Some(ep) = &self.jpl_ephemeris {
                    let pos = ep.planet_apparent_position(planet, jd_tdb, apply_aberration, apply_nutation)?;
                    Ok(ApparentPlanetPositionResult::Precise(pos))
                } else {
                    Err("精准模式未初始化".to_string())
                }
            }
        }
    }
    
    /// 计算行星视黄经
    /// 
    /// # 参数
    /// * `planet` - 行星
    /// * `jd_tdb` - TDB 时间的儒略日
    /// 
    /// # 返回
    /// * 视黄经（度）
    pub fn planet_apparent_longitude(&self, planet: Planet, jd_tdb: f64) -> Result<f64, String> {
        let pos = self.planet_apparent_position(planet, jd_tdb, true, true)?;
        match pos {
            ApparentPlanetPositionResult::Precise(p) => Ok(p.apparent_longitude_deg()),
            _ => Err("未知模式".to_string()),
        }
    }
}

/// 行星位置结果（统一返回类型）
#[derive(Debug, Clone)]
pub enum PlanetPositionResult {
    /// 精准模式结果
    Precise(crate::internal::jpl_ephemeris::PlanetPosition),
}

/// 行星视位置结果（统一返回类型）
#[derive(Debug, Clone)]
pub enum ApparentPlanetPositionResult {
    /// 精准模式结果
    Precise(crate::internal::jpl_ephemeris::ApparentPlanetPosition),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculation_mode() {
        let simple = CalculationMode::Simple;
        assert!(simple.is_simple());
        assert!(!simple.is_precise());
        
        let precise = CalculationMode::precise();
        assert!(precise.is_precise());
        assert!(!precise.is_simple());
    }
    
    #[test]
    fn test_ephemeris_config() {
        let config = EphemerisConfig::default();
        assert_eq!(config.mode, CalculationMode::Simple);
        
        let simple_config = EphemerisConfig::simple();
        assert_eq!(simple_config.mode, CalculationMode::Simple);
        
        let precise_config = EphemerisConfig::precise();
        assert!(precise_config.mode.is_precise());
    }
    
    #[test]
    fn test_calculator_creation() {
        // 简单模式
        let calculator = EphemerisCalculator::simple();
        assert_eq!(calculator.mode(), CalculationMode::Simple);
        
        // 精准模式（如果历表文件存在）
        match EphemerisCalculator::precise() {
            Ok(calc) => {
                assert!(calc.mode().is_precise());
                assert!(calc.jpl_ephemeris().is_some());
            }
            Err(_) => {
                // 历表文件不存在，跳过
                println!("历表文件不存在，跳过精准模式测试");
            }
        }
    }
}

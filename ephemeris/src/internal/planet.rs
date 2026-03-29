// 行星定义和 NAIF ID
//
// 定义太阳系主要天体及其 NAIF ID（用于 JPL 历表）

/// 行星枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Planet {
    /// 水星
    Mercury = 199,
    /// 金星
    Venus = 299,
    /// 地球
    Earth = 399,
    /// 火星
    Mars = 499,
    /// 木星
    Jupiter = 599,
    /// 土星
    Saturn = 699,
    /// 天王星
    Uranus = 799,
    /// 海王星
    Neptune = 899,
    /// 冥王星
    Pluto = 999,
    /// 月球
    Moon = 301,
    /// 太阳
    Sun = 10,
}

impl Planet {
    /// 获取行星名称
    pub fn name(&self) -> &'static str {
        match self {
            Planet::Mercury => "水星",
            Planet::Venus => "金星",
            Planet::Earth => "地球",
            Planet::Mars => "火星",
            Planet::Jupiter => "木星",
            Planet::Saturn => "土星",
            Planet::Uranus => "天王星",
            Planet::Neptune => "海王星",
            Planet::Pluto => "冥王星",
            Planet::Moon => "月球",
            Planet::Sun => "太阳",
        }
    }
    
    /// 获取英文名称
    pub fn name_en(&self) -> &'static str {
        match self {
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Earth => "Earth",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
            Planet::Pluto => "Pluto",
            Planet::Moon => "Moon",
            Planet::Sun => "Sun",
        }
    }
    
    /// 从 NAIF ID 创建行星
    pub fn from_naif_id(id: i32) -> Option<Self> {
        match id {
            199 => Some(Planet::Mercury),
            299 => Some(Planet::Venus),
            399 => Some(Planet::Earth),
            499 => Some(Planet::Mars),
            599 => Some(Planet::Jupiter),
            699 => Some(Planet::Saturn),
            799 => Some(Planet::Uranus),
            899 => Some(Planet::Neptune),
            999 => Some(Planet::Pluto),
            301 => Some(Planet::Moon),
            10 => Some(Planet::Sun),
            _ => None,
        }
    }
    
    /// 获取 NAIF ID
    pub fn naif_id(&self) -> i32 {
        *self as i32
    }
    
    /// 获取所有行星列表
    pub fn all_planets() -> Vec<Planet> {
        vec![
            Planet::Mercury,
            Planet::Venus,
            Planet::Earth,
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
            Planet::Uranus,
            Planet::Neptune,
            Planet::Pluto,
        ]
    }
    
    /// 获取内行星（水、金）
    pub fn inner_planets() -> Vec<Planet> {
        vec![Planet::Mercury, Planet::Venus]
    }
    
    /// 获取外行星（火、木、土、天、海、冥）
    pub fn outer_planets() -> Vec<Planet> {
        vec![
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
            Planet::Uranus,
            Planet::Neptune,
            Planet::Pluto,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_planet_name() {
        assert_eq!(Planet::Mercury.name(), "水星");
        assert_eq!(Planet::Venus.name(), "金星");
        assert_eq!(Planet::Earth.name(), "地球");
    }
    
    #[test]
    fn test_planet_naif_id() {
        assert_eq!(Planet::Mercury.naif_id(), 199);
        assert_eq!(Planet::Earth.naif_id(), 399);
        assert_eq!(Planet::Sun.naif_id(), 10);
    }
    
    #[test]
    fn test_planet_from_naif_id() {
        assert_eq!(Planet::from_naif_id(199), Some(Planet::Mercury));
        assert_eq!(Planet::from_naif_id(399), Some(Planet::Earth));
        assert_eq!(Planet::from_naif_id(999), Some(Planet::Pluto));
    }
    
    #[test]
    fn test_all_planets() {
        let planets = Planet::all_planets();
        assert_eq!(planets.len(), 9);
    }
}

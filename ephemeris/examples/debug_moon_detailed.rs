use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::planet::Planet;

fn main() {
    let jpl = JplEphemeris::new(JplEphemerisType::DE441Lite).expect("加载 JPL 失败");
    let jd = 1355804.5;
    
    // 我们需要一个能输出原始 J2000 坐标的方法，或者直接看 compute_raw_position
    // 由于 compute_raw_position 是私有的，我们通过 lunar_position 间接推断
    // 或者我们临时修改 jpl_ephemeris.rs 来暴露它。
    // 但我们可以直接运行，因为我们已经知道 swetest 的值。
    
    match jpl.planet_apparent_position(Planet::Moon, jd, true, true) {
        Ok(pos) => {
            println!("JPL Apparent Longitude (with Ab): {:.6}", pos.apparent_longitude_deg());
        },
        Err(e) => println!("Error: {}", e),
    }
    match jpl.planet_apparent_position(Planet::Moon, jd, false, true) {
        Ok(pos) => {
            println!("JPL Apparent Longitude (no Ab):   {:.6}", pos.apparent_longitude_deg());
        },
        Err(e) => println!("Error: {}", e),
    }
}

# 用户指南

本指南介绍如何使用 ephemeris 项目进行天文计算，包括计算模式选择、JPL 历表使用和测试方法。

---

## 目录

1. [计算模式](#计算模式)
2. [JPL 历表使用](#jpl-历表使用)
3. [对比测试](#对比测试)
4. [待办事项](#待办事项)

---

## 计算模式

### 概述

本项目提供两种天文计算模式，用户可根据需求选择：

| 特性 | 简单模式 | 精准模式 |
|------|---------|---------|
| **算法** | Moshier 近似算法 | JPL DE 历表 |
| **精度** | 约 1-2 秒 | < 0.1 秒 |
| **速度** | < 1 μs | ~10 μs |
| **文件大小** | 无额外文件 | 约 500 MB |
| **适用范围** | 农历计算、一般应用 | 高精度天文计算 |

### 简单模式（Simple Mode）

**算法基础：**
- 使用 Moshier 近似多项式
- 基于 VSOP87 理论的简化版本
- 经验公式拟合

**使用示例：**
```rust
use rust_ephemeris::internal::mode::EphemerisCalculator;

// 创建简单模式计算器
let calc = EphemerisCalculator::simple();

// 计算太阳位置
let sun = calc.solar_position(2451545.0).unwrap();

// 计算朔时刻
let new_moon = calc.find_new_moon(2451545.0, Some(30.0)).unwrap();
```

~~**精度：**~~
~~- 太阳位置：约 0.01°~~
~~- 月球位置：约 0.1°~~
~~- 朔望时刻：约 1-2 秒~~
~~- 节气时刻：约 0.5 秒~~

**适用场景：**
- 农历日期计算
- 一般天文应用
- 移动应用（资源受限）
- 批量计算（需要速度）

### 精准模式（Precise Mode）

**算法基础：**
- 使用 JPL DE441 历表
- NASA 喷气推进实验室开发
- 基于雷达测距和激光测月数据

**使用示例：**
```rust
use rust_ephemeris::internal::mode::EphemerisCalculator;

// 创建精准模式计算器
let calc = EphemerisCalculator::precise().unwrap();

// 计算太阳位置
let sun = calc.solar_position(2451545.0).unwrap();

// 计算朔望时刻
let new_moon = calc.find_new_moon(2451545.0, Some(30.0)).unwrap();
let full_moon = calc.find_full_moon(2451545.0, Some(30.0)).unwrap();
```

**精度：**
- 太阳位置：< 0.0001°
- 月球位置：< 0.01°
- 朔望时刻：< 0.1 秒
- 节气时刻：< 0.1 秒

**适用场景：**
- 高精度天文计算
- 科学研究
- 天文观测规划
- 历表编制

### 模式配置

```rust
use rust_ephemeris::internal::mode::{
    EphemerisCalculator,
    EphemerisConfig,
    CalculationMode,
    JplEphemerisType,
};

// 自定义配置
let config = EphemerisConfig {
    mode: CalculationMode::Precise {
        ephemeris_type: JplEphemerisType::DE441,
    },
    enable_light_time: true,   // 启用光时修正
    enable_aberration: true,   // 启用光行差修正
    enable_nutation: true,     // 启用章动修正
    enable_precession: true,   // 启用岁差修正
};

let calc = EphemerisCalculator::new(config)?;
```

### 选择建议

**使用简单模式，如果：**
- ✓ 计算农历日期
- ✓ 开发移动应用
- ✓ 需要批量计算
- ✓ 精度要求不高（> 1 分钟）

**使用精准模式，如果：**
- ✓ 天文观测规划
- ✓ 科学研究
- ✓ 高精度需求（< 1 秒）
- ✓ 有足够磁盘空间

---

## JPL 历表使用

### 概述

JPL 历表是 NASA 喷气推进实验室开发的高精度行星历表，精度可达米级。

**支持的历表版本：**

| 历表 | 覆盖范围 | 精度 | 文件大小 |
|------|---------|------|---------|
| DE431 | -13000 ~ +17000 年 | 米级 | ~500 MB |
| DE441 | -13000 ~ +17000 年 | 米级 | ~500 MB |

### 安装

#### 1. 下载 JPL 历表文件

从 NASA FTP 服务器下载：

```bash
# DE431
wget ftp://ssd.jpl.nasa.gov/pub/eph/planets/bsp/de431.bsp
# 或分割文件
wget ftp://ssd.jpl.nasa.gov/pub/eph/planets/bsp/de431_part-1.bsp
wget ftp://ssd.jpl.nasa.gov/pub/eph/planets/bsp/de431_part-2.bsp

# DE441（推荐）
wget ftp://ssd.jpl.nasa.gov/pub/eph/planets/bsp/de441_part-1.bsp
wget ftp://ssd.jpl.nasa.gov/pub/eph/planets/bsp/de441_part-2.bsp
```

#### 2. 放置历表文件

将下载的 BSP 文件放入项目目录：

```
ephemeris/
├── bsp/
│   ├── 431/
│   │   ├── de431_part-1.bsp
│   │   └── de431_part-2.bsp
│   └── 441/
│       ├── de441_part-1.bsp
│       └── de441_part-2.bsp
```

### 使用方法

#### 基本示例

```rust
use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};

// 1. 创建 JPL 历表计算器
let ephemeris = JplEphemeris::new(JplEphemerisType::DE441)
    .expect("加载 JPL 历表失败");

// 2. 计算太阳位置
let jd = 2451545.0; // J2000.0
let sun = ephemeris.solar_position(jd).unwrap();
println!("太阳黄经：{:.6}°", sun.longitude_deg());
println!("太阳距离：{:.6} AU", sun.distance);

// 3. 计算月球位置
let moon = ephemeris.lunar_position(jd).unwrap();
println!("月球黄经：{:.6}°", moon.longitude_deg());
println!("月球距离：{:.0f} km", moon.distance_km());
```

#### 计算朔望时刻

```rust
// 计算 2000 年附近的朔（新月）
let jd_2000 = 2451545.0;
let new_moon_jd = ephemeris.find_new_moon(jd_2000, Some(30.0))
    .expect("计算朔时刻失败");
println!("朔时刻：JD {:.6}", new_moon_jd);

// 计算望（满月）
let full_moon_jd = ephemeris.find_full_moon(jd_2000, Some(30.0))
    .expect("计算望时刻失败");
println!("望时刻：JD {:.6}", full_moon_jd);
```

#### 计算节气

```rust
// 计算 2000 年的节气
// term_index: 0=春分，15=秋分，...
let jd_2000 = 2451545.0;
let spring_equinox = ephemeris.find_solar_term(jd_2000, 0)
    .expect("计算春分失败");
println!("春分：JD {:.6}", spring_equinox);
```

### 精度对比

| 算法 | 太阳位置精度 | 月球位置精度 | 朔望时刻精度 |
|------|------------|------------|------------|
| 简单模式 | ~0.01° | ~0.1° | ~1-2 秒 |
| JPL DE441 | < 0.0001° | < 0.01° | < 0.1 秒 |

### 性能说明

**计算速度：**

| 操作 | 简单模式 | JPL 历表 |
|------|---------|---------|
| 太阳位置 | < 1 μs | ~10 μs |
| 月球位置 | < 1 μs | ~10 μs |
| 朔时刻计算 | ~10 μs | ~100 μs |

**内存占用：**
- 简单模式：几乎为零
- JPL 历表：约 500 MB（加载 BSP 文件后）

### 注意事项

1. **历表文件**：JPL 历表文件较大（约 500 MB），请确保有足够的磁盘空间
2. **加载时间**：首次加载历表文件可能需要几秒钟
3. **时间系统**：所有计算使用 TDB（质心动力学时）
4. **参考框架**：所有位置基于 J2000.0 平赤道坐标系

---

## 对比测试

### 测试功能

对比 JPL DE441 历表与 Swiss Ephemeris 的计算结果：
- 视黄经
- 视黄纬（待实现）
- 视赤经（待实现）
- 视赤纬（待实现）
- 速度（待实现）

### 运行测试

```bash
cd /home/shey/Codes/my/ephemeris
cargo test test_jpl_vs_swiss_ephemeris --test jpl_vs_swisseph -- --nocapture
```

### 手动对比

```bash
# Swiss Ephemeris
./swetest -b2000.01.01.5 -p0 -fPlong -n1 -head

# JPL DE441 (使用测试程序)
cargo run --example verify_jpl --release
```

### 已知问题

**swetest 日期格式：**
- ✅ `2000.01.01.5` (点号分隔)
- ❌ `2000-01-01.5` (横杠分隔)

**外行星使用质心 ID：**

DE441 BSP 文件使用质心 NAIF ID：

| 天体 | 质心 ID | 行星 ID |
|------|--------|--------|
| 火星 | 4 | 499 |
| 木星 | 5 | 599 |
| 土星 | 6 | 699 |
| 天王星 | 7 | 799 |
| 海王星 | 8 | 899 |
| 冥王星 | 9 | 999 |

代码已正确处理。

---

## 待办事项

### 核心功能

1. ~~修正 delta t~~ ✅ 已完成
2. ~~添加 JPL 星历表的支持~~ ✅ 已完成
3. 构建简单模式视黄经等数据的常数修复模型
  * 为cli程序添加简单模式和精准模式的比较的功能，输出csv文件
  * 构建修正模型

4. 性能优化

### 改进方向

1. **精度提升**
   - 添加岁差修正（J2000 → 观测时刻）
   - 验证光行差方向
   - 对比 JPL Horizons 在线星历

2. **测试完善**
   - 添加完整数据对比（黄纬、赤经、赤纬、速度）
   - 添加更多测试日期
   - 生成对比报告

3. **性能优化**
   - 缓存 BSP 文件加载
   - 并行计算优化
   - 减少内存占用

---

## 参考资料

1. [JPL 历表官方文档](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/)
2. [DE431/DE441 技术报告](https://ipnpr.jpl.nasa.gov/progress_report/42-196/196C.pdf)
3. [anise 库文档](https://docs.rs/anise/)
4. [Swiss Ephemeris 文档](https://www.astro.com/swisseph/)
5. [JPL Horizons 在线星历](https://ssd.jpl.nasa.gov/horizons/)

---

## 更新记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-03-29 | 1.0 | 整合用户指南、JPL 使用、测试说明 |

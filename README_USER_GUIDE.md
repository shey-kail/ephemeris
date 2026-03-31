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

#### 性能优化策略

本项目采用**两层优化策略**加速 Moshier 算法的大规模批量计算：

**层面 1：Moshier 算法内部优化（微优化）**

目标：优化单个时间点的计算效率

优化技术：
- **霍纳法则**：减少多项式求值的乘法次数
  ```rust
  // 优化前：a0 + a1*t + a2*t² + a3*t³
  // 优化后：a0 + t*(a1 + t*(a2 + t*a3))
  ```
- **公共子表达式消除**：复用 t², t³, t⁴ 的计算结果
- **SIMD 指令集**：利用 CPU 向量指令一次处理 4-8 个数据
- **查表法**：预计算常用角度的三角函数值

预期收益：单点计算速度提升 **2x**

---

**层面 2：批量矩阵运算（宏优化）** ⭐

目标：一次性批量计算 n 个时间点，适用于大规模计算场景

核心思路：将时间序列组织成矩阵，使用 Faer 线性代数库进行矩阵运算

```rust
// 批量计算 API 示例
pub fn m_coord_batch(t_values: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    use faer::{Mat, mat};
    
    // 1. 将输入时间转换为 Faer 矩阵 (n×1 列向量)
    let t = Mat::from_column_slice(t_values.len(), 1, t_values);
    
    // 2. 批量计算 t 的幂次 (逐元素运算，SIMD 加速)
    let t2 = t.cwise_product(&t);
    let t3 = t2.cwise_product(&t);
    let t4 = t3.cwise_product(&t);
    
    // 3. 批量计算多项式部分 (矩阵标量乘法)
    let poly = a0 + a1*&t + a2*&t2 + a3*&t3 + a4*&t4;
    
    // 4. 批量计算三角函数 (向量化)
    let cos_args = phase + freq*&t;
    let cos_vals = cos_args.cwise_map(|x| x.cos());
    
    // 5. 求和得到最终结果
    let result = poly + amp * cos_vals;
    
    (result.as_slice().to_vec(), ...)
}
```

优化技术：
- **Faer 矩阵运算**：高性能 Rust 线性代数库
- **多线程并行**：自动利用多核 CPU
- **CPU 缓存优化**：连续内存访问模式
- **内存预取**：减少缓存未命中

预期收益：

| 计算点数 | 无优化 | 两层优化 | 加速比 |
|---------|--------|---------|--------|
| 1,000 | 100ms | 5ms | **20x** |
| 10,000 | 1000ms | 25ms | **40x** |
| 100,000 | 10000ms | 150ms | **66x** |

适用场景：
- 对比模式大规模计算（如 10000 年逐日对比）
- 历表生成（生成数百年的位置数据）
- 蒙特卡洛模拟（需要数百万次计算）
- 实时交互式应用（需要亚秒级响应）

---

### 精确模式（JPL DE441）优化策略

精确模式的性能瓶颈与简单模式不同，主要开销在于 BSP 文件查找、Chebyshev 多项式求值和坐标转换。

**瓶颈分析**：

| 操作 | 耗时占比 | 说明 |
|------|---------|------|
| BSP 文件查找 | ~40% | Chebyshev 系数读取 |
| Chebyshev 求值 | ~30% | 多项式计算 |
| 坐标转换 | ~20% | 岁差、章动、光行差 |
| 其他 | ~10% | 内存分配、类型转换 |

---

**方案 1：BSP 文件缓存优化**

```rust
// 缓存最近的区间索引，避免重复查找
pub struct JplEphemeris {
    almanac: Arc<Almanac>,
    cached_interval: RwLock<Option<(Frame, f64, usize)>>,
}

fn compute_raw_position(&self, target: Frame, jd_tdb: f64) -> Result<...> {
    // 检查缓存
    if let Some((cached_frame, cached_jd, idx)) = *self.cached_interval.read() {
        if cached_frame == target && (jd_tdb - cached_jd).abs() < 0.01 {
            return self.evaluate_chebyshev_cached(idx, jd_tdb);
        }
    }
    // 正常查找并更新缓存
    let result = self.almanac.translate(...);
    *self.cached_interval.write() = Some((target, jd_tdb, new_idx));
    result
}
```

预期收益：连续时间点计算加速 **2-3x**

---

**方案 2：Chebyshev 多项式批量求值**

```rust
// 批量计算 Chebyshev 多项式，使用矩阵运算
pub fn evaluate_chebyshev_batch(
    coefficients: &[f64],
    t_values: &[f64],
) -> Vec<f64> {
    use faer::Mat;
    
    // 1. 构建 Chebyshev 基矩阵 T[0](t), T[1](t), ..., T[n](t)
    let n = coefficients.len();
    let mut basis = Mat::zeros(t_values.len(), n);
    
    for (i, &t) in t_values.iter().enumerate() {
        basis[(i, 0)] = 1.0;
        basis[(i, 1)] = t;
        for j in 2..n {
            basis[(i, j)] = 2.0 * t * basis[(i, j-1)] - basis[(i, j-2)];
        }
    }
    
    // 2. 矩阵 - 向量乘法
    let coeffs_vec = Mat::from_column_slice(n, 1, coefficients);
    let result = basis * coeffs_vec;
    
    result.as_slice().to_vec()
}
```

预期收益：批量计算加速 **10-20x**

---

**方案 3：坐标转换缓存**

```rust
// 缓存岁差、章动矩阵，避免重复计算
pub struct PrecessionCache {
    cached_t: f64,
    cached_matrix: Mat3,
}

impl PrecessionCache {
    fn get_matrix(&mut self, t: f64) -> &Mat3 {
        if (t - self.cached_t).abs() < 0.0001 {
            &self.cached_matrix  // 复用缓存
        } else {
            self.cached_t = t;
            self.cached_matrix = compute_precession_matrix(t);
            &self.cached_matrix
        }
    }
}
```

预期收益：坐标转换加速 **3-5x**

---

**方案 4：并行计算架构**

```rust
// 使用 Rayon 并行计算多个时间点
use rayon::prelude::*;

let results: Vec<_> = t_values.par_iter()
    .map(|&t| {
        thread_local_jpl().lunar_position(t)
    })
    .collect();
```

预期收益：多核 CPU 利用率提升 **4-8x**（取决于核心数）

---

**方案 5：Anise 库优化**

```rust
// 优化 Epoch 转换开销
// 方案 A：缓存 Epoch
struct CachedEpoch {
    jd_tdb: f64,
    epoch: Epoch,
}

// 方案 B：直接使用儒略日计算（需要 fork anise）
fn translate_from_jd(&self, jd_tdb: f64) -> Result<...> {
    // 绕过 Epoch 转换，直接使用 JD 计算
}
```

预期收益：减少 **10-20%** overhead

---

**精确模式综合优化效果**：

| 优化方案 | 单点计算 | 批量计算 (1000 点) |
|---------|---------|------------------|
| 无优化 | 10ms | 10000ms |
| 方案 1+BSP 缓存 | 5ms (2x) | 5000ms (2x) |
| 方案 2+Chebyshev 批量 | - | 500ms (20x) |
| 方案 3+坐标缓存 | 3ms (3x) | 3000ms (3x) |
| 方案 4+并行 | - | 100ms (100x) |
| **全部优化** | **2ms (5x)** | **50ms (200x)** |

**实施优先级**：
1. **高优先级**（收益大，实现简单）：方案 1（BSP 缓存）、方案 4（并行计算）
2. **中优先级**（收益中等）：方案 3（坐标转换缓存）
3. **低优先级**（需要深入 anise 库）：方案 2（Chebyshev 批量）、方案 5（Anise 优化）

---

### 多步骤矫正融合优化

本项目的天体位置计算需要多个矫正步骤，这些步骤可以融合到矩阵计算中进一步优化。

**当前计算流程**（以行星视位置为例）：

```rust
// 当前：每个时间点独立计算所有 6 个步骤
for &t in t_values {
    let step1 = light_time_correction(t);      // 步骤 1：光时修正
    let step2 = geo_coord(step1);               // 步骤 2：地心坐标转换
    let step3 = gravitational_deflect(step2);   // 步骤 3：引力偏折
    let step4 = aberration(step3);              // 步骤 4：光行差修正
    let step5 = precession(step4);              // 步骤 5：岁差修正
    let step6 = nutation(step5);                // 步骤 6：章动修正
    result.push(step6);
}
```

**问题**：
- 每个步骤都创建临时向量，内存分配开销大
- 无法利用步骤间的矩阵运算机会
- CPU 缓存命中率低

---

**优化方案：端到端矩阵流水线**

```rust
// 优化后：批量计算，多步骤融合
pub fn compute_apparent_position_batch(
    t_values: &[f64],
    body_positions: &[(f64, f64, f64)],
) -> Vec<(f64, f64, f64, f64, f64, f64)> {
    use faer::Mat;
    
    // ========== 步骤 1-2 融合：光时 + 地心坐标 ==========
    let geo_xyz_batch: Mat<f64> = compute_geo_coords_batch(t_values, body_positions);
    // geo_xyz_batch: n×3 矩阵 [x, y, z]
    
    // ========== 步骤 3：引力偏折（批量） ==========
    let geo_deflected = gravitational_deflection_batch(geo_xyz_batch, t_values);
    
    // ========== 步骤 4：光行差（批量） ==========
    let earth_vel_batch = compute_earth_velocity_batch(t_values);
    let geo_aberrated = annual_aberration_batch(geo_deflected, earth_vel_batch);
    
    // ========== 步骤 5-6 融合：岁差 + 章动 ==========
    // 关键优化：岁差和章动都是线性变换，可以合并为一个矩阵
    let combined_matrix = nutation_matrix(t) * precession_matrix(t);
    let combined_matrices = compute_combined_matrices_batch(t_values);
    
    // 批量应用变换：pos[i] = combined_matrix[i] * geo_aberrated[i]
    let geo_nutated = apply_rotations_batch(combined_matrices, geo_aberrated);
    
    // ========== 步骤 7-8 融合：球坐标 + 黄赤转换 ==========
    let epsilon_batch = compute_obliquity_batch(t_values);
    let (ecl_lon, ecl_lat) = equatorial_to_ecliptic_batch(geo_nutated, epsilon_batch);
    
    collect_results(ecl_lon, ecl_lat, ...)
}
```

---

**关键融合点**：

**融合点 1：岁差 + 章动矩阵合并**

```rust
// 当前：两个独立的矩阵乘法
let pos_mean = precession_matrix(t) * pos;
let pos_true = nutation_matrix(t) * pos_mean;

// 优化：合并为一个矩阵
let combined_matrix = nutation_matrix(t) * precession_matrix(t);
let pos_true = combined_matrix * pos;

// 批量版本
let combined_matrices = compute_combined_matrices_batch(t_values);
let pos_true_batch = apply_rotations_batch(combined_matrices, pos_batch);
```

预期收益：减少 **50%** 矩阵乘法

---

**融合点 2：光时 + 地心坐标**

```rust
// 当前：两次独立的坐标计算
let body_retarded = light_time_correction(t);
let geo = body_retarded - earth_pos;

// 优化：直接计算地心向量
let geo_batch = compute_geo_vector_batch(t_values, body_positions);
// 内部使用矩阵运算，避免中间分配
```

预期收益：减少临时向量分配

---

**融合点 3：坐标转换流水线**

```rust
// 当前：多次坐标转换
let xyz = llr2xyz(spherical);
let llr = xyz2llr(xyz);

// 优化：融合转换
let result = llr2llr_direct(spherical, rotation_matrix);
// 直接从球坐标到旋转后的球坐标
```

预期收益：避免 XYZ 中间格式

---

**完整优化架构**：

```
输入：时间向量 [t]，原始位置 [pos]
         ↓
    ┌─────────────────────────┐
    │ 步骤 1-2 融合            │
    │ 光时 + 地心坐标          │
    │ (批量矩阵运算)           │
    └─────────────────────────┘
         ↓ 地心向量 [geo_xyz]
    ┌─────────────────────────┐
    │ 步骤 3：引力偏折          │
    │ (批量向量化)             │
    └─────────────────────────┘
         ↓ 偏折后向量 [deflected]
    ┌─────────────────────────┐
    │ 步骤 4：光行差            │
    │ (批量矩阵乘法)           │
    └─────────────────────────┘
         ↓ 光行差后向量 [aberrated]
    ┌─────────────────────────┐
    │ 步骤 5-6 融合            │
    │ 岁差 + 章动矩阵合并       │
    │ (单个 3×3 矩阵乘法)       │
    └─────────────────────────┘
         ↓ 真赤道坐标 [nutated]
    ┌─────────────────────────┐
    │ 步骤 7-8 融合            │
    │ 球坐标 + 黄赤转换         │
    │ (批量向量化三角函数)      │
    └─────────────────────────┘
         ↓
输出：视位置 [(lon, lat, ra, dec, r)]
```

---

**性能预期**：

| 优化阶段 | 1000 点耗时 | 加速比 |
|---------|-----------|--------|
| 无优化（逐步骤逐个计算） | 1000ms | 1x |
| 步骤内批量（当前方案） | 200ms | 5x |
| **步骤融合 + 批量** | **50ms** | **20x** |
| + 并行计算（8 核） | **10ms** | **100x** |

---

**实施计划**：

1. [ ] 识别可融合的线性变换（岁差 + 章动）
2. [ ] 实现批量坐标转换函数
3. [ ] 创建端到端流水线 API
4. [ ] 性能基准测试和调优

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

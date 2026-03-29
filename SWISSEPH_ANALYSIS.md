# Swiss Ephemeris 朔望计算方法分析

## 核心算法流程

Swiss Ephemeris 在 `swevents.c` 文件中实现了月相（lunar phases）计算。

### 1. 三步插值法 + 牛顿迭代

```c
// 位置：swevents.c:1291-1345

/* lunar phases */
if (do_flag & DO_LPHASE || do_flag & DO_LPHASE0) {
  int new_phase;
  int old_phase;
  int nphases = swe_d2l(360 / phase_mod);  // 4 个相位（新月、上弦、满月、下弦）
  int j;
  double d, dv, xm[6], xs[6];
  
  // 1. 计算三个时间点的日月黄经差
  x2[0] = swe_degnorm((xp2[0] - xs2[0]) * RADTODEG);  // t + tstep
  x1[0] = swe_degnorm((xp1[0] - xs1[0]) * RADTODEG);  // t
  x0[0] = swe_degnorm((xp0[0] - xs0[0]) * RADTODEG);  // t - tstep
  
  // 2. 检测是否有相位变化
  new_phase = floor(x2[0] / phase_mod) + 1;
  old_phase = floor(x1[0] / phase_mod) + 1;
  
  if (old_phase != new_phase) {
    double lphase = HUGE;
    x0[0] = x0[0] / phase_mod - old_phase;
    x1[0] = x1[0] / phase_mod - old_phase;
    x2[0] = x2[0] / phase_mod - old_phase;
    
    // 3. 使用二次插值找到零点（find_zero）
    if ((nzer = find_zero(x0[0], x1[0], x2[0], tstep, &dt1, &dt2)) > 0) {
      if (fabs(dt2) < fabs(dt1))
        t2 = te + dt2;
      else
        t2 = te + dt1;
      
      // 4. 牛顿迭代精化（2 次迭代）
      for (j = 0; j < 2; j++) {
        iflgret = swe_calc(t2, (int) SE_MOON, iflag, xm, serr);
        iflgret = swe_calc(t2, (int) SE_SUN, iflag, xs, serr);
        d = swe_radnorm(xm[0] - xs[0]) * RADTODEG;
        dx = swe_degnorm(d - (new_phase - 1) * 90);
        if (dx > 180) dx -= 360;
        dv = (xm[3] - xs[3]) * RADTODEG;  // 速度差
        t2 -= dx / dv;  // 牛顿迭代
      }
      
      // 5. 输出结果
      print_item(sout, t2, new_phase, lphase, HUGE);
    }
  }
}
```

## 关键特点

### 1. 使用 JPL 历表或内置历表

```c
// swe_calc 调用底层历表
iflgret = swe_calc(t2, (int) SE_MOON, iflag, xm, serr);
iflgret = swe_calc(t2, (int) SE_SUN, iflag, xs, serr);
```

- 支持 JPL DE406/DE431/DE440 等高精度历表
- 也支持 Moshier 历表（MOSEPH，速度更快，精度略低）

### 2. 二次插值法（find_zero）

```c
// 位置：swecl.c:4148
static int find_zero(double y00, double y11, double y2, double dx,
                        double *dxret, double *dxret2)
{
  double a, b, c, x1, x2;
  c = y11;
  b = (y2 - y00) / 2.0;
  a = (y2 + y00) / 2.0 - c;
  
  if (b * b - 4 * a * c < 0)
    return ERR;
    
  x1 = (-b + sqrt(b * b - 4 * a * c)) / 2 / a;
  x2 = (-b - sqrt(b * b - 4 * a * c)) / 2 / a;
  
  *dxret = (x1 - 1) * dx;
  *dxret2 = (x2 - 1) * dx;
  return OK;
}
```

**算法说明：**
- 使用三个点拟合二次抛物线
- 求解抛物线的零点
- 比线性插值更精确，比高次插值更稳定

### 3. 牛顿迭代精化

```c
for (j = 0; j < 2; j++) {
  // 计算日月黄经
  d = swe_radnorm(xm[0] - xs[0]) * RADTODEG;
  
  // 计算与目标相位的偏差
  dx = swe_degnorm(d - (new_phase - 1) * 90);
  if (dx > 180) dx -= 360;
  
  // 计算速度差（月球速度 - 太阳速度）
  dv = (xm[3] - xs[3]) * RADTODEG;
  
  // 牛顿迭代：t_new = t_old - f(t) / f'(t)
  t2 -= dx / dv;
}
```

**关键点：**
- 只迭代 2 次（速度快）
- 使用速度差 `dv` 作为导数
- 月球平均速度约 13.176°/天，太阳约 0.986°/天
- 会合速度约 12.19°/天

## 与当前项目算法对比

| 方面 | Swiss Ephemeris | 当前项目 |
|------|----------------|---------|
| **历表** | JPL DE 系列 / Moshier | 近似多项式（类似 VSOP87） |
| **初始估计** | 三步扫描 + 二次插值 | 解析公式 / 迭代公式 |
| **精化方法** | 牛顿迭代 2 次 | 牛顿迭代 2-3 次 |
| **精度** | < 0.1 秒（JPL 历表） | 约 1 秒 |
| **速度** | 中等 | 快 |
| **依赖** | 需要历表文件 | 无外部依赖 |

## 相位定义

Swiss Ephemeris 使用日月黄经差定义相位：

| 相位 | 日月黄经差 | 符号 |
|------|-----------|------|
| 新月（朔） | 0° | 🌑 |
| 上弦月 | 90° | 🌓 |
| 满月（望） | 180° | 🌕 |
| 下弦月 | 270° | 🌗 |

```c
phase_mod = 90.0;  // 相位间隔
new_phase = floor(x2[0] / phase_mod) + 1;  // 1=新月，2=上弦，3=满月，4=下弦
```

## 关键代码位置

| 文件 | 函数 | 行号 | 功能 |
|------|------|------|------|
| `swevents.c` | `main()` | 1291-1345 | 月相计算主逻辑 |
| `swecl.c` | `find_zero()` | 4148-4162 | 二次插值求根 |
| `swecl.c` | `swe_pheno()` | 3791 | 天文现象计算 |
| `swejpl.c` | `swe_calc()` | - | 历表计算接口 |

## 可借鉴的技术

### 1. 二次插值 + 牛顿迭代

当前项目可以直接使用这个方法改进 `so_high`：

```rust
// 伪代码
fn find_new_moon(t_start: f64, t_step: f64) -> f64 {
    // 1. 计算三个时间点的日月黄经差
    let y0 = moon_lon(t_start - t_step) - sun_lon(t_start - t_step);
    let y1 = moon_lon(t_start) - sun_lon(t_start);
    let y2 = moon_lon(t_start + t_step) - sun_lon(t_start + t_step);
    
    // 2. 二次插值找零点
    let dt = find_zero(y0, y1, y2, t_step);
    let mut t = t_start + dt;
    
    // 3. 牛顿迭代精化（2 次）
    for _ in 0..2 {
        let d = moon_lon(t) - sun_lon(t);
        let dv = moon_speed(t) - sun_speed(t);
        t -= d / dv;
    }
    
    t
}
```

### 2. 使用速度差作为导数

当前项目使用固定速度 `v = 7771.37714500204`，可以改进为：

```rust
// 计算实际速度差
let dv = moon_a_lon(t, 20, 60).speed() - earth_lon(t, 20).speed();
t -= diff / dv;
```

### 3. 多历表支持

可以提供选项使用不同精度的历表：
- 快速模式：当前近似多项式
- 高精度模式：anise + JPL DE440

## 总结

Swiss Ephemeris 的核心优势：
1. **专业历表**：JPL DE 系列提供最高精度
2. **稳健算法**：二次插值 + 牛顿迭代，收敛快且稳定
3. **速度优化**：只迭代 2 次，平衡精度和性能

当前项目可以借鉴：
1. 使用二次插值改进初始估计
2. 使用实际速度差代替固定速度
3. 对于高精度需求，集成 anise 库

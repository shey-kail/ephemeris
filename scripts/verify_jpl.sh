#!/bin/bash
# 验证 JPL 历表计算结果
# 对比：JPL DE441 vs Swiss Ephemeris vs ephe 历表

echo "=== 验证 JPL 历表计算结果 ==="
echo ""

# 测试日期：2000 年 1 月 1 日 12:00 UT
TEST_DATE="2000-01-01.5"
JD="2451545.0"

echo "测试日期：$TEST_DATE (UT)"
echo "儒略日：$JD"
echo ""

# 1. 使用 swetest 计算（使用 ephe 历表）
echo "=== 1. Swiss Ephemeris (ephe 历表) - 行星黄经 ==="
./swetest -b$TEST_DATE -p0 -p1 -p2 -p3 -p4 -p5 -p6 -p7 -p8 -fPlong -n1 -head

echo ""
echo "=== 2. Swiss Ephemeris - 太阳详细位置 ==="
./swetest -b$TEST_DATE -p0 -fPTlrs -n1 -head

echo ""
echo "=== 3. Swiss Ephemeris - 月球详细位置 ==="
./swetest -b$TEST_DATE -p1 -fPTlrs -n1 -head

echo ""
echo "=== 4. Swiss Ephemeris - 金星位置 ==="
./swetest -b$TEST_DATE -p3 -fPTl -n1 -head

echo ""
echo "=== 5. Swiss Ephemeris - 水星位置 ==="
./swetest -b$TEST_DATE -p2 -fPTl -n1 -head

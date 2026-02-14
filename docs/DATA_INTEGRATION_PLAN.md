# ATC 数据集成方案

## 问题分析

当前系统只有 4 个硬编码机场，缺少：
1. 全球机场数据
2. 实时频率信息
3. 真实的 SID/STAR 程序
4. 航路点和航线数据

## 可用数据源

### 1. 开源数据库

#### OurAirports (推荐)
- **网站**: https://ourairports.com/data/
- **数据**: 全球 70,000+ 机场
- **格式**: CSV
- **包含**: ICAO/IATA、坐标、海拔、跑道
- **许可**: Public Domain
- **更新**: 每日更新

#### Open Aviation Data
- **GitHub**: https://github.com/open-aviation-data/airports
- **数据**: 全球机场基础信息
- **格式**: CSV
- **许可**: ODbL

### 2. 模拟器数据

#### X-Plane
- **路径**: `X-Plane 12/Resources/default scenery/default apt dat/Earth nav data/`
- **文件**: `apt.dat` (机场数据)
- **格式**: 文本格式，包含跑道、滑行道、停机位
- **优点**: 免费，详细的机场布局
- **缺点**: 不包含频率和 SID/STAR

#### MSFS
- **路径**: `MSFS/Official/OneStore/fs-base/scenery/`
- **格式**: BGL 二进制文件
- **优点**: 高质量机场数据
- **缺点**: 需要解析工具

### 3. 商业数据（最专业）

#### Navigraph
- **网站**: https://navigraph.com/
- **数据**: AIRAC 周期数据（28天更新）
- **包含**: 
  - 完整的 SID/STAR 程序
  - 进近程序
  - 航路点
  - 通信频率
  - 航路
- **价格**: $9.90/月 或 $89.90/年
- **格式**: 提供 API 和数据文件
- **许可**: 订阅制

#### ChartFox
- **网站**: https://chartfox.org/
- **数据**: 免费的航图和程序
- **包含**: SID/STAR、进近图
- **优点**: 免费
- **缺点**: 数据不如 Navigraph 完整

### 4. 官方数据源

#### FAA (美国)
- **网站**: https://www.faa.gov/air_traffic/flight_info/aeronav/
- **数据**: 美国机场和程序
- **格式**: PDF 航图 + 数据文件
- **许可**: 公开

#### Eurocontrol (欧洲)
- **网站**: https://www.eurocontrol.int/
- **数据**: 欧洲空域数据
- **许可**: 部分公开

#### CAAC (中国)
- **网站**: http://www.caac.gov.cn/
- **数据**: 中国机场 AIP
- **格式**: PDF
- **许可**: 部分公开

## 推荐方案

### 方案 1: 免费方案（适合开发测试）

```
OurAirports CSV + X-Plane apt.dat + 手动维护常用机场频率
```

**优点**:
- 完全免费
- 数据量大（全球机场）
- 易于集成

**缺点**:
- 缺少 SID/STAR
- 频率数据不完整
- 需要手动维护

**实现步骤**:
1. 下载 OurAirports CSV 数据
2. 解析 X-Plane apt.dat 文件
3. 为常用机场手动添加频率和 SID/STAR
4. 定期更新数据

### 方案 2: 商业方案（适合生产环境）

```
Navigraph API + OurAirports 基础数据
```

**优点**:
- 专业级数据质量
- 完整的 SID/STAR 程序
- 准确的频率信息
- 28天周期更新（AIRAC）

**缺点**:
- 需要订阅费用（$9.90/月）
- 需要用户自己订阅

**实现步骤**:
1. 用户订阅 Navigraph
2. 使用 Navigraph API 获取数据
3. 缓存到本地数据库
4. 定期同步更新

### 方案 3: 混合方案（推荐）

```
OurAirports (基础) + X-Plane apt.dat (跑道) + 用户可选 Navigraph (专业)
```

**优点**:
- 免费用户可用基础功能
- 付费用户获得专业数据
- 灵活可扩展

**缺点**:
- 需要维护两套数据源

**实现步骤**:
1. 默认使用免费数据源
2. 提供 Navigraph 集成选项
3. 用户可选择是否启用专业数据

## 实现计划

### Phase 1: 基础数据集成（1-2天）

1. 下载 OurAirports CSV
2. 创建数据导入脚本
3. 解析并存储到 SQLite 数据库
4. 实现机场搜索和查询

### Phase 2: X-Plane 数据解析（2-3天）

1. 解析 apt.dat 文件格式
2. 提取跑道、滑行道数据
3. 合并到数据库

### Phase 3: 频率数据（1-2天）

1. 为主要机场手动添加频率
2. 创建频率管理界面
3. 允许用户自定义频率

### Phase 4: Navigraph 集成（可选，3-5天）

1. 实现 Navigraph API 客户端
2. 数据同步和缓存
3. SID/STAR 程序解析
4. 用户订阅管理

## 数据文件示例

### OurAirports CSV 格式
```csv
id,ident,type,name,latitude_deg,longitude_deg,elevation_ft,continent,iso_country,iso_region,municipality,scheduled_service,gps_code,iata_code,local_code,home_link,wikipedia_link,keywords
6523,"00A","heliport","Total Rf Heliport",40.07080078125,-74.93360137939453,11,"NA","US","US-PA","Bensalem","no","00A",,"00A",,,
323361,"00AA","small_airport","Aero B Ranch Airport",38.704022,-101.473911,3435,"NA","US","US-KS","Leoti","no","00AA",,"00AA",,,
```

### X-Plane apt.dat 格式
```
1   3682 0 0 ZBAA Beijing Capital Intl
100 60.00 1 0 0.00 1 3 1 01  40.08010864  116.58460236    0    0 2 0 0 0 19  40.05010986  116.58460236    0    0 2 0 0 0
110 2 0.00 0.0000 New Taxiway 1
111  40.08010864  116.58460236
```

## 建议

**对于 Virtual ATC 项目，我建议：**

1. **短期（本周）**: 使用方案 1（免费方案）
   - 快速集成 OurAirports 数据
   - 手动维护 10-20 个常用机场的频率
   - 足够支持基础 ATC 功能

2. **中期（下月）**: 添加 X-Plane apt.dat 解析
   - 提供更详细的跑道信息
   - 支持滑行道指引

3. **长期（未来）**: 提供 Navigraph 集成选项
   - 让专业用户获得真实数据
   - 作为高级功能收费

## 立即行动

我现在可以：
1. 下载 OurAirports 数据并集成
2. 创建数据库导入脚本
3. 实现全球机场查询功能

需要我开始吗？

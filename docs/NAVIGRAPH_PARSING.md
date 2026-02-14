# Navigraph 数据解析指南

## Navigraph 数据格式

Navigraph 为 MSFS 提供的数据通常有以下格式：

### 1. BGL 格式（MSFS 原生）
- **位置**: `MSFS Community/navigraph-navdata/scenery/`
- **文件**: `.bgl` 二进制文件
- **内容**: 机场、导航点、航路、程序
- **问题**: 二进制格式，需要专门的解析器

### 2. 数据库格式（推荐）
Navigraph 也提供 SQLite 数据库格式：
- **文件**: `navigraph.db` 或 `navdata.db3`
- **位置**: Navigraph 安装目录
- **优点**: 可以直接用 SQL 查询

### 3. 文本格式
某些工具会导出为文本格式：
- **格式**: CSV, JSON, XML
- **优点**: 易于解析

## 如何获取数据

### 方法 1: 从 Navigraph 安装目录提取

如果你安装了 Navigraph Navdata Center：

**Windows 路径**:
```
C:\Users\<用户名>\AppData\Local\Navigraph\Navigraph Navdata Center\
```

**可能的文件**:
- `navdata.db3` - SQLite 数据库
- `cycle_info.json` - 周期信息
- `airports.json` - 机场数据

### 方法 2: 从 MSFS Community 文件夹提取

**路径**:
```
MSFS Community\navigraph-navdata\scenery\
```

**文件类型**:
- `.bgl` 文件（需要解析器）

### 方法 3: 使用 Little Navmap

Little Navmap 可以读取 Navigraph 数据并导出：

1. 安装 Little Navmap
2. 加载 Navigraph 数据
3. 导出为 SQLite 数据库
4. 我们可以直接读取这个数据库

## 我能解析的格式

### ✅ 可以直接解析：

1. **SQLite 数据库** (.db, .db3, .sqlite)
   ```sql
   SELECT * FROM airport WHERE icao = 'ZBAA';
   SELECT * FROM runway WHERE airport_id = ...;
   SELECT * FROM frequency WHERE airport_id = ...;
   ```

2. **JSON 格式**
   ```json
   {
     "airports": [...],
     "runways": [...],
     "frequencies": [...]
   }
   ```

3. **CSV 格式**
   ```csv
   icao,name,lat,lon,elevation
   ZBAA,Beijing Capital,40.08,116.58,116
   ```

### ⚠️ 需要工具解析：

1. **BGL 二进制文件**
   - 需要使用 BglManip 或类似工具
   - 可以转换为 XML

## 立即可用的方案

### 方案 A: 你提供 Navigraph 数据文件

**如果你有以下任一文件，我可以立即解析：**

1. `navdata.db3` (SQLite 数据库)
2. `airports.json` (JSON 格式)
3. `airports.csv` (CSV 格式)
4. Little Navmap 导出的数据库

**我会：**
1. 解析数据结构
2. 提取机场、跑道、频率、SID/STAR
3. 转换为 Virtual ATC 可用的格式
4. 创建导入脚本

### 方案 B: 使用 Little Navmap 导出

**步骤：**

1. 下载 Little Navmap: https://albar965.github.io/littlenavmap.html
2. 启动后，它会自动检测 Navigraph 数据
3. 菜单 → Scenery Library → Load Scenery Library
4. 数据库位置: `C:\Users\<用户名>\AppData\Roaming\ABarthel\little_navmap_db\`
5. 找到 `little_navmap_navigraph.sqlite` 文件
6. 发给我，我来解析

### 方案 C: 使用开源数据 + 手动补充

如果 Navigraph 数据不方便分享（版权问题），我可以：

1. 使用 OurAirports 开源数据（全球机场）
2. 你告诉我常用的 10-20 个机场
3. 我手动添加这些机场的频率和 SID/STAR
4. 足够日常使用

## 数据库结构示例

如果你有 Navigraph SQLite 数据库，通常包含这些表：

```sql
-- 机场表
CREATE TABLE airport (
    airport_id INTEGER PRIMARY KEY,
    ident TEXT,
    icao TEXT,
    iata TEXT,
    name TEXT,
    latitude REAL,
    longitude REAL,
    elevation INTEGER,
    ...
);

-- 跑道表
CREATE TABLE runway (
    runway_id INTEGER PRIMARY KEY,
    airport_id INTEGER,
    name TEXT,
    heading REAL,
    length INTEGER,
    width INTEGER,
    ...
);

-- 频率表
CREATE TABLE frequency (
    frequency_id INTEGER PRIMARY KEY,
    airport_id INTEGER,
    type TEXT,  -- TOWER, GROUND, APPROACH, etc.
    frequency REAL,
    name TEXT,
    ...
);

-- SID 表
CREATE TABLE sid (
    sid_id INTEGER PRIMARY KEY,
    airport_id INTEGER,
    name TEXT,
    runway TEXT,
    ...
);

-- STAR 表
CREATE TABLE star (
    star_id INTEGER PRIMARY KEY,
    airport_id INTEGER,
    name TEXT,
    runway TEXT,
    ...
);
```

## 下一步

**请告诉我：**

1. 你有哪种格式的 Navigraph 数据？
   - SQLite 数据库？
   - JSON/CSV 文件？
   - BGL 文件？
   - Little Navmap 数据库？

2. 数据文件在哪里？
   - 可以上传到项目目录吗？
   - 或者告诉我路径，我来读取

3. 如果没有 Navigraph 数据：
   - 我可以先用开源数据
   - 你告诉我常用机场，我手动添加频率

**一旦你提供数据文件，我可以在 30 分钟内完成解析和集成！**

# Emulsion-360

> **Emulsion** 是一个快速、极简的开源图片查看器（Rust 编写）。
> **Emulsion-360** 在其基础上增加了 **360° 全景图预览**功能，支持 equirectangular 投影照片的球体内壁渲染。

基于 [ArturKovacs/emulsion](https://github.com/ArturKovacs/emulsion)（v12.3）fork 并扩展。原项目已停更。

## 新增功能

- **360° 全景预览** — 自动检测 2:1 比例全景图，渲染到 3D 球体内壁
- **3D/2D 切换** — 点击底部栏圆形按钮在球体渲染和平铺视图间切换
- **LRU 纹理缓存** — 缓存最近 3 张全景图，左右切换照片时秒开
- **mipmap 支持** — 消除远处锯齿和闪烁
- **鼠标操控** — 拖拽旋转视角，滚轮缩放 FOV

![screenshot](https://github.com/kira4094/Emulsion-360/raw/master/resource/emulsion48.png)

## 构建

需要 [Rust](https://www.rust-lang.org/) 稳定版。

```bash
git clone https://github.com/kira4094/Emulsion-360.git
cd Emulsion-360
cargo run --release
```

打开后拖入一张 equirectangular 全景照片（2:1 比例，宽≥2048px）即可自动切换到 3D 全景模式。

## 操作

| 操作 | 全景模式 | 普通模式 |
|:----|:---------|:---------|
| 左键拖拽 | 旋转视角 | 平移图片 |
| 滚轮 | 缩放 FOV | 缩放图片 |
| A/D 或 ←/→ | 切换照片 | 切换照片 |
| W/S 或 ↑/↓ | 键盘旋转 | 键盘平移 |
| 底部栏 ⬤/○ 按钮 | 切换 3D/2D 模式 | - |
| 底部栏 ▲ 按钮 | 打开 Releases 页面 | 打开 Releases 页面 |
| Esc | 退出全屏/退出程序 | 退出全屏/退出程序 |

## 支持的格式

JPG, PNG, BMP, GIF, TGA, AVIF, TIFF, ICO, HDR, PBM, PAM, PPM, PGM 等。

## 致谢

- [ArturKovacs/emulsion](https://github.com/ArturKovacs/emulsion) — 原始项目，MIT 许可
- 360° 全景渲染基于 OpenGL（glium）实现

## 许可证

MIT License。详见 [LICENSE.txt](LICENSE.txt)。

原项目 [ArturKovacs/emulsion](https://github.com/ArturKovacs/emulsion) 同样使用 MIT 许可。

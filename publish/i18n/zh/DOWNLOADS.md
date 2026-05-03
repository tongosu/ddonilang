# 下载 (中文)

> 这是 starter 本地化文档。命令和文件名保持 canonical 写法。

## 分发位置

- 公开二进制文件应放在 GitHub Releases。
- git 仓库不保存面向用户的二进制文件。

## 目标平台

- Windows x64
- macOS x64/arm64
- Linux x64/arm64

## 推荐文件名

- 'ddonirang-tool-<version>-windows-x64.zip'
- 'ddonirang-tool-<version>-macos-arm64.zip'
- 'ddonirang-tool-<version>-linux-x64.tar.gz'

## 推荐包结构

~~~txt
ddonirang-tool-<version>-<os>-<arch>/
  ddonirang-tool(.exe)
  LICENSE
  NOTICE.txt
  README.txt
~~~

## 校验和

发布时提供 SHA256SUMS.txt。可用时添加签名。

## 源码路径

当前开发状态请从源码构建并本地运行 Seamgrim。见 QUICKSTART.md。

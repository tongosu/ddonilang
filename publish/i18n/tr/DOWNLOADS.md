# İndirmeler (Türkçe)

> Başlangıç çevirisidir. Komutlar ve dosya adları canonical kalır.

## Dağıtım yeri

- Kullanıcıya açık binary'ler GitHub Releases içinde olmalıdır.
- git deposu kullanıcı binary'lerini saklamaz.

## Hedef platformlar

- Windows x64
- macOS x64/arm64
- Linux x64/arm64

## Önerilen dosya adları

- 'ddonirang-tool-<version>-windows-x64.zip'
- 'ddonirang-tool-<version>-macos-arm64.zip'
- 'ddonirang-tool-<version>-linux-x64.tar.gz'

## Önerilen paket yapısı

~~~txt
ddonirang-tool-<version>-<os>-<arch>/
  ddonirang-tool(.exe)
  LICENSE
  NOTICE.txt
  README.txt
~~~

## Checksum

Release ile SHA256SUMS.txt verin. Mümkünse imza ekleyin.

## Kaynak yolu

Güncel geliştirme için kaynaktan derleyip Seamgrim'i yerelde çalıştırın. QUICKSTART.md dosyasına bakın.

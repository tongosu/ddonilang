# Geliştirme yapısı (Türkçe)

> Başlangıç çevirisidir. Komutlar ve dosya adları canonical kalır.

Bu yerelleştirilmiş genel özettir. Ayrıntılı kanonik dosya '../../DDONIRANG_DEV_STRUCTURE.md' dosyasıdır.

## Çekirdek katmanlar

| Katman | Yol | Rol |
| --- | --- | --- |
| core | 'core/' | deterministik motor çekirdeği |
| lang | 'lang/' | gramer, parser, kanonikleştirme |
| tool | 'tool/' | runtime/araç uygulaması |
| CLI | 'tools/teul-cli/' | CLI çalıştırma ve kontroller |
| packs | 'pack/' | çalıştırılabilir pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web çalışma alanı ve Bogae görünümleri |
| tests | 'tests/' | entegrasyon ve ürün testleri |
| publish | 'publish/' | genel dokümanlar |

## Seamgrim workspace V2

- 'ui/index.html': tek giriş noktası
- 'ui/screens/run.js': çalıştırma ekranı ve current-line yürütme
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, ayna özeti
- 'tools/ddn_exec_server.py': yerel statik sunucu ve yardımcı API

## Runtime ilkesi

- DDN runtime, pack'ler, state hash'ler ve ayna/replay kayıtları truth sahibidir.
- Bogae bir görünüm katmanıdır ve runtime truth sahibi değildir.
- Python/JS check ve UI orkestrasyonu yapabilir, fakat dil anlamını test-only lowering ile değiştiremez.

## Güncel evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks

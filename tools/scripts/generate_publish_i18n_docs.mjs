import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const publishRoot = path.join(repoRoot, "publish");
const i18nRoot = path.join(publishRoot, "i18n");

const common = {
  examples: [
    "console-grid minimal example",
    "space2d pendulum and bounce probe",
    "console-grid Tetris",
    "formula/proof/lambda examples",
    "maze probe",
  ],
  commands: [
    "python tests/run_seamgrim_product_stabilization_smoke_check.py",
    "node tests/seamgrim_sample_grid_space_runner.mjs",
    "node tests/seamgrim_studio_layout_contract_runner.mjs",
    "node tests/seamgrim_run_toolbar_compact_runner.mjs",
    "python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py",
  ],
};

const locales = [
  {
    code: "ko", name: "한국어", note: "한국어 기준 공개 문서 묶음입니다.",
    quick: ["빠른 시작", "소스 빌드", "필요: Rust + Cargo", "CLI 확인:", "셈그림 작업실 실행", "로컬 서버를 시작합니다:", "브라우저에서 엽니다:", "작업실은 다음 예제 목록을 열 수 있습니다:", "제품 회귀 확인", "바이너리 릴리스 경로", "공개 바이너리가 배포되면 GitHub Releases에서 내려받습니다. 바이너리는 git 저장소에 넣지 않습니다."],
    dev: ["개발 구조", "이 문서는 공개용 다국어 요약입니다. 자세한 기준 문서는 '../../DDONIRANG_DEV_STRUCTURE.md'입니다.", "핵심 레이어", "레이어", "경로", "역할", "결정성 엔진 코어", "문법, 파서, 정본화", "런타임/도구 구현", "CLI 실행과 검증", "실행 가능한 pack evidence", "웹 작업실과 보개", "통합/제품 테스트", "공개 문서", "셈그림 작업실 V2", "런타임 원칙", "DDN 런타임, pack, state hash, 거울/replay 기록이 truth를 소유합니다.", "보개는 보기 계층이며 runtime truth를 소유하지 않습니다.", "Python/JS는 orchestration과 UI를 맡을 수 있지만 언어 의미를 test-only lowering으로 대신하면 안 됩니다.", "현재 evidence", "CLI/WASM runtime parity", "4권 raw current-line bundle parity", "셈그림 제품 smoke", "보개 마디/그래프 UI check"],
    down: ["다운로드", "배포 위치", "사용자 공개 바이너리는 GitHub Releases에 둡니다.", "git 저장소에는 사용자용 바이너리를 넣지 않습니다.", "지원 대상", "권장 파일명", "권장 패키지 구조", "체크섬", "릴리스에는 SHA256SUMS.txt를 함께 제공합니다. 가능하면 서명도 추가합니다.", "소스 실행", "현재 개발 상태는 소스 빌드와 로컬 셈그림 실행으로 확인합니다. QUICKSTART.md를 보세요."],
    rel: ["릴리즈 노트 2026-02-11", "역사적 릴리즈 노트입니다. 현재 공개 진입점은 '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'입니다.", "요약", "이번 릴리스는 AGE2 Open 정책 강화와 open.net/open.ffi/open.gpu 최소 스키마 및 runtime API에 초점을 둡니다.", "주요 변경", "동작 변경", "open=record|replay는 age_target >= AGE2에서만 허용됩니다. --unsafe-open을 쓰면 명시적으로 우회합니다.", "당시 테스트 명령", "현재 상태 안내", "현재 셈그림/WASM/current-line 상태는 이 폴더의 QUICKSTART.md와 DEV_STRUCTURE.md를 보세요."],
  },
  {
    code: "en", name: "English", note: "English public document set.",
    quick: ["Quick start", "Build from source", "Requirements: Rust + Cargo", "Check the CLI:", "Run Seamgrim workspace", "Start the local server:", "Open in the browser:", "The workspace can open this sample inventory:", "Product regression checks", "Binary release path", "When release binaries are published, download them from GitHub Releases. Binaries are not stored in the git repository."],
    dev: ["Development structure", "This is a localized public summary. The canonical detailed file is '../../DDONIRANG_DEV_STRUCTURE.md'.", "Core layers", "Layer", "Path", "Role", "deterministic engine core", "grammar, parser, canonicalization", "runtime/tool implementation", "CLI execution and checks", "runnable pack evidence", "web workspace and Bogae views", "integration and product tests", "public documents", "Seamgrim workspace V2", "Runtime principle", "DDN runtime, packs, state hashes, and mirror/replay records own truth.", "Bogae is a view layer and must not own runtime truth.", "Python/JS may orchestrate checks and UI, but must not replace language semantics with test-only lowering.", "Current evidence", "CLI/WASM runtime parity", "Vol4 raw current-line bundle parity", "Seamgrim product smoke", "Bogae madi/graph UI checks"],
    down: ["Downloads", "Distribution location", "Public binaries belong in GitHub Releases.", "The git repository does not store user-facing binaries.", "Target platforms", "Recommended file names", "Recommended package layout", "Checksums", "Provide SHA256SUMS.txt with releases. Add a signature when available.", "Source path", "For current development, build from source and run Seamgrim locally. See QUICKSTART.md."],
    rel: ["Release notes 2026-02-11", "Historical release note. Current public entry points are '../../README.md', '../../QUICKSTART.md', and '../../DDONIRANG_DEV_STRUCTURE.md'.", "Summary", "This release focused on AGE2 Open policy hardening and minimum schemas/runtime APIs for open.net/open.ffi/open.gpu.", "Highlights", "Behavior change", "open=record|replay is allowed only for age_target >= AGE2 unless --unsafe-open is used.", "Historical test command", "Current status pointer", "For current Seamgrim/WASM/current-line status, use QUICKSTART.md and DEV_STRUCTURE.md in this language folder."],
  },
  {
    code: "ja", name: "日本語", note: "スターター翻訳です。コマンドとファイル名は canonical のまま維持します。",
    quick: ["クイックスタート", "ソースからビルド", "必要: Rust + Cargo", "CLIを確認:", "셈그림 作業室を実行", "ローカルサーバーを起動:", "ブラウザで開く:", "作業室はこのサンプル一覧を開けます:", "製品回帰チェック", "バイナリリリース経路", "公開バイナリがある場合は GitHub Releases から取得します。バイナリは git リポジトリに保存しません。"],
    dev: ["開発構造", "これは公開用の多言語要約です。詳細な基準文書は '../../DDONIRANG_DEV_STRUCTURE.md' です。", "主要レイヤー", "レイヤー", "パス", "役割", "決定的エンジンコア", "文法、パーサー、正本化", "ランタイム/ツール実装", "CLI実行と検証", "実行可能な pack evidence", "Web作業室と보개", "統合/製品テスト", "公開文書", "셈그림 作業室 V2", "ランタイム原則", "DDN runtime、pack、state hash、鏡/replay 記録が truth を所有します。", "보개 は view layer で runtime truth を所有しません。", "Python/JS は orchestration と UI を担当できますが、言語意味を test-only lowering で置き換えてはいけません。", "現在の evidence", "CLI/WASM runtime parity", "4巻 raw current-line bundle parity", "셈그림 製品 smoke", "보개 madi/graph UI check"],
    down: ["ダウンロード", "配布場所", "ユーザー向けバイナリは GitHub Releases に置きます。", "git リポジトリにはユーザー向けバイナリを入れません。", "対象プラットフォーム", "推奨ファイル名", "推奨パッケージ構造", "チェックサム", "リリースには SHA256SUMS.txt を添付します。可能なら署名も追加します。", "ソース実行", "現在の開発状態はソースビルドとローカル 셈그림 実行で確認します。QUICKSTART.md を見てください。"],
    rel: ["リリースノート 2026-02-11", "過去のリリースノートです。現在の公開入口は '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md' です。", "概要", "このリリースは AGE2 Open ポリシー強化と open.net/open.ffi/open.gpu の最小 schema/runtime API に焦点を当てました。", "主な変更", "動作変更", "open=record|replay は age_target >= AGE2 の場合のみ許可されます。--unsafe-open で明示的に迂回します。", "当時のテストコマンド", "現在状態への案内", "現在の 셈그림/WASM/current-line 状態は、このフォルダの QUICKSTART.md と DEV_STRUCTURE.md を見てください。"],
  },
  {
    code: "tr", name: "Türkçe", note: "Başlangıç çevirisidir. Komutlar ve dosya adları canonical kalır.",
    quick: ["Hızlı başlangıç", "Kaynaktan derleme", "Gereksinimler: Rust + Cargo", "CLI kontrolü:", "Seamgrim çalışma alanını çalıştır", "Yerel sunucuyu başlat:", "Tarayıcıda aç:", "Çalışma alanı bu örnek envanterini açabilir:", "Ürün regresyon kontrolleri", "Binary release yolu", "Release binary'leri yayımlandığında GitHub Releases üzerinden indirin. Binary dosyaları git deposunda saklanmaz."],
    dev: ["Geliştirme yapısı", "Bu yerelleştirilmiş genel özettir. Ayrıntılı kanonik dosya '../../DDONIRANG_DEV_STRUCTURE.md' dosyasıdır.", "Çekirdek katmanlar", "Katman", "Yol", "Rol", "deterministik motor çekirdeği", "gramer, parser, kanonikleştirme", "runtime/araç uygulaması", "CLI çalıştırma ve kontroller", "çalıştırılabilir pack evidence", "web çalışma alanı ve Bogae görünümleri", "entegrasyon ve ürün testleri", "genel dokümanlar", "Seamgrim workspace V2", "Runtime ilkesi", "DDN runtime, pack'ler, state hash'ler ve ayna/replay kayıtları truth sahibidir.", "Bogae bir görünüm katmanıdır ve runtime truth sahibi değildir.", "Python/JS check ve UI orkestrasyonu yapabilir, fakat dil anlamını test-only lowering ile değiştiremez.", "Güncel evidence", "CLI/WASM runtime parity", "Vol4 raw current-line bundle parity", "Seamgrim product smoke", "Bogae madi/graph UI checks"],
    down: ["İndirmeler", "Dağıtım yeri", "Kullanıcıya açık binary'ler GitHub Releases içinde olmalıdır.", "git deposu kullanıcı binary'lerini saklamaz.", "Hedef platformlar", "Önerilen dosya adları", "Önerilen paket yapısı", "Checksum", "Release ile SHA256SUMS.txt verin. Mümkünse imza ekleyin.", "Kaynak yolu", "Güncel geliştirme için kaynaktan derleyip Seamgrim'i yerelde çalıştırın. QUICKSTART.md dosyasına bakın."],
    rel: ["Release notları 2026-02-11", "Tarihsel release notudur. Güncel genel giriş noktaları '../../README.md', '../../QUICKSTART.md' ve '../../DDONIRANG_DEV_STRUCTURE.md' dosyalarıdır.", "Özet", "Bu release AGE2 Open politika güçlendirmesi ve open.net/open.ffi/open.gpu minimum schema/runtime API'lerine odaklandı.", "Öne çıkanlar", "Davranış değişikliği", "open=record|replay yalnızca age_target >= AGE2 için izinlidir; --unsafe-open açık bypass sağlar.", "Tarihsel test komutu", "Güncel durum yönlendirmesi", "Güncel Seamgrim/WASM/current-line durumu için bu klasördeki QUICKSTART.md ve DEV_STRUCTURE.md dosyalarını kullanın."],
  },
  {
    code: "mn", name: "Монгол", note: "Starter орчуулга. Команд ба файлын нэрс canonical хэвээр байна.",
    quick: ["Хурдан эхлэх", "Эх кодоос build хийх", "Шаардлага: Rust + Cargo", "CLI шалгах:", "Seamgrim ажлын өрөөг ажиллуулах", "Локал сервер эхлүүлэх:", "Browser дээр нээх:", "Ажлын өрөө энэ sample inventory-г нээж чадна:", "Бүтээгдэхүүний regression шалгалт", "Binary release зам", "Release binary нийтлэгдсэн үед GitHub Releases-ээс татна. Binary файлууд git repository-д хадгалагдахгүй."],
    dev: ["Хөгжүүлэлтийн бүтэц", "Энэ бол олон хэлний нийтийн товч хураангуй. Дэлгэрэнгүй canonical файл нь '../../DDONIRANG_DEV_STRUCTURE.md'.", "Үндсэн давхаргууд", "Давхарга", "Зам", "Үүрэг", "детерминист engine core", "дүрэм, parser, canonicalization", "runtime/tool хэрэгжилт", "CLI ажиллуулах ба шалгах", "ажиллах pack evidence", "web workspace ба Bogae view", "integration ба product tests", "нийтийн баримтууд", "Seamgrim workspace V2", "Runtime зарчим", "DDN runtime, packs, state hashes, mirror/replay records truth-ийг эзэмшинэ.", "Bogae нь view layer бөгөөд runtime truth эзэмшихгүй.", "Python/JS orchestration ба UI хийж болно, харин хэлний утгыг test-only lowering-оор орлож болохгүй.", "Одоогийн evidence", "CLI/WASM runtime parity", "Vol4 raw current-line bundle parity", "Seamgrim product smoke", "Bogae madi/graph UI checks"],
    down: ["Татах", "Түгээлтийн байршил", "Хэрэглэгчийн binary нь GitHub Releases-д байрлана.", "git repository хэрэглэгчийн binary хадгалахгүй.", "Зорилтот платформууд", "Санал болгосон файлын нэр", "Санал болгосон package бүтэц", "Checksum", "Release-д SHA256SUMS.txt хавсаргана. Боломжтой бол signature нэмнэ.", "Эх кодын зам", "Одоогийн хөгжүүлэлтийг эх кодоос build хийж Seamgrim-ийг локалаар ажиллуулж шалгана. QUICKSTART.md-г харна уу."],
    rel: ["Release notes 2026-02-11", "Түүхэн release note. Одоогийн нийтийн эхлэл нь '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", "Хураангуй", "Энэ release AGE2 Open policy hardening болон open.net/open.ffi/open.gpu minimum schema/runtime API дээр төвлөрсөн.", "Гол өөрчлөлтүүд", "Behavior change", "open=record|replay нь age_target >= AGE2 үед зөвшөөрөгдөнө; --unsafe-open нь ил bypass.", "Түүхэн test command", "Одоогийн төлөв рүү заавар", "Одоогийн Seamgrim/WASM/current-line төлөвийг энэ folder-ийн QUICKSTART.md ба DEV_STRUCTURE.md-ээс харна уу."],
  },
];

function starterLocale(code, name, note, w) {
  return {
    code, name, note,
    quick: [w.quick, w.build, w.req, w.cli, w.run, w.server, w.open, w.samples, w.regress, w.binary, w.binaryText],
    dev: [w.dev, w.devIntro, w.layers, w.layer, w.path, w.role, w.core, w.lang, w.tool, w.cliRole, w.packs, w.seamgrim, w.tests, w.publish, w.workspace, w.principle, w.truth, w.view, w.noLower, w.evidence, w.parity, w.vol4, w.smoke, w.bogae],
    down: [w.downloads, w.dist, w.binRelease, w.noBinRepo, w.targets, w.names, w.layout, w.checksums, w.sumText, w.source, w.sourceText],
    rel: [w.releaseTitle, w.hist, w.summary, w.summaryText, w.highlights, w.behavior, w.behaviorText, w.histTest, w.pointer, w.pointerText],
  };
}

const starter = [
  starterLocale("ay", "Aymara", "Aka qillqata qallta localización ukhamawa. Comandos ukat file sutinakax canonical qhiparaki.", {
    quick:"Jank'aki qallta", build:"Source ukat build luraña", req:"Munasi: Rust + Cargo", cli:"CLI uñakipaña:", run:"Seamgrim workspace apayaña", server:"Local server qalltaña:", open:"Browser ukan jist'araña:", samples:"Workspace ukax aka sample inventory jist'ari", regress:"Producto regression uñakipaña", binary:"Binary release thakhi", binaryText:"Release binary utjipan GitHub Releases ukat apaqaña. Binary file ukax git repository ukar jan uchañawa.",
    dev:"Development structure", devIntro:"Aka qillqatax público localización resumen ukhamawa. Canonical detalle file '../../DDONIRANG_DEV_STRUCTURE.md' ukawa.", layers:"Core layers", layer:"Layer", path:"Path", role:"Lurawi", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI apayaña ukat uñakipaña", packs:"runnable pack evidence", seamgrim:"web workspace ukat Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"Runtime principio", truth:"DDN runtime, packs, state hashes, mirror/replay records ukaw truth katuyi.", view:"Bogae ukax view layer; runtime truth janiw katuykiti.", noLower:"Python/JS orchestration ukat UI lurapxaspawa, ukampis language semantics test-only lowering ukamp jan lantintañawa.", evidence:"Jichha evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Downloads", dist:"Kawkhans apayaña", binRelease:"Public binaries ukax GitHub Releases ukankañapa.", noBinRepo:"git repository ukax user binary janiw imkiti.", targets:"Target platforms", names:"File sutininakaxa wali aski", layout:"Package layout aski", checksums:"Checksums", sumText:"Release ukar SHA256SUMS.txt churam. Atisaxa signature yapxatam.", source:"Source thakhi", sourceText:"Jichha desarrollo uñjañatakix source build luram ukat Seamgrim local apayam. QUICKSTART.md uñjam.",
    releaseTitle:"Release notes 2026-02-11", hist:"Nayra release note. Jichha público mantañanakax '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"Resumen", summaryText:"Aka release AGE2 Open policy hardening ukat open.net/open.ffi/open.gpu minimum schema/runtime API ukanakaru uñtatawa.", highlights:"Jach'a mayjt'awinaka", behavior:"Behavior mayjt'awi", behaviorText:"open=record|replay ukax age_target >= AGE2 ukaki; --unsafe-open ukax qhana bypass.", histTest:"Nayra test command", pointer:"Jichha estado uñacht'awi", pointerText:"Jichha Seamgrim/WASM/current-line estado uñjañatakix aka folder QUICKSTART.md ukat DEV_STRUCTURE.md apnaqam."
  }),
  starterLocale("eu", "Euskara", "Hasierako lokalizazioa da. Komandoak eta fitxategi-izenak canonical geratzen dira.", {
    quick:"Hasiera azkarra", build:"Iturburutik eraiki", req:"Beharrezkoa: Rust + Cargo", cli:"CLI egiaztatu:", run:"Seamgrim workspace abiarazi", server:"Zerbitzari lokala hasi:", open:"Nabigatzailean ireki:", samples:"Workspace-k sample inventory hau ireki dezake", regress:"Produktuaren erregresio probak", binary:"Binary release bidea", binaryText:"Release binaryak argitaratzean GitHub Releases-etik jaitsi. Binary fitxategiak ez dira git biltegian gordetzen.",
    dev:"Garapen egitura", devIntro:"Hau lokalizatutako laburpen publikoa da. Xehetasun canonicala '../../DDONIRANG_DEV_STRUCTURE.md' da.", layers:"Geruza nagusiak", layer:"Geruza", path:"Bidea", role:"Rola", core:"motor deterministaren muina", lang:"gramatika, parserra, canonicalization", tool:"runtime/tresna inplementazioa", cliRole:"CLI exekuzioa eta egiaztapenak", packs:"exekutagarri pack evidence", seamgrim:"web workspace eta Bogae ikuspegiak", tests:"integrazio eta produktu probak", publish:"dokumentu publikoak", workspace:"Seamgrim workspace V2", principle:"Runtime printzipioa", truth:"DDN runtime, packs, state hashes eta mirror/replay erregistroek truth dute.", view:"Bogae view layer da eta ez du runtime truth jabetzen.", noLower:"Python/JS orkestrazio eta UI-rako erabil daitezke, baina ez dute hizkuntza esanahia test-only lowering bidez ordezkatu behar.", evidence:"Uneko evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Deskargak", dist:"Banaketa lekua", binRelease:"Binary publikoak GitHub Releases-en egon behar dira.", noBinRepo:"git biltegiak ez ditu erabiltzaileentzako binaryak gordetzen.", targets:"Helburu plataformak", names:"Gomendatutako fitxategi-izenak", layout:"Gomendatutako pakete egitura", checksums:"Checksums", sumText:"Release bakoitzarekin SHA256SUMS.txt eman. Ahal bada sinadura gehitu.", source:"Iturburu bidea", sourceText:"Uneko garapena ikusteko iturburutik eraiki eta Seamgrim lokalean exekutatu. Ikusi QUICKSTART.md.",
    releaseTitle:"Release oharrak 2026-02-11", hist:"Ohar historikoa da. Uneko sarrera publikoak '../../README.md', '../../QUICKSTART.md' eta '../../DDONIRANG_DEV_STRUCTURE.md' dira.", summary:"Laburpena", summaryText:"Release honek AGE2 Open policy hardening eta open.net/open.ffi/open.gpu minimum schema/runtime APIak landu zituen.", highlights:"Nabarmenak", behavior:"Portaera aldaketa", behaviorText:"open=record|replay age_target >= AGE2 denean bakarrik onartzen da; --unsafe-open bypass esplizitua da.", histTest:"Test komando historikoa", pointer:"Uneko egoeraren lotura", pointerText:"Uneko Seamgrim/WASM/current-line egoerarako erabili folder honetako QUICKSTART.md eta DEV_STRUCTURE.md."
  }),
  starterLocale("kn", "ಕನ್ನಡ", "ಇದು starter localization. Commands ಮತ್ತು file names canonical ಆಗಿಯೇ ಉಳಿಯುತ್ತವೆ.", {
    quick:"ತ್ವರಿತ ಆರಂಭ", build:"source ನಿಂದ build", req:"ಅವಶ್ಯಕತೆ: Rust + Cargo", cli:"CLI ಪರಿಶೀಲನೆ:", run:"Seamgrim workspace ಚಾಲನೆ", server:"local server ಆರಂಭಿಸಿ:", open:"browser ನಲ್ಲಿ ತೆರೆಯಿರಿ:", samples:"workspace ಈ sample inventory ತೆರೆಯಬಹುದು", regress:"product regression checks", binary:"binary release ಮಾರ್ಗ", binaryText:"release binaries ಪ್ರಕಟವಾದಾಗ GitHub Releases ನಿಂದ ಪಡೆಯಿರಿ. binaries ಅನ್ನು git repository ನಲ್ಲಿ ಇರಿಸಲಾಗುವುದಿಲ್ಲ.",
    dev:"development structure", devIntro:"ಇದು public localized summary. canonical ವಿವರವಾದ file '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"core layers", layer:"layer", path:"path", role:"ಪಾತ್ರ", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI execution ಮತ್ತು checks", packs:"runnable pack evidence", seamgrim:"web workspace ಮತ್ತು Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"runtime principle", truth:"DDN runtime, packs, state hashes, mirror/replay records truth ಹೊಂದಿವೆ.", view:"Bogae view layer; runtime truth ಹೊಂದುವುದಿಲ್ಲ.", noLower:"Python/JS orchestration ಮತ್ತು UI ಮಾಡಬಹುದು; language semantics ಅನ್ನು test-only lowering ಮೂಲಕ ಬದಲಿಸಬಾರದು.", evidence:"current evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"downloads", dist:"distribution location", binRelease:"public binaries GitHub Releases ನಲ್ಲಿ ಇರಬೇಕು.", noBinRepo:"git repository user-facing binaries ಸಂಗ್ರಹಿಸುವುದಿಲ್ಲ.", targets:"target platforms", names:"recommended file names", layout:"recommended package layout", checksums:"checksums", sumText:"release ಜೊತೆಗೆ SHA256SUMS.txt ನೀಡಿ. ಸಾಧ್ಯವಾದರೆ signature ಸೇರಿಸಿ.", source:"source path", sourceText:"current development ಪರಿಶೀಲಿಸಲು source build ಮಾಡಿ Seamgrim local ಆಗಿ ಚಲಾಯಿಸಿ. QUICKSTART.md ನೋಡಿ.",
    releaseTitle:"release notes 2026-02-11", hist:"historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"ಸಾರಾಂಶ", summaryText:"ಈ release AGE2 Open policy hardening ಮತ್ತು open.net/open.ffi/open.gpu minimum schema/runtime API ಮೇಲೆ ಕೇಂದ್ರೀಕರಿಸಿತು.", highlights:"ಮುಖ್ಯಾಂಶಗಳು", behavior:"behavior change", behaviorText:"open=record|replay age_target >= AGE2 ಆಗಿರುವಾಗ ಮಾತ್ರ; --unsafe-open explicit bypass.", histTest:"historical test command", pointer:"current status pointer", pointerText:"current Seamgrim/WASM/current-line status ಗಾಗಿ ಈ folder ನ QUICKSTART.md ಮತ್ತು DEV_STRUCTURE.md ಬಳಸಿ."
  }),
  starterLocale("ne", "नेपाली", "यो starter localization हो। Commands र file names canonical नै रहन्छन्।", {
    quick:"छिटो सुरु", build:"source बाट build", req:"आवश्यक: Rust + Cargo", cli:"CLI जाँच:", run:"Seamgrim workspace चलाउने", server:"local server सुरु गर्नुहोस्:", open:"browser मा खोल्नुहोस्:", samples:"workspace ले यो sample inventory खोल्न सक्छ", regress:"product regression checks", binary:"binary release बाटो", binaryText:"release binaries प्रकाशित भएपछि GitHub Releases बाट डाउनलोड गर्नुहोस्। binaries git repository मा राखिँदैन।",
    dev:"development structure", devIntro:"यो public localized summary हो। canonical विस्तृत file '../../DDONIRANG_DEV_STRUCTURE.md' हो।", layers:"core layers", layer:"layer", path:"path", role:"भूमिका", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI execution र checks", packs:"runnable pack evidence", seamgrim:"web workspace र Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"runtime principle", truth:"DDN runtime, packs, state hashes, mirror/replay records ले truth राख्छन्।", view:"Bogae view layer हो; runtime truth राख्दैन।", noLower:"Python/JS orchestration र UI का लागि हुन सक्छ, तर language semantics लाई test-only lowering ले बदल्नु हुँदैन।", evidence:"current evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"downloads", dist:"distribution location", binRelease:"public binaries GitHub Releases मा हुनुपर्छ।", noBinRepo:"git repository ले user-facing binaries राख्दैन।", targets:"target platforms", names:"recommended file names", layout:"recommended package layout", checksums:"checksums", sumText:"release सँग SHA256SUMS.txt दिनुहोस्। सम्भव भए signature थप्नुहोस्।", source:"source path", sourceText:"current development हेर्न source build गरेर Seamgrim local चलाउनुहोस्। QUICKSTART.md हेर्नुहोस्।",
    releaseTitle:"release notes 2026-02-11", hist:"historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"सारांश", summaryText:"यो release AGE2 Open policy hardening र open.net/open.ffi/open.gpu minimum schema/runtime API मा केन्द्रित थियो।", highlights:"मुख्य बुँदा", behavior:"behavior change", behaviorText:"open=record|replay age_target >= AGE2 मा मात्र; --unsafe-open explicit bypass हो।", histTest:"historical test command", pointer:"current status pointer", pointerText:"current Seamgrim/WASM/current-line status का लागि यस folder को QUICKSTART.md र DEV_STRUCTURE.md प्रयोग गर्नुहोस्।"
  }),
  starterLocale("qu", "Runasimi", "Kay qillqa starter localizaciónmi. Comandos hinallataq file sutikuna canonical kachkan.", {
    quick:"Utqay qallariy", build:"Source manta build ruray", req:"Munakun: Rust + Cargo", cli:"CLI qhaway:", run:"Seamgrim workspace purichiy", server:"Local server qallariy:", open:"Browserpi kichay:", samples:"Workspace kay sample inventory kichanman", regress:"Product regression qhaway", binary:"Binary release ñan", binaryText:"Release binaries lluqsiptin GitHub Releases manta uraykachiy. Binary filekunaqa git repositorypi mana churakunchu.",
    dev:"Development structure", devIntro:"Kayqa público localización resumenmi. Canonical detalle file '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"Core layers", layer:"Layer", path:"Path", role:"Ruray", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI purichiy chaymanta qhaway", packs:"runnable pack evidence", seamgrim:"web workspace chaymanta Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"Runtime principio", truth:"DDN runtime, packs, state hashes, mirror/replay records truth hap'in.", view:"Bogae view layermi; runtime truth mana hap'inchu.", noLower:"Python/JS orchestration/UI rurayta atin, ichaqa language semantics test-only loweringwan mana tikranachu.", evidence:"Kunan evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Downloads", dist:"Maypi rakiy", binRelease:"Public binaries GitHub Releasespi kanan.", noBinRepo:"git repository user binarykunata mana waqaychan.", targets:"Target platforms", names:"File sutikuna allinchasqa", layout:"Package layout allinchasqa", checksums:"Checksums", sumText:"Releasewan SHA256SUMS.txt quy. Atikuptinqa signature yapay.", source:"Source ñan", sourceText:"Kunan development qhawanapaq source build ruray chaymanta Seamgrim local purichiy. QUICKSTART.md qhaway.",
    releaseTitle:"Release notes 2026-02-11", hist:"Ñawpa release note. Kunan público yaykuna '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"Resumen", summaryText:"Kay release AGE2 Open policy hardening, open.net/open.ffi/open.gpu minimum schema/runtime API nisqaman qhawarqan.", highlights:"Hatun willakuykuna", behavior:"Behavior tikray", behaviorText:"open=record|replay age_target >= AGE2 kaptinlla; --unsafe-open qhapaq bypass.", histTest:"Ñawpa test command", pointer:"Kunan estado ñiqichay", pointerText:"Kunan Seamgrim/WASM/current-line estado qhawanapaq kay folder QUICKSTART.md, DEV_STRUCTURE.md apaykachay."
  }),
  starterLocale("sym3", "sym3", "compact localized set :: commands + file names stay canonical", {
    quick:"quick_start", build:"build_from_source", req:"requires: Rust + Cargo", cli:"check_cli:", run:"run_Seamgrim_workspace", server:"start_local_server:", open:"open_browser:", samples:"workspace opens sample_inventory", regress:"product_regression_checks", binary:"binary_release_path", binaryText:"download release binaries from GitHub Releases; do not store binaries in git repo.",
    dev:"dev_structure", devIntro:"localized public summary; canonical detail = '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"core_layers", layer:"layer", path:"path", role:"role", core:"det_engine_core", lang:"grammar/parser/canon", tool:"runtime_tool_impl", cliRole:"CLI_run_checks", packs:"runnable_pack_evidence", seamgrim:"web_workspace+Bogae_views", tests:"integration_product_tests", publish:"public_docs", workspace:"Seamgrim_workspace_V2", principle:"runtime_principle", truth:"DDN_runtime+packs+state_hash+mirror/replay own truth.", view:"Bogae == view_layer; not runtime_truth.", noLower:"Python/JS == orchestration/UI only; no test_only_lowering semantics.", evidence:"current_evidence", parity:"CLI_WASM_runtime_parity", vol4:"vol4_raw_currentline_parity", smoke:"Seamgrim_product_smoke", bogae:"Bogae_madi_graph_UI_checks",
    downloads:"downloads", dist:"distribution", binRelease:"public_binaries -> GitHub_Releases", noBinRepo:"git_repo stores no user_binaries", targets:"targets", names:"file_names", layout:"package_layout", checksums:"checksums", sumText:"ship SHA256SUMS.txt; add signature if possible.", source:"source_path", sourceText:"for current dev: build source + run Seamgrim local; see QUICKSTART.md.",
    releaseTitle:"release_notes_20260211", hist:"historical note; current entries: '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"summary", summaryText:"AGE2 Open hardening + open.net/open.ffi/open.gpu minimum schema/runtime API.", highlights:"highlights", behavior:"behavior_change", behaviorText:"open=record|replay only when age_target>=AGE2; --unsafe-open = explicit bypass.", histTest:"historical_test_command", pointer:"current_status_pointer", pointerText:"current Seamgrim/WASM/current-line => QUICKSTART.md + DEV_STRUCTURE.md."
  }),
  starterLocale("ta", "தமிழ்", "இது starter localization. Commands மற்றும் file names canonical ஆகவே இருக்கும்.", {
    quick:"விரைவு தொடக்கம்", build:"source இலிருந்து build", req:"தேவை: Rust + Cargo", cli:"CLI சரிபார்ப்பு:", run:"Seamgrim workspace இயக்கவும்", server:"local server தொடங்கு:", open:"browser இல் திற:", samples:"workspace இந்த sample inventory திறக்க முடியும்", regress:"product regression checks", binary:"binary release பாதை", binaryText:"release binaries வெளியானபின் GitHub Releases இலிருந்து பதிவிறக்கவும். binaries git repository இல் வைக்கப்படாது.",
    dev:"development structure", devIntro:"இது public localized summary. canonical விவர file '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"core layers", layer:"layer", path:"path", role:"பங்கு", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI execution மற்றும் checks", packs:"runnable pack evidence", seamgrim:"web workspace மற்றும் Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"runtime principle", truth:"DDN runtime, packs, state hashes, mirror/replay records truth வைத்திருக்கும்.", view:"Bogae view layer; runtime truth வைத்திருக்காது.", noLower:"Python/JS orchestration மற்றும் UI செய்யலாம்; ஆனால் language semantics ஐ test-only lowering ஆக மாற்றக்கூடாது.", evidence:"current evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"downloads", dist:"distribution location", binRelease:"public binaries GitHub Releases இல் இருக்க வேண்டும்.", noBinRepo:"git repository user-facing binaries சேமிக்காது.", targets:"target platforms", names:"recommended file names", layout:"recommended package layout", checksums:"checksums", sumText:"release உடன் SHA256SUMS.txt வழங்கவும். முடிந்தால் signature சேர்க்கவும்.", source:"source path", sourceText:"current development பார்க்க source build செய்து Seamgrim local இயக்கவும். QUICKSTART.md பார்க்கவும்.",
    releaseTitle:"release notes 2026-02-11", hist:"historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"சுருக்கம்", summaryText:"இந்த release AGE2 Open policy hardening மற்றும் open.net/open.ffi/open.gpu minimum schema/runtime API மீது கவனம் வைத்தது.", highlights:"முக்கிய மாற்றங்கள்", behavior:"behavior change", behaviorText:"open=record|replay age_target >= AGE2 என்றால் மட்டும்; --unsafe-open explicit bypass.", histTest:"historical test command", pointer:"current status pointer", pointerText:"current Seamgrim/WASM/current-line status க்கு இந்த folder இல் QUICKSTART.md மற்றும் DEV_STRUCTURE.md பயன்படுத்தவும்."
  }),
  starterLocale("te", "తెలుగు", "ఇది starter localization. Commands మరియు file names canonical గానే ఉంటాయి.", {
    quick:"త్వరిత ప్రారంభం", build:"source నుండి build", req:"అవసరం: Rust + Cargo", cli:"CLI తనిఖీ:", run:"Seamgrim workspace నడపండి", server:"local server ప్రారంభించండి:", open:"browser లో తెరవండి:", samples:"workspace ఈ sample inventory తెరవగలదు", regress:"product regression checks", binary:"binary release మార్గం", binaryText:"release binaries వచ్చినప్పుడు GitHub Releases నుండి తీసుకోండి. binaries git repository లో ఉంచబడవు.",
    dev:"development structure", devIntro:"ఇది public localized summary. canonical వివర file '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"core layers", layer:"layer", path:"path", role:"పాత్ర", core:"deterministic engine core", lang:"grammar, parser, canonicalization", tool:"runtime/tool implementation", cliRole:"CLI execution మరియు checks", packs:"runnable pack evidence", seamgrim:"web workspace మరియు Bogae views", tests:"integration/product tests", publish:"public documents", workspace:"Seamgrim workspace V2", principle:"runtime principle", truth:"DDN runtime, packs, state hashes, mirror/replay records truth కలిగి ఉంటాయి.", view:"Bogae view layer; runtime truth కలిగి ఉండదు.", noLower:"Python/JS orchestration మరియు UI చేయగలవు; language semantics ను test-only lowering తో మార్చకూడదు.", evidence:"current evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"downloads", dist:"distribution location", binRelease:"public binaries GitHub Releases లో ఉండాలి.", noBinRepo:"git repository user-facing binaries నిల్వ చేయదు.", targets:"target platforms", names:"recommended file names", layout:"recommended package layout", checksums:"checksums", sumText:"release తో SHA256SUMS.txt ఇవ్వండి. సాధ్యమైతే signature జోడించండి.", source:"source path", sourceText:"current development చూడటానికి source build చేసి Seamgrim local నడపండి. QUICKSTART.md చూడండి.",
    releaseTitle:"release notes 2026-02-11", hist:"historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"సారాంశం", summaryText:"ఈ release AGE2 Open policy hardening మరియు open.net/open.ffi/open.gpu minimum schema/runtime API పై కేంద్రీకృతమైంది.", highlights:"ముఖ్యాంశాలు", behavior:"behavior change", behaviorText:"open=record|replay age_target >= AGE2 లో మాత్రమే; --unsafe-open explicit bypass.", histTest:"historical test command", pointer:"current status pointer", pointerText:"current Seamgrim/WASM/current-line status కోసం ఈ folder లో QUICKSTART.md మరియు DEV_STRUCTURE.md వాడండి."
  }),
  starterLocale("zh", "中文", "这是 starter 本地化文档。命令和文件名保持 canonical 写法。", {
    quick:"快速开始", build:"从源码构建", req:"需要: Rust + Cargo", cli:"检查 CLI:", run:"运行 Seamgrim 工作室", server:"启动本地服务器:", open:"在浏览器中打开:", samples:"工作室可以打开这个样例清单", regress:"产品回归检查", binary:"二进制发布路径", binaryText:"发布二进制文件时，从 GitHub Releases 下载。二进制文件不存入 git 仓库。",
    dev:"开发结构", devIntro:"这是公开用本地化摘要。详细 canonical 文档是 '../../DDONIRANG_DEV_STRUCTURE.md'。", layers:"核心层", layer:"层", path:"路径", role:"角色", core:"确定性引擎核心", lang:"语法、解析器、canonicalization", tool:"runtime/tool 实现", cliRole:"CLI 执行与检查", packs:"可执行 pack evidence", seamgrim:"web 工作室与 Bogae 视图", tests:"集成和产品测试", publish:"公开文档", workspace:"Seamgrim workspace V2", principle:"Runtime 原则", truth:"DDN runtime、packs、state hashes、mirror/replay records 拥有 truth。", view:"Bogae 是 view layer，不拥有 runtime truth。", noLower:"Python/JS 可以做 orchestration 和 UI，但不能用 test-only lowering 取代语言语义。", evidence:"当前 evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"下载", dist:"分发位置", binRelease:"公开二进制文件应放在 GitHub Releases。", noBinRepo:"git 仓库不保存面向用户的二进制文件。", targets:"目标平台", names:"推荐文件名", layout:"推荐包结构", checksums:"校验和", sumText:"发布时提供 SHA256SUMS.txt。可用时添加签名。", source:"源码路径", sourceText:"当前开发状态请从源码构建并本地运行 Seamgrim。见 QUICKSTART.md。",
    releaseTitle:"发布说明 2026-02-11", hist:"历史发布说明。当前公开入口是 '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'。", summary:"摘要", summaryText:"本次发布聚焦 AGE2 Open 策略强化，以及 open.net/open.ffi/open.gpu 的最小 schema/runtime API。", highlights:"重点", behavior:"行为变更", behaviorText:"open=record|replay 仅在 age_target >= AGE2 时允许；--unsafe-open 是显式绕过。", histTest:"历史测试命令", pointer:"当前状态指引", pointerText:"当前 Seamgrim/WASM/current-line 状态请使用本文件夹的 QUICKSTART.md 和 DEV_STRUCTURE.md。"
  }),
  starterLocale("es", "Español", "Traducción starter. Los comandos y nombres de archivo se mantienen canonical.", {
    quick:"Inicio rápido", build:"Construir desde el código fuente", req:"Requisitos: Rust + Cargo", cli:"Comprobar la CLI:", run:"Ejecutar el workspace de Seamgrim", server:"Inicia el servidor local:", open:"Abre en el navegador:", samples:"El workspace puede abrir este inventario de ejemplos", regress:"Comprobaciones de regresión del producto", binary:"Ruta de publicación binaria", binaryText:"Cuando haya binarios publicados, descárgalos desde GitHub Releases. Los binarios no se guardan en el repositorio git.",
    dev:"Estructura de desarrollo", devIntro:"Este es un resumen público localizado. El archivo canonical detallado es '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"Capas principales", layer:"Capa", path:"Ruta", role:"Rol", core:"núcleo de motor determinista", lang:"gramática, parser, canonicalization", tool:"implementación runtime/tool", cliRole:"ejecución y checks de CLI", packs:"pack evidence ejecutable", seamgrim:"workspace web y vistas Bogae", tests:"tests de integración y producto", publish:"documentos públicos", workspace:"Seamgrim workspace V2", principle:"Principio de runtime", truth:"DDN runtime, packs, state hashes y registros mirror/replay poseen la truth.", view:"Bogae es una view layer y no posee runtime truth.", noLower:"Python/JS puede orquestar checks y UI, pero no debe reemplazar la semántica del lenguaje con test-only lowering.", evidence:"Evidence actual", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Descargas", dist:"Ubicación de distribución", binRelease:"Los binarios públicos deben estar en GitHub Releases.", noBinRepo:"El repositorio git no guarda binarios para usuarios.", targets:"Plataformas objetivo", names:"Nombres de archivo recomendados", layout:"Estructura de paquete recomendada", checksums:"Checksums", sumText:"Incluye SHA256SUMS.txt con las releases. Añade firma cuando esté disponible.", source:"Ruta desde código fuente", sourceText:"Para el desarrollo actual, construye desde source y ejecuta Seamgrim localmente. Consulta QUICKSTART.md.",
    releaseTitle:"Notas de release 2026-02-11", hist:"Nota histórica de release. Las entradas públicas actuales son '../../README.md', '../../QUICKSTART.md' y '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"Resumen", summaryText:"Esta release se centró en endurecer la política AGE2 Open y en los minimum schemas/runtime APIs de open.net/open.ffi/open.gpu.", highlights:"Cambios principales", behavior:"Cambio de comportamiento", behaviorText:"open=record|replay solo se permite con age_target >= AGE2; --unsafe-open es un bypass explícito.", histTest:"Comando histórico de test", pointer:"Puntero de estado actual", pointerText:"Para el estado actual de Seamgrim/WASM/current-line, usa QUICKSTART.md y DEV_STRUCTURE.md en esta carpeta."
  }),
  starterLocale("fr", "Français", "Traduction starter. Les commandes et noms de fichiers restent canonical.", {
    quick:"Démarrage rapide", build:"Construire depuis les sources", req:"Pré-requis : Rust + Cargo", cli:"Vérifier la CLI :", run:"Lancer le workspace Seamgrim", server:"Démarrer le serveur local :", open:"Ouvrir dans le navigateur :", samples:"Le workspace peut ouvrir cet inventaire d'exemples", regress:"Contrôles de régression produit", binary:"Chemin de release binaire", binaryText:"Quand des binaires sont publiés, les télécharger depuis GitHub Releases. Les binaires ne sont pas stockés dans le dépôt git.",
    dev:"Structure de développement", devIntro:"Ceci est un résumé public localisé. Le fichier canonical détaillé est '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"Couches principales", layer:"Couche", path:"Chemin", role:"Rôle", core:"coeur de moteur déterministe", lang:"grammaire, parser, canonicalization", tool:"implémentation runtime/tool", cliRole:"exécution CLI et checks", packs:"pack evidence exécutable", seamgrim:"workspace web et vues Bogae", tests:"tests d'intégration et produit", publish:"documents publics", workspace:"Seamgrim workspace V2", principle:"Principe runtime", truth:"Le DDN runtime, les packs, state hashes et enregistrements mirror/replay possèdent la truth.", view:"Bogae est une view layer et ne possède pas la runtime truth.", noLower:"Python/JS peuvent orchestrer les checks et l'UI, mais ne doivent pas remplacer la sémantique par du test-only lowering.", evidence:"Evidence actuelle", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Téléchargements", dist:"Lieu de distribution", binRelease:"Les binaires publics doivent être dans GitHub Releases.", noBinRepo:"Le dépôt git ne stocke pas de binaires utilisateur.", targets:"Plateformes cibles", names:"Noms de fichiers recommandés", layout:"Structure de paquet recommandée", checksums:"Checksums", sumText:"Fournir SHA256SUMS.txt avec les releases. Ajouter une signature si disponible.", source:"Chemin source", sourceText:"Pour l'état de développement actuel, construire depuis les sources et lancer Seamgrim localement. Voir QUICKSTART.md.",
    releaseTitle:"Notes de release 2026-02-11", hist:"Note de release historique. Les points d'entrée publics actuels sont '../../README.md', '../../QUICKSTART.md' et '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"Résumé", summaryText:"Cette release a renforcé la politique AGE2 Open et les minimum schemas/runtime APIs de open.net/open.ffi/open.gpu.", highlights:"Points forts", behavior:"Changement de comportement", behaviorText:"open=record|replay est autorisé seulement avec age_target >= AGE2 ; --unsafe-open est un contournement explicite.", histTest:"Commande de test historique", pointer:"Pointeur d'état actuel", pointerText:"Pour l'état actuel Seamgrim/WASM/current-line, utiliser QUICKSTART.md et DEV_STRUCTURE.md dans ce dossier."
  }),
  starterLocale("de", "Deutsch", "Starter-Lokalisierung. Befehle und Dateinamen bleiben canonical.", {
    quick:"Schnellstart", build:"Aus dem Quellcode bauen", req:"Voraussetzungen: Rust + Cargo", cli:"CLI prüfen:", run:"Seamgrim workspace starten", server:"Lokalen Server starten:", open:"Im Browser öffnen:", samples:"Der workspace kann dieses Beispielinventar öffnen", regress:"Produkt-Regressionsprüfungen", binary:"Binary-Release-Pfad", binaryText:"Wenn Release-Binaries veröffentlicht werden, lade sie aus GitHub Releases herunter. Binaries werden nicht im git repository gespeichert.",
    dev:"Entwicklungsstruktur", devIntro:"Dies ist eine lokalisierte öffentliche Zusammenfassung. Die detaillierte canonical Datei ist '../../DDONIRANG_DEV_STRUCTURE.md'.", layers:"Kernschichten", layer:"Schicht", path:"Pfad", role:"Rolle", core:"deterministischer Engine-Kern", lang:"Grammatik, Parser, canonicalization", tool:"runtime/tool Implementierung", cliRole:"CLI-Ausführung und Checks", packs:"ausführbare pack evidence", seamgrim:"Web-workspace und Bogae views", tests:"Integrations- und Produkttests", publish:"öffentliche Dokumente", workspace:"Seamgrim workspace V2", principle:"Runtime-Prinzip", truth:"DDN runtime, packs, state hashes und mirror/replay records besitzen die truth.", view:"Bogae ist eine view layer und besitzt keine runtime truth.", noLower:"Python/JS dürfen Checks und UI orchestrieren, aber Sprachsemantik nicht durch test-only lowering ersetzen.", evidence:"Aktuelle evidence", parity:"CLI/WASM runtime parity", vol4:"Vol4 raw current-line bundle parity", smoke:"Seamgrim product smoke", bogae:"Bogae madi/graph UI checks",
    downloads:"Downloads", dist:"Distributionsort", binRelease:"Öffentliche Binaries gehören in GitHub Releases.", noBinRepo:"Das git repository speichert keine Benutzer-Binaries.", targets:"Zielplattformen", names:"Empfohlene Dateinamen", layout:"Empfohlene Paketstruktur", checksums:"Checksums", sumText:"Mit Releases SHA256SUMS.txt bereitstellen. Wenn möglich eine Signatur hinzufügen.", source:"Quellpfad", sourceText:"Für den aktuellen Entwicklungsstand aus den Quellen bauen und Seamgrim lokal starten. Siehe QUICKSTART.md.",
    releaseTitle:"Release notes 2026-02-11", hist:"Historische Release-Notiz. Aktuelle öffentliche Einstiegspunkte sind '../../README.md', '../../QUICKSTART.md' und '../../DDONIRANG_DEV_STRUCTURE.md'.", summary:"Zusammenfassung", summaryText:"Diese Release fokussierte AGE2 Open policy hardening sowie minimum schemas/runtime APIs für open.net/open.ffi/open.gpu.", highlights:"Highlights", behavior:"Verhaltensänderung", behaviorText:"open=record|replay ist nur mit age_target >= AGE2 erlaubt; --unsafe-open ist ein expliziter Bypass.", histTest:"Historischer Testbefehl", pointer:"Aktueller Statushinweis", pointerText:"Für den aktuellen Seamgrim/WASM/current-line Status QUICKSTART.md und DEV_STRUCTURE.md in diesem Ordner nutzen."
  }),
];

locales.push(...starter);

const uiFileLines = {
  ko: ["단일 진입점", "실행 화면과 current-line 실행", "console/graph/space2d/grid 보개 렌더링", "마디, 런타임 상태, 거울 요약", "로컬 정적 서버와 보조 API"],
  en: ["single entry point", "run screen and current-line execution", "console/graph/space2d/grid rendering", "madi, runtime state, mirror summary", "local static server and helper API"],
  ja: ["単一入口", "実行画面と current-line 実行", "console/graph/space2d/grid の보개レンダリング", "madi、runtime state、鏡の要約", "ローカル静的サーバーと補助 API"],
  tr: ["tek giriş noktası", "çalıştırma ekranı ve current-line yürütme", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, ayna özeti", "yerel statik sunucu ve yardımcı API"],
  mn: ["нэг орох цэг", "ажиллуулах дэлгэц ба current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror summary", "локал static server ба helper API"],
  ay: ["maya mantaña chiqapa", "run screen ukat current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror resumen", "local static server ukat helper API"],
  eu: ["sarrera puntu bakarra", "exekuzio pantaila eta current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror laburpena", "zerbitzari estatiko lokala eta helper API"],
  kn: ["ಒಂದು entry point", "run screen ಮತ್ತು current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror summary", "local static server ಮತ್ತು helper API"],
  ne: ["एकल entry point", "run screen र current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror summary", "local static server र helper API"],
  qu: ["huk yaykuna", "run screen chaymanta current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror resumen", "local static server chaymanta helper API"],
  sym3: ["single_entry", "run_screen + current_line", "console/graph/space2d/grid render", "madi + runtime_state + mirror_summary", "local_static_server + helper_API"],
  ta: ["ஒற்றை entry point", "run screen மற்றும் current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror summary", "local static server மற்றும் helper API"],
  te: ["ఒక entry point", "run screen మరియు current-line execution", "console/graph/space2d/grid Bogae rendering", "madi, runtime state, mirror summary", "local static server మరియు helper API"],
  zh: ["单一入口", "运行画面与 current-line 执行", "console/graph/space2d/grid Bogae 渲染", "madi、runtime state、mirror 摘要", "本地静态服务器与辅助 API"],
  es: ["punto de entrada único", "pantalla de ejecución y ejecución current-line", "render de Bogae console/graph/space2d/grid", "madi, estado runtime y resumen mirror", "servidor estático local y API auxiliar"],
  fr: ["point d'entrée unique", "écran d'exécution et exécution current-line", "rendu Bogae console/graph/space2d/grid", "madi, état runtime et résumé mirror", "serveur statique local et API auxiliaire"],
  de: ["einziger Einstiegspunkt", "Ausführungsansicht und current-line Ausführung", "Bogae-Rendering für console/graph/space2d/grid", "madi, runtime state und mirror summary", "lokaler statischer Server und Hilfs-API"],
};

const releaseLines = {
  ko: ["age_target < AGE2에서는 'open=record|replay'가 차단됩니다.", "'--unsafe-open' 우회 옵션이 추가되었습니다.", "open log 스키마:", "pack:"],
  en: ["'open=record|replay' is blocked when 'age_target < AGE2'.", "'--unsafe-open' was added as an explicit bypass.", "open log schemas:", "packs:"],
  ja: ["age_target < AGE2 では 'open=record|replay' がブロックされます。", "'--unsafe-open' が明示的な迂回として追加されました。", "open log schema:", "pack:"],
  tr: ["age_target < AGE2 iken 'open=record|replay' engellenir.", "'--unsafe-open' açık bypass olarak eklendi.", "open log schema'ları:", "pack'ler:"],
  mn: ["age_target < AGE2 үед 'open=record|replay' хоригдоно.", "'--unsafe-open' ил bypass байдлаар нэмэгдсэн.", "open log schemas:", "packs:"],
  ay: ["age_target < AGE2 ukax 'open=record|replay' jark'iwa.", "'--unsafe-open' qhana bypass ukham yapxatata.", "open log schemas:", "packs:"],
  eu: ["age_target < AGE2 denean 'open=record|replay' blokeatzen da.", "'--unsafe-open' bypass esplizitu gisa gehitu da.", "open log schemak:", "packak:"],
  kn: ["age_target < AGE2 ಆಗಿದ್ದರೆ 'open=record|replay' ತಡೆಹಿಡಿಯಲಾಗುತ್ತದೆ.", "'--unsafe-open' explicit bypass ಆಗಿ ಸೇರಿಸಲಾಗಿದೆ.", "open log schemas:", "packs:"],
  ne: ["age_target < AGE2 हुँदा 'open=record|replay' रोकिन्छ।", "'--unsafe-open' explicit bypass का रूपमा थपियो।", "open log schemas:", "packs:"],
  qu: ["age_target < AGE2 kaptinqa 'open=record|replay' hark'asqa.", "'--unsafe-open' qhapaq bypass hina yapasqa.", "open log schemas:", "packs:"],
  sym3: ["age_target<AGE2 blocks 'open=record|replay'.", "'--unsafe-open' added as explicit bypass.", "open_log_schemas:", "packs:"],
  ta: ["age_target < AGE2 என்றால் 'open=record|replay' தடுக்கப்படும்.", "'--unsafe-open' explicit bypass ஆக சேர்க்கப்பட்டது.", "open log schemas:", "packs:"],
  te: ["age_target < AGE2 అయితే 'open=record|replay' నిరోధించబడుతుంది.", "'--unsafe-open' explicit bypass గా జోడించబడింది.", "open log schemas:", "packs:"],
  zh: ["当 age_target < AGE2 时，'open=record|replay' 会被阻止。", "'--unsafe-open' 已作为显式绕过选项加入。", "open log schema:", "pack:"],
  es: ["'open=record|replay' se bloquea cuando 'age_target < AGE2'.", "Se añadió '--unsafe-open' como bypass explícito.", "schemas de open log:", "packs:"],
  fr: ["'open=record|replay' est bloqué quand 'age_target < AGE2'.", "'--unsafe-open' a été ajouté comme contournement explicite.", "schemas open log :", "packs :"],
  de: ["'open=record|replay' wird blockiert, wenn 'age_target < AGE2' ist.", "'--unsafe-open' wurde als expliziter Bypass hinzugefügt.", "open-log-Schemas:", "packs:"],
};

const macLines = {
  ko: "macOS/Linux: 'chmod +x ./ddonirang-tool' 후 './ddonirang-tool --help'",
  en: "macOS/Linux: 'chmod +x ./ddonirang-tool' then './ddonirang-tool --help'",
  ja: "macOS/Linux: 'chmod +x ./ddonirang-tool' の後 './ddonirang-tool --help'",
  tr: "macOS/Linux: önce 'chmod +x ./ddonirang-tool', sonra './ddonirang-tool --help'",
  mn: "macOS/Linux: эхлээд 'chmod +x ./ddonirang-tool', дараа нь './ddonirang-tool --help'",
  ay: "macOS/Linux: nayraqata 'chmod +x ./ddonirang-tool', ukat './ddonirang-tool --help'",
  eu: "macOS/Linux: lehenik 'chmod +x ./ddonirang-tool', gero './ddonirang-tool --help'",
  kn: "macOS/Linux: ಮೊದಲು 'chmod +x ./ddonirang-tool', ನಂತರ './ddonirang-tool --help'",
  ne: "macOS/Linux: पहिले 'chmod +x ./ddonirang-tool', त्यसपछि './ddonirang-tool --help'",
  qu: "macOS/Linux: ñawpaq 'chmod +x ./ddonirang-tool', chaymanta './ddonirang-tool --help'",
  sym3: "macOS/Linux: 'chmod +x ./ddonirang-tool' -> './ddonirang-tool --help'",
  ta: "macOS/Linux: முதலில் 'chmod +x ./ddonirang-tool', பின்னர் './ddonirang-tool --help'",
  te: "macOS/Linux: ముందుగా 'chmod +x ./ddonirang-tool', తర్వాత './ddonirang-tool --help'",
  zh: "macOS/Linux: 先运行 'chmod +x ./ddonirang-tool'，再运行 './ddonirang-tool --help'",
  es: "macOS/Linux: primero 'chmod +x ./ddonirang-tool', luego './ddonirang-tool --help'",
  fr: "macOS/Linux : d'abord 'chmod +x ./ddonirang-tool', puis './ddonirang-tool --help'",
  de: "macOS/Linux: zuerst 'chmod +x ./ddonirang-tool', dann './ddonirang-tool --help'",
};

const exampleLines = {
  ko: ["console-grid 최소 예제", "space2d 진자와 튕김 probe", "console-grid 테트리스", "수식/증명/람다 예제", "미로 probe"],
  en: common.examples,
  ja: ["console-grid 最小例", "space2d 振り子と反射 probe", "console-grid Tetris", "数式/証明/ラムダ例", "迷路 probe"],
  tr: ["console-grid en küçük örnek", "space2d sarkaç ve sekme probe", "console-grid Tetris", "formül/kanıt/lambda örnekleri", "labirent probe"],
  mn: ["console-grid хамгийн бага жишээ", "space2d дүүжин ба ойх probe", "console-grid Tetris", "томьёо/нотолгоо/lambda жишээ", "лабиринт probe"],
  ay: ["console-grid jisk'a uñacht'awi", "space2d pendulum ukat bounce probe", "console-grid Tetris", "formula/proof/lambda uñacht'awinaka", "ch'iqhi thakhi probe"],
  eu: ["console-grid gutxieneko adibidea", "space2d pendulua eta bounce probe", "console-grid Tetris", "formula/proof/lambda adibideak", "labirinto probe"],
  kn: ["console-grid ಕನಿಷ್ಠ example", "space2d pendulum ಮತ್ತು bounce probe", "console-grid Tetris", "ಸೂತ್ರ/proof/lambda examples", "maze ಪರಿಶೀಲನೆ"],
  ne: ["console-grid न्यूनतम example", "space2d pendulum र bounce probe", "console-grid Tetris", "सूत्र/proof/lambda examples", "भुलभुलैया probe"],
  qu: ["console-grid uchuy rikuchiy", "space2d pendulum chaymanta bounce probe", "console-grid Tetris", "formula/proof/lambda rikuchiykuna", "chinkana probe"],
  sym3: ["console_grid_min", "space2d_pendulum+bounce_probe", "console_grid_tetris", "formula/proof/lambda", "maze_probe"],
  ta: ["console-grid குறைந்தபட்ச example", "space2d pendulum மற்றும் bounce probe", "console-grid Tetris", "சூத்திரம்/proof/lambda examples", "maze ஆய்வு"],
  te: ["console-grid కనిష్ఠ example", "space2d pendulum మరియు bounce probe", "console-grid Tetris", "సూత్రం/proof/lambda examples", "maze పరిశీలన"],
  zh: ["console-grid 最小示例", "space2d 摆和反弹 probe", "console-grid Tetris", "公式/证明/lambda 示例", "迷宫 probe"],
  es: ["ejemplo mínimo console-grid", "péndulo space2d y bounce probe", "Tetris console-grid", "ejemplos de fórmula/prueba/lambda", "prueba de laberinto"],
  fr: ["exemple minimal console-grid", "pendule space2d et bounce probe", "Tetris console-grid", "exemples formule/preuve/lambda", "sonde de labyrinthe"],
  de: ["minimales console-grid Beispiel", "space2d Pendel und bounce probe", "console-grid Tetris", "Formel/Beweis/Lambda Beispiele", "Labyrinth-Probe"],
};

function write(file, text) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text.replace(/\r\n/g, "\n"), "utf8");
}

function codeBlock(lang, body) {
  return `~~~${lang}\n${body}\n~~~`;
}

function readme(l) {
  const mainDocs = {
    ko: "주요 문서",
    ja: "主要文書",
    tr: "Ana dokümanlar",
    mn: "Үндсэн баримтууд",
    ay: "Jach'a qillqatanaka",
    eu: "Dokumentu nagusiak",
    kn: "ಮುಖ್ಯ ದಾಖಲೆಗಳು",
    ne: "मुख्य कागजात",
    qu: "Hatun qillqakuna",
    sym3: "main_docs",
    ta: "முக்கிய ஆவணங்கள்",
    te: "ముఖ్య పత్రాలు",
    zh: "主要文档",
    es: "Documentos principales",
    fr: "Documents principaux",
    de: "Hauptdokumente",
  }[l.code] ?? "Main documents";
  const currentExamples = {
    ko: "현재 예제",
    ja: "現在の例",
    tr: "Güncel örnekler",
    mn: "Одоогийн жишээнүүд",
    ay: "Jichha uñacht'awinaka",
    eu: "Uneko adibideak",
    kn: "ಪ್ರಸ್ತುತ examples",
    ne: "हालका examples",
    qu: "Kunan rikuchiykuna",
    sym3: "current_examples",
    ta: "தற்போதைய examples",
    te: "ప్రస్తుత examples",
    zh: "当前示例",
    es: "Ejemplos actuales",
    fr: "Exemples actuels",
    de: "Aktuelle Beispiele",
  }[l.code] ?? "Current examples";
  const status = {
    ko: "번역 상태",
    ja: "翻訳状態",
    tr: "Çeviri durumu",
    mn: "Орчуулгын төлөв",
    ay: "Localización estado",
    eu: "Itzulpen egoera",
    kn: "translation status",
    ne: "translation status",
    qu: "localización estado",
    sym3: "localization_status",
    ta: "translation status",
    te: "translation status",
    zh: "翻译状态",
    es: "Estado de traducción",
    fr: "État de traduction",
    de: "Übersetzungsstatus",
  }[l.code] ?? "Localization status";
  const examples = exampleLines[l.code] ?? common.examples;
  return `# Ddonilang (${l.code})

> ${l.note}

## ${l.quick[0]}

- ${l.quick[1]}: ${l.quick[2]}
- ${l.quick[4]}: ${l.quick[6]} http://localhost:8787/
- ${l.dev[15]}: ${l.dev[16]}

## ${mainDocs}

- QUICKSTART.md
- DEV_STRUCTURE.md
- DOWNLOADS.md
- RELEASE_NOTES_20260211.md
- ../../README.md
- ../../README_en.md

## ${currentExamples}

${examples.map((x) => `- ${x}`).join("\n")}

## ${status}

${l.note}
`;
}

function quickstart(l) {
  return `# ${l.quick[0]} (${l.name})

> ${l.note}

## 1. ${l.quick[1]}

${l.quick[2]}

${codeBlock("sh", "cargo build --release")}

${l.quick[3]}

${codeBlock("sh", "cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help")}

## 2. ${l.quick[4]}

${l.quick[5]}

${codeBlock("sh", "python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py")}

${l.quick[6]}

${codeBlock("txt", "http://localhost:8787/")}

${l.quick[7]} 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. ${l.quick[8]}

${codeBlock("sh", common.commands.join("\n"))}

## 4. ${l.quick[9]}

${l.quick[10]}

- Windows: '.\\ddonirang-tool.exe --help'
- ${macLines[l.code] ?? macLines.en}
`;
}

function devStructure(l) {
  const fileLines = uiFileLines[l.code] ?? uiFileLines.en;
  const rows = [
    ["core", "core/", l.dev[6]],
    ["lang", "lang/", l.dev[7]],
    ["tool", "tool/", l.dev[8]],
    ["CLI", "tools/teul-cli/", l.dev[9]],
    ["packs", "pack/", l.dev[10]],
    ["Seamgrim", "solutions/seamgrim_ui_mvp/", l.dev[11]],
    ["tests", "tests/", l.dev[12]],
    ["publish", "publish/", l.dev[13]],
  ];
  return `# ${l.dev[0]} (${l.name})

> ${l.note}

${l.dev[1]}

## ${l.dev[2]}

| ${l.dev[3]} | ${l.dev[4]} | ${l.dev[5]} |
| --- | --- | --- |
${rows.map((r) => `| ${r[0]} | '${r[1]}' | ${r[2]} |`).join("\n")}

## ${l.dev[14]}

- 'ui/index.html': ${fileLines[0]}
- 'ui/screens/run.js': ${fileLines[1]}
- 'ui/components/bogae.js': ${fileLines[2]}
- 'ui/seamgrim_runtime_state.js': ${fileLines[3]}
- 'tools/ddn_exec_server.py': ${fileLines[4]}

## ${l.dev[15]}

- ${l.dev[16]}
- ${l.dev[17]}
- ${l.dev[18]}

## ${l.dev[19]}

- ${l.dev[20]}
- ${l.dev[21]}
- ${l.dev[22]}
- ${l.dev[23]}
`;
}

function downloads(l) {
  return `# ${l.down[0]} (${l.name})

> ${l.note}

## ${l.down[1]}

- ${l.down[2]}
- ${l.down[3]}

## ${l.down[4]}

- Windows x64
- macOS x64/arm64
- Linux x64/arm64

## ${l.down[5]}

- 'ddonirang-tool-<version>-windows-x64.zip'
- 'ddonirang-tool-<version>-macos-arm64.zip'
- 'ddonirang-tool-<version>-linux-x64.tar.gz'

## ${l.down[6]}

${codeBlock("txt", "ddonirang-tool-<version>-<os>-<arch>/\n  ddonirang-tool(.exe)\n  LICENSE\n  NOTICE.txt\n  README.txt")}

## ${l.down[7]}

${l.down[8]}

## ${l.down[9]}

${l.down[10]}
`;
}

function releaseNotes(l) {
  const lines = releaseLines[l.code] ?? releaseLines.en;
  return `# ${l.rel[0]} (${l.name})

> ${l.rel[1]}

## ${l.rel[2]}

${l.rel[3]}

## ${l.rel[4]}

- ${lines[0]}
- ${lines[1]}
- ${lines[2]}
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- ${lines[3]}
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## ${l.rel[5]}

${l.rel[6]}

## ${l.rel[7]}

${codeBlock("sh", "python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay")}

## ${l.rel[8]}

${l.rel[9]}
`;
}

fs.mkdirSync(i18nRoot, { recursive: true });

const rows = [];
for (const l of locales) {
  const dir = path.join(i18nRoot, l.code);
  write(path.join(dir, "README.md"), readme(l));
  write(path.join(dir, "QUICKSTART.md"), quickstart(l));
  write(path.join(dir, "DEV_STRUCTURE.md"), devStructure(l));
  write(path.join(dir, "DOWNLOADS.md"), downloads(l));
  write(path.join(dir, "RELEASE_NOTES_20260211.md"), releaseNotes(l));
  rows.push(`| ${l.code} | ${l.name} | publish/i18n/${l.code}/README.md | starter/localized |`);
}

write(path.join(i18nRoot, "INDEX.md"), `# publish/i18n index

This folder contains localized public document sets. Command names, file paths, package names, and runtime contracts stay canonical across languages.

| Code | Name | Entry | Status |
| --- | --- | --- | --- |
${rows.join("\n")}

## Included documents

Each language folder contains:

- README.md
- QUICKSTART.md
- DEV_STRUCTURE.md
- DOWNLOADS.md
- RELEASE_NOTES_20260211.md

## Boundary

These files are public localization guides. They do not own DDN language/runtime truth.
`);

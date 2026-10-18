# 세 파일 누적 작품 — Native 저장·이어 실행

`root.ddn`이 시작 정의와 두 모듈을 지정합니다. `lib/힘.ddn`의 값은 3이고,
`app/main.ddn`은 한 마디마다 누적 값에 3을 더합니다. 첫 마디는 3, 다음 마디는 6입니다.

동봉한 `authored-project.json`은 이 원문을 실제 Rust/WASM 제품의 명시적인 작품 만들기로
저장한 자료입니다. 작품 정체는 파일 안에 보이며, 파일 경로나 이름으로 만들지 않습니다.
`initial-input.json`과 `next-input.json`은 이 원문에 결속된 마디 0·1의 입력입니다.
키 행동은 없습니다. 씨앗은 7, 화면 속도는 60마디/초, 세계 dt는 1입니다.
이 입력 프로필 이름들은 이 예제의 결속값이지 새 언어 표준 이름이 아닙니다.

## 저장하기

ZIP을 새 폴더에 푼 뒤, 그 폴더에서 PowerShell을 엽니다. 아래는 기존 저장본을
덮어쓰지 않도록 `누적저장` 폴더가 이미 있으면 중단합니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_counter'
$store = Join-Path (Get-Location) '누적저장'
if (Test-Path -LiteralPath $store) { throw '이미 있는 저장 폴더입니다. 다른 새 폴더를 쓰세요.' }
New-Item -ItemType Directory -Path $store | Out-Null
$resultText = & ./bin/ddonilang.exe 작품자료 논리기록 "$example/authored-project.json" --입력 "$example/initial-input.json" --부모 처음 --판 02 --저장소 "$store" --저장소-초기화
if ($LASTEXITCODE -ne 0) { throw '저장 실패: 성공 자료를 쓰지 않습니다.' }
$resultText = $resultText -join "`n"
$result = $resultText | ConvertFrom-Json
[System.IO.File]::WriteAllText("$store/result.json", $resultText, $utf8)
[System.IO.File]::WriteAllText("$store/saved.json", $result.saved_artifact_json_utf8, $utf8)
$result.summary.resources.fixed64.누적
```

마지막 출력은 **3**입니다. SQLite 저장과 그 영수증은 제품이 만들며,
PowerShell은 제품이 반환한 원문 그대로를 안내 파일에 보관할 뿐입니다.

## 닫고 다시 이어 실행하기

PowerShell 창을 닫아도 됩니다. ZIP 폴더에서 새 창을 열고 실행합니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_counter'
$store = Join-Path (Get-Location) '누적저장'
$result = Get-Content -LiteralPath "$store/result.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$continuedText = & ./bin/ddonilang.exe 작품자료 이어실행 "$store/saved.json" --부모 처음 --판 02 --마디수 1 --저장소 "$store" --거래 $result.transaction_digest_sha256 --입력 "$example/next-input.json"
if ($LASTEXITCODE -ne 0) { throw '이어 실행 실패: 성공으로 처리하지 않습니다.' }
$continued = ($continuedText -join "`n") | ConvertFrom-Json
$continued.resources.fixed64.누적
```

마지막 출력은 **6**입니다. `ddonilang.exe` 대신 `ddonirang.exe`나 `또니랑.exe`를
사용해도 같습니다. 이어 실행만으로 기존 저장본이 다음 판으로 바뀌지는 않습니다.

## 이 예제가 보장하지 않는 것

이 안내는 Native SQLite 경로입니다. 브라우저 저장과 저장소를 혼동하지 마세요.
테트리스 저장은 옆의 `beta1_logical_tetris/README.ko.md`를 따르세요.
손상 자료는 임의로 고치거나 부분 성공으로 처리하지 않습니다.

원문을 바꾸면 이 입력의 원문 결속도 다시 만들어야 합니다. 동봉한 JSON의 해시만
손으로 바꾸어 통과시키지 마세요. 별개의 작품 정체가 필요하면 기존 명시 복제 기능을
사용하며, 파일을 복사하거나 이름을 바꾸는 것만으로 새 작품 정체가 생기지 않습니다.

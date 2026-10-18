# 테트리스 — 저장하고 닫은 뒤 이어 하기

`root.ddn`은 작품의 시작을, `main.ddn`은 기존 차림·셈씨로 만든 칸 규칙을 담습니다.
`authored-project.json`은 이 두 원문을 제품의 작품 만들기로 묶은 자료입니다.
`initial-input.json`은 0~2마디, `next-input.json`은 3~6마디의 입력입니다.
같은 씨앗과 같은 순서로 왼쪽 이동·회전·떨어뜨리기·오른쪽 이동을 처리합니다.
입력 JSON의 결속 해시는 손으로 고치지 마세요.

## 3마디를 저장하기

ZIP을 새 폴더에 풀고 그 폴더에서 PowerShell을 엽니다.
기존 자료를 덮어쓰지 않도록 아래는 새 `테트리스저장` 폴더만 사용합니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris'
$store = Join-Path (Get-Location) '테트리스저장'
if (Test-Path -LiteralPath $store) { throw '새 저장 폴더를 사용하세요.' }
New-Item -ItemType Directory -Path $store | Out-Null
$text = & ./bin/ddonilang.exe 작품자료 논리기록 "$example/authored-project.json" --입력 "$example/initial-input.json" --부모 처음 --판 02 --저장소 "$store" --저장소-초기화
if ($LASTEXITCODE -ne 0) { throw '저장 실패: 성공 자료를 게시하지 않습니다.' }
$text = $text -join "`n"
$saved = $text | ConvertFrom-Json
[System.IO.File]::WriteAllText("$store/result.json", $text, $utf8)
[System.IO.File]::WriteAllText("$store/saved.json", $saved.saved_artifact_json_utf8, $utf8)
$saved.summary.state_hash
```

## 새 프로세스에서 4마디 이어 실행하기

창을 닫고 ZIP 폴더에서 새 PowerShell을 열어도 됩니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris'
$store = Join-Path (Get-Location) '테트리스저장'
$saved = Get-Content -LiteralPath "$store/result.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$text = & ./bin/ddonilang.exe 작품자료 이어실행 "$store/saved.json" --부모 처음 --판 02 --마디수 4 --저장소 "$store" --거래 $saved.transaction_digest_sha256 --입력 "$example/next-input.json"
if ($LASTEXITCODE -ne 0) { throw '이어 실행 실패' }
$continued = ($text -join "`n") | ConvertFrom-Json
$continued.state_hash
```

0~6번 마디를 같은 입력으로 연속 실행한 결과와 마지막 `state_hash` 및 자원 값이 같아야 합니다.
이 예제의 마지막 해시는 `blake3:c7c9359477b58adc903fe72ad9b1c9105b5da8158db169842ef5a36d4468202d`입니다.
`ddonirang.exe`·`또니랑.exe`도 같은 명령을 받습니다. 이어 실행만으로 저장 머리가 갱신되지는 않습니다.

## 브라우저에서 저장·다시 열기

여기서 말하는 저장은 **작품 저장**입니다. 입력 패널의 **기록 중단·저장**과 다릅니다.
작품 저장은 원문·실행 상태를 보존해 다시 열고 이어 실행하기 위한 것입니다.
live 입력을 명시적으로 중단한 V3 파일은 `Incomplete`, 실제 실패 전 입력은 `Aborted`로 보존되며 정상 완료 기록으로 재생하지 않습니다.
이 파일을 불러오는 데 실패했다고 기존 작품·VM을 먼저 닫거나 저장본을 덮어쓰지는 않습니다.

다음 통합 V3 개발 후보에서 유한 입력을 정상 기록·재생하려면, 원래 `initial-input.json`처럼 끝이 정해진 요청을 CLI의 `논리입력기록`에 전달합니다.
여러 파일 작품은 `--authored`와 `authored-project.json`을 명시하며, 재생할 때도 같은 작품 자료와 원래 입력 계약을 별도로 전달합니다.
기본 운반 형식은 V3이며 옛 V1·잠정 V2 파일을 자동 탐지해 대신 읽지 않습니다.
입력 파일 하나만으로 필요한 모든 실행 정체·저장 문맥이 복원된다는 뜻은 아닙니다. 이전 ZIP의 지원 범위는 그 ZIP 안내를 따릅니다.

작업대에서 이 `authored-project.json`을 가져오고, 논리 입력의 테트리스 계약과 키 사상을 명시 선택합니다.
▶로 실행하고 일시정지한 뒤 저장합니다. 브라우저를 닫았다가 다시 열면 같은 작품과 입력 계약을 선택하고 저장 상태를 다시 엽니다.
복원 직후에도 판과 현재 조각이 보여야 하며, 한 마디를 누르면 이어집니다.
CLI SQLite 저장소를 브라우저 저장소처럼 자동으로 여는 기능은 아닙니다.

글·참거짓은 정식 자원 값으로 저장합니다. 옛 `resource_json` 저장본은 자동 변환하지 않고 거부합니다.
저장 머리를 바꾸기 전 실패는 기존 저장본을 유지합니다. 바꾼 뒤 확인 실패는 `RecoveryRequired`이며 성공으로 게시하지 않습니다.
정전 복구·RD-669 전체 내구 영수증·Human 검증은 이 안내의 보장 범위가 아닙니다.

## 개발 후보의 복구 기록 경로 — 새 저장소의 첫 저장만

아래는 기존 저장소를 바꾸지 않고 별도의 새 저장소에서 복구 확인을 시험하는 고급 실습입니다.
명령 이름은 잠정이며, 현재는 첫 저장과 그 저장본의 복구·이어 실행만 지원합니다.
같은 저장소에 다음 판을 저장하는 기능과 전체 내구성 보장이 완성됐다는 뜻은 아닙니다.

ZIP 폴더에서 새 PowerShell을 열고 실행합니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris'
$store = Join-Path (Get-Location) '테트리스복구저장'
$prepared = Join-Path (Get-Location) '테트리스복구자료'
if ((Test-Path -LiteralPath $store) -or (Test-Path -LiteralPath $prepared)) { throw '별도의 새 폴더 이름을 사용하세요.' }
New-Item -ItemType Directory -Path $store | Out-Null
$text = & ./bin/ddonilang.exe 작품자료 내구기록 "$example/authored-project.json" --입력 "$example/initial-input.json" --판 09 --저장소 "$store" --저장소-초기화 --복구자료 "$prepared"
if ($LASTEXITCODE -ne 0) { throw '저장이 완료되지 않았습니다. 준비 자료가 있어도 성공이 아닙니다. 자료를 보존하고 진단을 확인하세요.' }
$saved = ($text -join "`n") | ConvertFrom-Json
$saved.summary.state_hash
```

창을 닫고 새 PowerShell에서 같은 ZIP 폴더를 연 다음 이어 실행합니다.
복구 자료는 원요청·토큰·원문·허가의 사본일 뿐, 그 자체가 저장 성공 영수증은 아닙니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris'
$store = Join-Path (Get-Location) '테트리스복구저장'
$prepared = Join-Path (Get-Location) '테트리스복구자료'
$grant = (Get-Content -LiteralPath "$prepared/grant.txt" -Raw -Encoding UTF8).Trim()
$token = (Get-Content -LiteralPath "$prepared/token.hex" -Raw -Encoding UTF8).Trim()
$text = & ./bin/ddonilang.exe 작품자료 복구 "$prepared/saved.json" --판 09 --저장소 "$store" --허가 $grant --요청 "$prepared/request.bin" --토큰 $token --입력 "$example/next-input.json"
if ($LASTEXITCODE -ne 0) { throw '복구 확인에 실패했습니다. 자료를 지우거나 새 허가로 덮지 말고 진단을 확인하세요.' }
$continued = ($text -join "`n") | ConvertFrom-Json
$continued.state_hash
```

정상 경로의 결과는 위의 연속 7마디 해시와 같습니다. 잘못된 씨앗·다른 입력 종류·불완전한 확인 기록은 거부합니다.
이 경로도 브라우저 저장소를 대신하지 않으며, 정전·장치 고장·전체 장애 복구의 완료 판정은 별도입니다.

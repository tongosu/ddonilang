# 위치형 값꾸러미 — 씨를 지키며 저장하고 다시 열기

`정수묶음`과 `소수묶음`에는 똑같이 `7`, `"가"`, `참`이 들어갑니다. 하지만 첫 값의 선언은 각각 `바른수`와 `셈수`입니다. 같은 숫자처럼 보여도 저장한 위치별 씨는 구분됩니다.

빈 값꾸러미, 중첩, 정적 위치 선택과 반열림 범위 선택도 원문에 있습니다. 실행할 때마다 `진행`은 하나 증가합니다. 3마디 뒤 저장하고 4마디를 이어 실행하면 `진행`은 7이어야 합니다.

`declared_call.ddn`은 같은 원리를 셈씨 호출로 확인하는 별도 원문입니다. `정수전달`의 결과씨는 `바른수`, `실수전달`은 `셈수`로 명시합니다. 둘 다 7을 돌려줘도 Tuple의 위치씨는 다릅니다. 이 원문으로 작품을 만들려면 새 작품의 `main.ddn` 내용으로 넣고 기존 `root.ddn`을 사용하세요. 이전 작품 봉인이나 저장본을 손으로 고치지는 마세요. 이 예제는 주석 없는 수 리터럴 기본씨 결정을 대신하지 않습니다.

## 브라우저

`authored-project.json`을 작업대로 가져옵니다. ▶로 실행하고 일시정지한 뒤 저장합니다. 브라우저를 닫았다가 다시 열고 같은 작품과 저장 상태를 연 뒤 한 마디 더 진행해 보세요. 글과 참거짓, 원소 순서와 위치별 씨가 유지되어야 합니다.

`create-request.json`은 제품의 작품 만들기에 사용한 입력입니다. `root.ddn`과 `main.ddn`을 수정한 경우에는 이 입력도 같은 원문으로 바꾸고 새 작품을 명시적으로 만들어야 합니다. 저장된 정체나 봉인을 손으로 고치지 마세요.

## CLI에서 3마디 저장

ZIP 폴더에서 PowerShell을 열고 새 저장 폴더를 사용합니다.

```powershell
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8
$example = Join-Path (Get-Location) 'solutions/seamgrim_ui_mvp/samples/beta1_typed_tuple'
$store = Join-Path (Get-Location) '위치형저장'
if (Test-Path -LiteralPath $store) { throw '새 저장 폴더를 사용하세요.' }
New-Item -ItemType Directory -Path $store | Out-Null
$text = & ./bin/ddonilang.exe 작품자료 실행 "$example/authored-project.json" --마디수 3 --저장소 "$store" --부모 처음 --판 09 --저장소-초기화
if ($LASTEXITCODE -ne 0) { throw '저장 실패' }
$text = $text -join "`n"
$saved = $text | ConvertFrom-Json
[System.IO.File]::WriteAllText("$store/result.json", $text, $utf8)
[System.IO.File]::WriteAllText("$store/saved.json", $saved.saved_artifact_json_utf8, $utf8)
```

## 닫고 다시 열어 4마디 이어 실행

새 PowerShell에서 같은 ZIP 폴더로 이동합니다.

```powershell
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$store = Join-Path (Get-Location) '위치형저장'
$saved = Get-Content -LiteralPath "$store/result.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$text = & ./bin/ddonilang.exe 작품자료 이어실행 "$store/saved.json" --부모 처음 --판 09 --마디수 4 --저장소 "$store" --거래 $saved.transaction_digest_sha256 --일반
if ($LASTEXITCODE -ne 0) { throw '이어 실행 실패' }
$continued = ($text -join "`n") | ConvertFrom-Json
$continued.next_tick_id
$continued.resources.fixed64.진행
```

둘 다 7이어야 합니다. `ddonirang.exe`와 `또니랑.exe`도 같은 명령을 받습니다. 계속 실행한 기준과 비교할 때는 원문·씨앗·입력 조건도 같아야 합니다.

## 한정 지원 범위

현재는 원문에서 명시한 씨, 같은 결속의 이름, 결과씨가 명시된 셈씨 호출, 위치형 생성, 정적 선택·범위, 같은 위치씨의 교체로 근거가 이어지는 저장 경로를 지원합니다. 값의 모양을 보고 씨를 지어내지 않습니다. 타입 공급이 없는 리터럴·결과씨가 없는 호출·동적 선택이나 모호한 결속의 저장은 명시 거부합니다. 순서를 차림이나 짝맞춤으로 바꿔 저장하지 않습니다.

이 저장 실습은 명시적인 얽음 뿌리와 시작 정의가 있는 작품 자료를 사용합니다. 검증된 저장 원문·시작 문맥을 공급하지 않는 독립 상태 JSON은 복원하지 않습니다. 원문만 보관하는 것과 실행 상태까지 저장하는 것은 구분합니다.

이 예제의 저장 검증은 정전 복구·RD-669 전체 영수증·베타1 전체 완료나 Human PASS를 뜻하지 않습니다.

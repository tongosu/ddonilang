"""Fixed-source Alpha/Beta candidate assembler; never grants review/Human/public PASS.

No checkout reads for source payloads. Baseline is an explicit exact prior ZIP.
Runtime binaries need an explicit build identity with exact member hashes.
"""
import argparse
import hashlib
import json
import posixpath
import re
import subprocess
from pathlib import Path
import zipfile

from alpha1_r20_archive import (
    FIRST_STEPS, UI, first_steps_project_assets, is_retired_ui_path,
    verify_first_steps_catalog, verify_first_launch_instructions,
)

PACKAGE = 'DDN-1.0.0-alpha.2-windows-x64'
VERSIONS = ('1.0.0-alpha.2', '1.0.0-alpha.3', '1.0.0-alpha.4', '1.0.0-beta.1')
ALPHA3_OR_LATER = VERSIONS[1:]
ALPHA4_OR_LATER = VERSIONS[2:]
BASE = 'solutions/seamgrim_ui_mvp/'
WASM = BASE + 'ui/wasm/'
SAMPLES = BASE + 'samples/'
ALPHA2_COURSE = BASE + 'first_steps_alpha2/'
ALPHA3_COURSE = BASE + 'first_steps_alpha3/'
ALPHA4_COURSE = BASE + 'first_steps_alpha4/'
BETA1_COURSE = BASE + 'first_steps_beta1/'
ALPHA4_SAMPLES = tuple(SAMPLES + name for name in (
    'alpha4_lrl_lab.ddn', 'alpha4_lrl_lab.intent.json',
    'alpha4_lrl_lab.project.ddn', 'alpha3_double_pendulum_lab.project.ddn',
    'alpha4_drawing_common.ddn',
    'alpha4_manipulation_point.ddn', 'alpha4_manipulation_point.binding.json',
    'alpha4_manipulation_point.action_map.json'))
BETA1_SAMPLES = tuple(SAMPLES + 'beta1_logical_counter/' + name for name in (
    'root.ddn', 'app/main.ddn', 'lib/힘.ddn', 'authored-project.json',
    'initial-input.json', 'next-input.json', 'README.ko.md')) + tuple(
    SAMPLES + 'beta1_logical_tetris/' + name for name in (
    'root.ddn', 'main.ddn', 'authored-project.json', 'initial-input.json', 'next-input.json', 'README.ko.md')) + tuple(
    SAMPLES + 'beta1_typed_tuple/' + name for name in (
    'root.ddn', 'main.ddn', 'declared_call.ddn', 'create-request.json', 'authored-project.json', 'README.ko.md'))
ALPHA3_SAMPLES = tuple(SAMPLES + name for name in (
    'alpha3_function_lab.ddn', 'alpha3_function_lab.intent.json',
    'alpha3_double_pendulum_lab.ddn', 'alpha3_double_pendulum_lab.intent.json',
    'alpha3_market_lab.ddn', 'alpha3_market_lab.intent.json',
    'alpha3_numeric_display.ddn', 'alpha3_units.ddn',
    'alpha3_chaebi_once.ddn', 'alpha3_chaebi_empty.ddn',
    'alpha3_scene_time.ddn', 'alpha3_scene_relative.ddn', 'alpha3_table_time.ddn')) + (
    'samples/alpha3_logical_reaction.ddn', 'samples/alpha3_basic_tensor.ddn')
REQUIRED_SAMPLES = tuple(SAMPLES + name for name in (
    'alpha2_logical_tetris.ddn', 'alpha2_logical_tetris.binding.json',
    'alpha2_logical_tetris.action_map.json', 'alpha2_logical_tetris.intent.json',
    'alpha2_parabola_composition.ddn', 'alpha2_pendulum_composition.ddn',
    'alpha2_linear_composition.ddn'))
NATIVES = ('ddonirang.exe', 'ddonilang.exe', '또니랑.exe')
# createWasmLoader resolves wrapperPath relative to wasm_page_common.js and
# imports the variable at runtime. It is a live module, not retired app.js UI.
DYNAMIC_MODULES = (BASE + 'ui/wasm_ddn_wrapper.js',)
METADATA = {'PACKAGE_ALLOWLIST.json', 'RELEASE_MANIFEST.json', 'SHA256SUMS.txt'}
REFS = re.compile(r'''(?:\bfrom\s*|\bimport\s*\(?\s*|\bnew\s+(?:URL|Worker)\s*\(\s*|\bfetch\s*\(\s*|\b(?:src|href)\s*=\s*|\burl\s*:\s*|\burl\(\s*)["']([^"']+)["']''')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encode(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode('utf-8')


def _beta1_progress_notes(text, identity):
    """Update only the explicitly bound successor's user-facing notes.

    These notes never grant product/review/SSOT authority. Historical builds
    without the exact baseline and INITIAL marker retain their own wording.
    """
    if identity.get('vc13_fixed_baseline') == 'dffd495d5eb674720090c5f2720de49820d96cf0':
        text = text.replace(
            '전체 VC13 행 폐쇄는 별도 검토입니다.',
            'VC13 필수 범위는 r55 고정 대상에서 검토·정본 결속을 마쳤습니다. '
            '이번 누적 후보에 그 행 전체 PASS를 자동 상속하지 않으며 변경분 검토는 별도입니다.')
        text = text.replace(
            'VC13 B01 전체가 끝난 것은 아닙니다. 정수 리터럴의 셈수 자리 사용과 혼합 리터럴 정책은 후속 SSOT 결정 전까지 기존 실행을 유지합니다.',
            '수 리터럴과 혼합 처리는 이 고정 계열의 승인된 규칙을 따릅니다.')
    if identity.get('m2_initial_custody') is True:
        if identity.get('scope') not in (
                'CUMULATIVE_R57_PLUS_M2_INITIAL_NATIVE_CUSTODY_ONLY; NOT_FULL_M2_OR_COMPLETE_VC04',
                'CUMULATIVE_R58_PLUS_VC04_SOURCE_LIFECYCLE_PENDING_COMPLETE_ROW_REVIEW; NOT_FULL_M2'):
            raise ValueError('INITIAL 복구 안내의 한정 빌드 범위가 없습니다')
        text = text.replace(
            'Native 저장 도중 프로세스가 중단된 자료의 자동 복구는 아직 완료되지 않았습니다. 새 SQL 머리가 보이는 것만으로 저장 성공이나 이어 실행 권한이 생기지 않습니다.\n'
            '재열기에서 사용할 복구 기록·토큰의 영속 공급과 동일 요청 재시도 경로는 미완입니다. 이번 공통 저장 실행기 검증을 전체 복구 보장으로 읽지 마세요. 검사 전용 빌드의 네 단계 강제 종료 근거와 배포 CLI의 정상 저장·이어 실행 근거는 구분합니다.',
            '추가 실험 경로인 `작품자료 내구기록`·`작품자료 복구`는 새로 명시한 저장소의 첫 거래만 지원합니다. 기존 저장 명령과 자동으로 섞거나 이관하지 않습니다.\n'
            '이 경로는 저장 전에 원문·요청·토큰·허가의 복구 자료를 보관하고, 새 프로세스에서 검증된 일반·논리 실행 상태를 이어갑니다. 필요한 자료나 저장 장벽·기록이 불완전하면 성공을 게시하지 않습니다.\n'
            '같은 저장소의 후속 다른 거래와 모든 장애의 자동 복구는 아직 완료되지 않았습니다. 새 SQL 머리만으로 성공을 인정하지 않으며, 검사 전용 빌드의 중단 주입과 배포 CLI의 정상 저장·이어 실행 근거는 구분합니다. 정전 시험을 직접 하지 마세요.\n'
            '복구 자료 폴더에는 재개에 필요한 열쇠가 있으므로 공개하거나 임의로 지우지 마세요. 이 경로 역시 전체 RD-669 내구 영수증·Human·release PASS를 뜻하지 않습니다.')
    if identity.get('vc04_source_lifecycle') is True:
        if identity.get('scope') != 'CUMULATIVE_R58_PLUS_VC04_SOURCE_LIFECYCLE_PENDING_COMPLETE_ROW_REVIEW; NOT_FULL_M2':
            raise ValueError('VC04 원문 수명 안내의 한정 빌드 범위가 없습니다')
        text += ('\n## 임자·원자 원문의 Native 저장\n\n'
                 'Native 작업대에서 작품 정체를 명시적으로 만든 뒤 원문을 저장하고 ▶로 실행할 수 있습니다. '
                 '일시정지한 현재 세계를 상단 저장 단추로 다시 저장하고, 작업대와 브라우저를 종료한 뒤 '
                 '같은 기기의 작성 자료를 다시 열어 이어 실행할 수 있습니다.\n'
                 '임자만 새 대상을 만들며, 셈씨 안의 직접·간접 생성은 거부합니다. '
                 '폐기한 대상의 기록과 다음 번호, 원문에서 등록된 원자의 정체는 같은 세계 저장에 보존합니다.\n'
                 '이 경로는 Windows Native 보호 저장소를 실제로 확인합니다. '
                 'JSON 파일이나 브라우저의 자료만으로 임자 세계의 실행 허가를 대신하지 않습니다. '
                 '원문 변경, 처리하지 않은 입력, 저장 근거 누락·손상은 명시 거부하고 기존 저장본을 성공으로 덮지 않습니다.\n'
                 '이 후보의 VC04 행 전체 독립검토·Human·공개 승인은 아직 별도입니다. '
                 '전체 RD-669 내구성이나 모든 장애의 자동 복구를 뜻하지 않습니다.\n')
    if identity.get('vc04_temporary_source') is True:
        if identity.get('scope') != 'CUMULATIVE_R58_PLUS_VC04_SOURCE_LIFECYCLE_PENDING_COMPLETE_ROW_REVIEW; NOT_FULL_M2':
            raise ValueError('VC04 임시 원문 실행 안내의 한정 빌드 범위가 없습니다')
        text += ('\n## 한 파일 이름씨·원자·임자 실행\n\n'
                 '`ddonilang run a.ddn --source-profile v1-core-v25`로 원문을 직접 실행할 수 있습니다. '
                 '`ddonirang`과 `또니랑`도 같은 경로입니다. 작업대에서는 세부 메뉴의 v25 단일 원문 열기로 엽니다. '
                 '실행 전에 공통 Rust가 필요한 등록을 준비하므로 원문을 숨은 실행 함수로 감싸거나 별도 등록 명령을 쓰지 않습니다.\n'
                 '이 경로의 정체는 해당 실행 동안만 유지됩니다. 서로 독립적으로 실행한 임시 세계를 같은 저장 세계로 취급하지 않습니다. '
                 '임시 값을 저장 작품이나 이어 실행 권한으로 자동 승격하지 않습니다. 저장·종료·재열기는 위의 명시적 Native 작품 저장 경로를 사용하세요.\n')
    if identity.get('beta1_freeze_contract_candidate') == 'r65':
        text = text.replace(
            '고름씨 pattern의 exact-name 정렬은 PD-WO19-001 내부 canon 후보입니다. 정본 ParamPin 물리 comparator는 미완·미동결이며 출시 전 clean-break 대상입니다.',
            '등록된 고름씨 pattern은 선택한 받는자리의 정체 바이트 순으로 정렬합니다. 이름이나 원문 순서를 정체 대신 사용하지 않습니다. 이전 등록자료를 새 형식으로 몰래 고치지 않습니다.')
        text = text.replace(
            '글자 행동 입력은 미지원이며 잠정 입력 기록 이름·머리표·단계 철자는 동결되지 않았습니다.',
            '기본 입력 기록은 키·문자·수·좌표의 자리를 담는 V3 한 운반 형식입니다. 글자 행동의 실제 실행은 아직 미지원입니다. 옛 기록을 자동으로 바꿔 읽지 않으며 명시한 옛 형식 경로나 거부 진단을 사용합니다. 제품 후보의 구현과 Owner의 최종 표면 동결은 구별합니다.')
        text += ('\n## 이번 동결 계약 후보에서 확인할 것\n\n'
                 '조작단계는 설정·시작·이동·끝 중 하나를 글로 돌려줍니다. 다른 글과 비교하면 경고하지만 비교의 실행 뜻을 바꾸지는 않습니다.\n'
                 '`<<-` 파생 문법은 예약되어 있으며 사용하면 아직 지원하지 않는다고 명시 거부합니다. 실제 파생 실행·저장은 베타2 작업입니다.\n'
                 '화살표와 점선의 공용 셈씨를 기존 작품들이 함께 사용합니다. 새 표준 이름이나 함수 삭제는 하지 않았습니다.\n'
                 '실시간 기록의 중단·저장은 미완 기록을 보존하며 정상 재생을 거부합니다. 완료 기록은 입력 기록의 고급 설정에서 미리 정한 입력 요청 JSON을 열어 만듭니다. 모든 마디가 처리되고 현재 원문·독립 입력 계약과 맞아야 파일을 저장합니다. 실시간 미완 기록을 완료로 고치는 기능은 아닙니다.\n'
                 '이 안내는 새 ZIP의 검토 완료·표면 동결·공개·직접 스모크 통과를 뜻하지 않습니다. 검증 결과와 남은 실패는 배포 기록을 확인하세요.\n')
    return text


def native_workbench_payload(payload):
    """Project only release-selected bytes for the existing Rust static host.

    Keep the Python server for compatibility checks, but never invoke it from
    START. No development tree or alternative WASM is included here.
    """
    assets = {rel[len(BASE):]: data for rel, data in payload.items()
              if rel.startswith(BASE) and not rel.startswith(BASE + 'tools/')}
    # The Python release host filters this route at request time. A static
    # host must seal the same explicit membership, not expose the old index.
    if 'lessons/index.json' in assets:
        allow = json.loads(assets['lessons/active_allowlist.detjson'])
        if allow.get('schema') != 'seamgrim.active_lessons.allowlist.v1' or allow.get('mode') != 'reps_only':
            raise ValueError('Native 목록의 명시 허용 목록이 없습니다')
        ids = allow.get('lesson_ids')
        if not isinstance(ids, list) or not ids or len(set(ids)) != len(ids):
            raise ValueError('Native 목록의 허용 정체가 올바르지 않습니다')
        index = json.loads(assets['lessons/index.json'])
        rows = [row for row in index['lessons'] if row.get('id') in ids]
        if {row['id'] for row in rows} != set(ids):
            raise ValueError('Native 목록의 허용 정체가 누락됐습니다')
        assets['lessons/index.json'] = encode({**index, 'lessons': rows, 'deferred': []})
    assets['index.html'] = (
        '<!doctype html><html lang="ko"><meta charset="utf-8">'
        '<title>또니랑 작업대</title><script>'
        'location.replace("/ui/workbench/index.html"+location.search+location.hash);'
        '</script><a href="/ui/workbench/index.html">작업대 열기</a></html>\n'
    ).encode('utf-8')
    types = {'.wasm': 'application/wasm', '.js': 'text/javascript',
             '.json': 'application/json', '.detjson': 'application/json',
             '.ddn': 'text/plain; charset=utf-8', '.md': 'text/plain; charset=utf-8',
             '.html': 'text/html; charset=utf-8', '.css': 'text/css', '.pdf': 'application/pdf'}
    files = [{'path': rel, 'bytes': len(data), 'sha256': sha(data),
              'content_type': types.get(Path(rel).suffix.lower(), 'application/octet-stream'),
              'role': 'entry' if rel == 'index.html' else 'runtime'}
             for rel, data in sorted(assets.items())]
    assets['asset_manifest.detjson'] = encode({
        'schema': 'ddonirang.workbench_asset_manifest.v1', 'files': files})
    return {'bin/workbench/' + rel: data for rel, data in assets.items()}


def safe_path(rel):
    if (not rel or '\\' in rel or rel.startswith('/') or ':' in rel or
            any(part in ('', '.', '..') for part in rel.split('/'))):
        raise ValueError('안전하지 않은 압축 경로: ' + rel)
    return rel


def forbidden(rel):
    if rel.startswith('bin/workbench/'):
        tail = rel.removeprefix('bin/workbench/')
        return tail not in ('index.html', 'asset_manifest.detjson') and forbidden(BASE + tail)
    return (is_retired_ui_path(rel) or rel.startswith(('docs/', 'work/', '.git/', 'private/')) or
            'tetris' in rel.lower() and rel not in REQUIRED_SAMPLES and rel not in BETA1_SAMPLES)


def read_baseline(path, expected_sha):
    if sha(path.read_bytes()) != expected_sha:
        raise ValueError('Alpha1 기준 ZIP SHA-256 불일치')
    with zipfile.ZipFile(path) as archive:
        files = [item.filename for item in archive.infolist() if not item.is_dir()]
        roots = {name.split('/')[0] for name in files}
        if len(roots) != 1:
            raise ValueError('기준 ZIP 최상위 폴더가 하나여야 합니다')
        prefix = next(iter(roots)) + '/'
        payload = {safe_path(name[len(prefix):]): archive.read(name) for name in files}
        if len(payload) != len(files):
            raise ValueError('기준 ZIP 중복 회원')
    if set(json.loads(payload['PACKAGE_ALLOWLIST.json'])['paths']) != set(payload):
        raise ValueError('기준 ZIP 허용 목록 불일치')
    for line in payload['SHA256SUMS.txt'].decode('utf-8').splitlines():
        digest, rel = line.split('  ', 1)
        if sha(payload[safe_path(rel)]) != digest:
            raise ValueError('기준 ZIP 체크섬 불일치: ' + rel)
    return payload


def dependencies(payload):
    pending, visited = [UI + 'index.html', *DYNAMIC_MODULES], set()
    while pending:
        rel = pending.pop()
        if rel in visited:
            continue
        if rel not in payload or forbidden(rel):
            raise ValueError('작업대 의존 파일 누락 또는 금지: ' + rel)
        visited.add(rel)
        if Path(rel).suffix not in ('.html', '.js', '.mjs', '.css'):
            continue
        for ref in REFS.findall(payload[rel].decode('utf-8')):
            ref = ref.split('?')[0].split('#')[0]
            if ref.startswith(('./', '../')):
                pending.append(safe_path(posixpath.normpath(posixpath.join(posixpath.dirname(rel), ref))))
    return visited


def verify_payload(payload, retired_hashes=(), version='1.0.0-alpha.2'):
    if len({name.casefold() for name in payload}) != len(payload):
        raise ValueError('대소문자 중복 회원')
    for rel, data in payload.items():
        safe_path(rel)
        if forbidden(rel) or sha(data) in retired_hashes:
            raise ValueError('금지 파일 또는 옛 화면 동일 바이트 사본: ' + rel)
    native = {rel: data for rel, data in payload.items() if rel.startswith('bin/workbench/')}
    native_start = b'bin\\ddonilang.exe" workbench' in payload.get('START_SEAMGRIM.cmd', b'')
    if native or native_start:
        if version != '1.0.0-beta.1' or not native_start or native != native_workbench_payload(payload):
            raise ValueError('Native 작업대 자산과 시작 경로의 결속 불일치')
    index = payload[UI + 'index.html'].decode('utf-8')
    for marker in ('<title>', 'data-testid="product-version"'):
        line = next((line for line in index.splitlines() if marker in line), '')
        if version not in line:
            raise ValueError('TODO: 고정 제품의 title/product-version을 ' + version + '로 수정: ' + marker)
    required = set(REQUIRED_SAMPLES) | {WASM + 'ddonirang_tool_bg.wasm',
        WASM + 'ddonirang_tool.js', WASM + 'ddonirang_tool.manifest.detjson',
        FIRST_STEPS + '학생용_첫걸음_10차시.pdf', FIRST_STEPS + '교사용_안내와_해답.pdf'}
    if version == '1.0.0-beta.1':
        required.update(BETA1_SAMPLES)
    manifest = json.loads(payload[FIRST_STEPS + '검증목록.json'])
    required.update(first_steps_project_assets(manifest))
    verify_first_steps_catalog(manifest, payload[UI + 'first_steps_catalog.js'].decode('utf-8'))
    course_manifest = ALPHA2_COURSE + '검증목록.json'
    course_catalog = UI + 'first_steps_alpha2_catalog.js'
    if course_catalog in payload and course_manifest not in payload:
        raise ValueError('알파2 첫걸음 검증목록 누락')
    if course_manifest in payload:
        required.add(course_catalog)
        course = json.loads(payload[ALPHA2_COURSE + '검증목록.json'])
        if [row['id'] for row in course['lessons']] != list(range(11, 17)):
            raise ValueError('알파2 첫걸음 차시 누락')
        rows = course['cases'] + course['modifications']
        if len(rows) != 18:
            raise ValueError('알파2 첫걸음 원문은 18개여야 합니다')
        required.update(ALPHA2_COURSE + row['file'] for row in rows)
        required.update(ALPHA2_COURSE + row[key] for row in rows for key in ('binding_file', 'request_file') if key in row)
        required.update(ALPHA2_COURSE + name for name in ('입력/action_map.json', '학생용_첫걸음_11-16차시.pdf', '교사용_안내와_해답.pdf'))
        for row in rows:
            if sha(payload.get(ALPHA2_COURSE + row['file'], b'')) != row['source_sha256']:
                raise ValueError('알파2 교재 원문 hash 불일치: ' + row['id'])
    if version in ALPHA3_OR_LATER:
        required.update(ALPHA3_SAMPLES)
        required.update(ALPHA3_COURSE + name for name in (
            '검증목록.json', '학생용_첫걸음_17-20차시.pdf', '교사용_안내와_해답.pdf'))
        required.add(UI + 'first_steps_alpha3_catalog.js')
        course = json.loads(payload.get(ALPHA3_COURSE + '검증목록.json', b'{}'))
        lesson_ids = [row['id'] for row in course.get('lessons', [])]
        extended_course = lesson_ids == list(range(17, 29))
        if lesson_ids != list(range(17, 21)) and not extended_course:
            raise ValueError('알파3 첫걸음 17~20차시 누락')
        case_ids = [row['id'] for row in course.get('cases', [])]
        if case_ids[:6] != ['17A', '17B', '18A', '18B', '19A', '20A'] or (not extended_course and len(case_ids) != 6) or [row['id'] for row in course.get('modifications', [])] != ['MOD17']:
            raise ValueError('알파3 첫걸음 예제·수정실습 누락')
        if extended_course:
            if len(case_ids) != len(set(case_ids)) or not set(range(21, 29)) <= {row.get('lesson') for row in course.get('cases', [])}:
                raise ValueError('알파3 보충 21~28차시 실행 활동 누락 또는 중복 ID')
            required.update(ALPHA3_COURSE + name for name in (
                '학생용_첫걸음_21-28차시.pdf', '교사용_첫걸음_21-28차시.pdf'))
        for row in course.get('cases', []) + course.get('modifications', []):
            for item in row.get('project_files', []):
                required.add(safe_path(posixpath.normpath(ALPHA3_COURSE + item['file'])))
            for key in ('file', 'binding_file', 'request_file'):
                if key in row:
                    # Course references intentionally cross into shared samples.
                    rel = safe_path(posixpath.normpath(ALPHA3_COURSE + row[key]))
                    required.add(rel)
                    if key == 'file' and 'source_sha256' in row and sha(payload.get(rel, b'')) != row['source_sha256']:
                        raise ValueError('알파3 교재 원문 hash 불일치: ' + row['id'])
    if version in ALPHA4_OR_LATER:
        required.update(ALPHA4_SAMPLES)
        required.update(ALPHA4_COURSE + name for name in ('학생용_첫걸음_29-30차시.pdf', '교사용_첫걸음_29-30차시.pdf'))
        required.add(UI + 'first_steps_alpha4_catalog.js')
    if not required <= set(payload):
        raise ValueError('필수 동적 자산 누락: ' + ', '.join(sorted(required - set(payload))))
    if version == '1.0.0-beta.1':
        positional = SAMPLES + 'beta1_typed_tuple/'
        tuple_authored = json.loads(payload[positional + 'authored-project.json'])
        tuple_request = json.loads(payload[positional + 'create-request.json'])
        tuple_snapshot = {'root_path': 'root.ddn', 'entry_path': 'main.ddn',
            'files': [{'path': name, 'text': payload[positional + name].decode('utf-8')}
                      for name in ('root.ddn', 'main.ddn')]}
        if (tuple_authored.get('source_snapshot') != tuple_snapshot
                or tuple_request.get('source_snapshot') != tuple_snapshot):
            raise ValueError('위치형 저장 예제와 배포 원문이 다릅니다')
        tetris = SAMPLES + 'beta1_logical_tetris/'
        authored_tetris = json.loads(payload[tetris + 'authored-project.json'])
        if authored_tetris.get('source_snapshot') != {'root_path': 'root.ddn', 'entry_path': 'main.ddn',
                'files': [{'path': name, 'text': payload[tetris + name].decode('utf-8')}
                          for name in ('root.ddn', 'main.ddn')]}:
            raise ValueError('테트리스 저장 예제와 배포 원문이 다릅니다')
        prefix = json.loads(payload[tetris + 'initial-input.json'])
        suffix = json.loads(payload[tetris + 'next-input.json'])
        if (prefix.get('binding') != suffix.get('binding')
                or [row.get('madi') for row in prefix.get('snapshots', [])] != [0, 1, 2]
                or [row.get('madi') for row in suffix.get('snapshots', [])] != [3, 4, 5, 6]):
            raise ValueError('테트리스 저장 예제의 입력 결속 또는 순서가 다릅니다')
        counter = SAMPLES + 'beta1_logical_counter/'
        authored = json.loads(payload[counter + 'authored-project.json'])
        source_snapshot = authored.get('source_snapshot', {})
        expected_files = [{'path': name, 'text': payload[counter + name].decode('utf-8')}
                          for name in ('root.ddn', 'app/main.ddn', 'lib/힘.ddn')]
        if (authored.get('source_profile') != 'v1-core-v25'
                or source_snapshot != {'root_path': 'root.ddn', 'entry_path': 'app/main.ddn', 'files': expected_files}):
            raise ValueError('누적 작품의 저장 원문과 배포 원문이 다릅니다')
        first = json.loads(payload[counter + 'initial-input.json'])
        following = json.loads(payload[counter + 'next-input.json'])
        if (first.get('binding') != following.get('binding')
                or first.get('snapshots') != [{'madi': 0, 'events': [], 'held': {}}]
                or following.get('snapshots') != [{'madi': 1, 'events': [], 'held': {}}]
                or first['binding'].get('dt_raw') != 4294967296
                or first['binding'].get('madi_hz') != 60
                or first['binding'].get('channels') != {}):
            raise ValueError('누적 작품의 마디 입력 결속이 다릅니다')
    if sum(rel.startswith(FIRST_STEPS) and rel.endswith('.ddn') for rel in payload) != 57:
        raise ValueError('첫걸음 원문 회원은 정확히 57개여야 합니다')
    return {'static_dependencies': len(dependencies(payload)), 'first_steps_sources': 57,
            'dynamic_required': sorted(required), 'network_browser_validation': 'PENDING'}


def write_archive(target, payload, retired_hashes=(), version='1.0.0-alpha.2'):
    if target.exists():
        raise ValueError('출력 ZIP이 이미 있습니다. 덮어쓰지 않습니다')
    verify_payload(payload, retired_hashes, version)
    package = 'DDN-' + version + '-windows-x64'
    target.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(target, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for rel in sorted(payload):
            info = zipfile.ZipInfo(package + '/' + rel, (2026, 10, 9, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, payload[rel], compresslevel=9)
    with zipfile.ZipFile(target) as archive:
        files = [item.filename for item in archive.infolist() if not item.is_dir()]
        prefix = package + '/'
        if any(not name.startswith(prefix) for name in files):
            raise ValueError('ZIP 회원이 제품 폴더 밖입니다')
        actual = {name[len(prefix):]: archive.read(name) for name in files}
        if len(files) != len(actual) or actual != payload or archive.testzip() is not None:
            raise ValueError('실제 ZIP 바이트/회원 불일치')
        checks = verify_payload(actual, retired_hashes, version)
        startup = verify_first_launch_instructions(archive, package)
    return {'zip': str(target), 'sha256': sha(target.read_bytes()), 'bytes': target.stat().st_size,
            'checks': checks, 'startup': startup, 'independent_review': 'PENDING',
            'owner_human_smoke': 'PENDING', 'public_approval': 'PENDING'}


def read_notice_bundle(folder, lock_sha256, manifest_sha256):
    root = Path(folder).resolve()
    data = (root / 'THIRD_PARTY_NOTICES.json').read_bytes()
    if not manifest_sha256 or sha(data) != manifest_sha256:
        raise ValueError('제삼자 고지 manifest 해시 불일치')
    manifest = json.loads(data)
    if (manifest.get('schema') != 'ddonirang.third_party_notices.v1'
            or manifest.get('source_lock_sha256') != lock_sha256
            or manifest.get('license_review') != 'NOT_RUN'):
        raise ValueError('제삼자 고지 출처/검토 경계 불일치')
    payload = {'THIRD_PARTY_NOTICES.json': data,
               'THIRD_PARTY_NOTICES.md': (root / 'THIRD_PARTY_NOTICES.md').read_bytes()}
    if sha(payload['THIRD_PARTY_NOTICES.md']) != manifest.get('markdown_sha256'):
        raise ValueError('제삼자 고지 안내 해시 불일치')
    missing = 0
    for package in manifest['packages']:
        entries = package['files']
        missing += not entries
        expected = 'SOURCE_TEXT_COPIED' if entries else 'TEXT_MISSING_REQUIRES_REVIEW'
        if package['status'] != expected:
            raise ValueError('제삼자 고지 상태 불일치')
        for entry in entries:
            rel = safe_path(entry['path'])
            if not rel.startswith('THIRD_PARTY_LICENSES/') or rel in payload:
                raise ValueError('제삼자 고지 경로/중복 오류')
            local = (root / rel).resolve()
            if not local.is_relative_to(root):
                raise ValueError('제삼자 고지 경로 탈출')
            content = local.read_bytes()
            if not content or sha(content) != entry['sha256']:
                raise ValueError('제삼자 고지 원문 해시 불일치')
            payload[rel] = content
    if manifest['missing_text_count'] != missing:
        raise ValueError('제삼자 고지 미확인 개수 불일치')
    return payload, {'manifest_sha256': sha(data), 'missing_text_count': missing,
                     'license_review': 'NOT_RUN', 'scope': manifest['scope']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--version', choices=VERSIONS, default='1.0.0-alpha.2')
    for arg in ('base', 'base-sha256', 'source-commit', 'build-identity', 'native-dir', 'output'):
        parser.add_argument('--' + arg, required=True)
    parser.add_argument('--third-party-dir')
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    def git(*parts):
        return subprocess.check_output(['git', '--no-optional-locks', '-C', str(repo), *parts])
    commit = git('rev-parse', args.source_commit + '^{commit}').decode().strip()
    tree = git('rev-parse', commit + '^{tree}').decode().strip()
    parents = git('show', '-s', '--format=%P', commit).decode().strip().split()
    def source(rel):
        return git('show', commit + ':' + safe_path(rel))
    if source('tools/alpha2_completion_archive.py') != Path(__file__).read_bytes():
        raise ValueError('조립 도구 자체가 고정 commit과 다릅니다')
    for helper in ('alpha1_r20_archive.py', 'alpha1_r19_archive.py', 'alpha1_test_gate.py'):
        if source('tools/' + helper) != (repo / 'tools' / helper).read_bytes():
            raise ValueError('조립 보조 도구가 고정 commit과 다릅니다: ' + helper)
    baseline = read_baseline(Path(args.base).resolve(), args.base_sha256)
    retired_paths = [BASE + 'ui/' + name for name in ('index.html', 'play.html', 'app.js', 'playground.js')]
    retired_hashes = set()
    for rel in retired_paths:
        try:
            retired_hashes.add(sha(source(rel)))
        except subprocess.CalledProcessError:
            pass
    payload = {rel: data for rel, data in baseline.items()
               if rel not in METADATA and not rel.startswith(BASE + 'ui/') and not forbidden(rel)}
    # Bounded committed UI inventory; dependencies select the actual user surface.
    ui_paths = git('ls-tree', '-r', '--name-only', '-z', commit, BASE + 'ui/').decode().rstrip('\0').split('\0')
    retired_hashes.update(sha(source(rel)) for rel in ui_paths if is_retired_ui_path(rel))
    candidate = dict(payload)
    candidate.update({rel: source(rel) for rel in ui_paths if rel and not forbidden(rel)})
    required_samples = REQUIRED_SAMPLES + (ALPHA3_SAMPLES if args.version in ALPHA3_OR_LATER else ()) + (ALPHA4_SAMPLES if args.version in ALPHA4_OR_LATER else ())
    if args.version == '1.0.0-beta.1':
        required_samples += BETA1_SAMPLES
    candidate.update({rel: source(rel) for rel in required_samples})
    course_roots = [ALPHA2_COURSE] + ([ALPHA3_COURSE] if args.version in ALPHA3_OR_LATER else []) + ([ALPHA4_COURSE] if args.version in ALPHA4_OR_LATER else [])
    if args.version == '1.0.0-beta.1':
        course_roots.append(BETA1_COURSE)
    course_paths = git('ls-tree', '-r', '--name-only', '-z', commit, *course_roots).decode().rstrip('\0').split('\0')
    for rel in course_paths:
        if rel:
            candidate[rel] = source(rel)
            payload[rel] = candidate[rel]
    selected = dependencies(candidate)
    payload.update({rel: candidate[rel] for rel in selected})
    for rel in ui_paths:
        if rel.startswith(WASM):
            payload[rel] = source(rel)
    for rel in required_samples:
        payload[rel] = source(rel)
    payload[BASE + 'tools/ddn_exec_server.py'] = source(BASE + 'tools/ddn_exec_server.py')
    # Use the same provided launcher in isolated startup checks without stopping
    # a user's existing default-port server. Runtime/world input is unaffected.
    payload['START_SEAMGRIM.cmd'] = b'''@echo off\r
setlocal\r
cd /d "%~dp0"\r
python -c "import sys; assert sys.version_info >= (3,10)"\r
if errorlevel 1 (echo Python 3.10+ is required. & pause & exit /b 1)\r
set "DDONILANG_BIN=%~dp0bin\\ddonilang.exe"\r
if not defined SEAMGRIM_PORT set "SEAMGRIM_PORT=8787"\r
echo http://127.0.0.1:%SEAMGRIM_PORT%/ - Ctrl+C to stop\r
python "solutions\\seamgrim_ui_mvp\\tools\\ddn_exec_server.py" --host 127.0.0.1 --port %SEAMGRIM_PORT% --catalog-mode reps_only\r
exit /b %errorlevel%\r
'''
    identity = json.loads(Path(args.build_identity).read_text(encoding='utf-8'))
    if identity.get('source_commit') != commit or identity.get('source_tree') != tree:
        raise ValueError('빌드 출처 commit/tree 불일치')
    notices = {'status': 'NOT_SUPPLIED', 'license_review': 'NOT_RUN'}
    if args.third_party_dir:
        notice_payload, notices = read_notice_bundle(args.third_party_dir,
            sha(source('Cargo.lock')), identity.get('notices_manifest_sha256'))
        payload.update(notice_payload)
    manifest = json.loads(payload[WASM + 'ddonirang_tool.manifest.detjson'])
    if manifest['wasm_sha256'] != sha(payload[WASM + 'ddonirang_tool_bg.wasm']):
        raise ValueError('WASM manifest 바이트 불일치')
    if identity.get('wasm_sha256') != manifest['wasm_sha256']:
        raise ValueError('빌드 출처 WASM 불일치')
    if manifest['js_binding_sha256'] != sha(payload[WASM + 'ddonirang_tool.js'].replace(b'\r\n', b'\n')):
        raise ValueError('WASM JavaScript binding 바이트 불일치')
    if manifest['cargo_lock_sha256'] != sha(source('Cargo.lock').replace(b'\r\n', b'\n')):
        raise ValueError('WASM Cargo.lock 출처 불일치')
    for row in manifest['source_files']:
        data = source(row['path']).replace(b'\r\n', b'\n')
        if sha(data) != row['sha256']:
            raise ValueError('WASM 빌드 뒤 소스 변경: ' + row['path'])
    for name in NATIVES:
        data = (Path(args.native_dir) / name).read_bytes()
        if identity.get('native_sha256', {}).get(name) != sha(data):
            raise ValueError('Native 빌드 출처 해시 불일치: ' + name)
        payload['bin/' + name] = data
    if args.version == '1.0.0-beta.1':
        payload.update(native_workbench_payload(payload))
        payload['START_SEAMGRIM.cmd'] = b'@echo off\r\nsetlocal\r\ncd /d "%~dp0"\r\n"%~dp0bin\\ddonilang.exe" workbench\r\nif errorlevel 1 (pause & exit /b 1)\r\nexit /b %errorlevel%\r\n'
    payload['README.ko.md'] = '''# 또니랑 1.0.0-alpha.2

한국어로 원문을 쓰고, 실행하고, 고쳐 보는 알파 체험판입니다.
첫걸음 코스와 함께 시작하거나, 원문이 만든 여러 보기를 살펴보고 테트리스를 해 보세요.

## 처음 열기

1. ZIP을 새 폴더에 모두 풉니다. 압축 안에서 실행하지 마세요.
2. Python 3.10 이상이 설치되어 있어야 합니다. Python이 없으면 먼저 설치해야 합니다.
3. START_SEAMGRIM.cmd를 두 번 클릭합니다. PowerShell에서는 ./START_SEAMGRIM.cmd로 실행합니다.
4. 브라우저가 열리지 않으면 http://127.0.0.1:8787/ 로 들어갑니다.
5. 서버를 닫을 때는 서버 창에서 Ctrl+C를 누릅니다.

파일을 브라우저에서 직접 여는 대신 위 로컬 서버를 사용합니다. 예제·Worker·WASM을 같은 서버에서 읽습니다.

## 테트리스와 기록 재생

작업대의 논리 입력 패널을 열고 ‘실행을 닫고 새 테트리스 작업 열기’를 누릅니다.
다른 작품의 원문을 덮어쓰지 않고 새 작업이 열립니다. 위쪽 ▶ 하나로 시작하고 메인 보개에서 키를 누릅니다.
같은 단추로 일시정지·재개하고, 한 마디 단추로 한 번씩 진행할 수 있습니다.
화살표 왼쪽·오른쪽은 이동, 위는 회전, 아래는 내리기, Space는 떨어뜨리기입니다.
일시정지 후 기록을 봉인·저장하면 실제 계산한 마디를 담은 파일을 내려받습니다.
같은 원문·입력 계약을 선택한 뒤 기록 파일을 열면 같은 판을 재생합니다.
입력 영역에서 초점이 벗어나거나 실행 중 원문이 바뀌면 조용히 무입력으로 진행하지 않고 오류를 냅니다.

## 원문이 만든 여러 보기

메뉴의 포물선·진자·일차함수는 원문이 으뜸·겹·곁·돋을 선언합니다.
보기 단추로 겹 표시를 숨기거나 곁을 닫고 다시 열 수 있습니다. 이 조작은 세계 계산을 바꾸지 않습니다.
으뜸의 표본 점·추·고정점·공을 누르면 선택한 대상의 정보를 돋으로 봅니다. 다른 대상을 누르면 이전 돋을 닫고 새로 엽니다.
‘지나온 자리’는 화면이 기록한 이전 그림이며 원문 선언 보기와 구별됩니다.
일차함수는 시간에 따라 바뀌지 않으므로 한 번 실행합니다. 진자는 작은 각도 근사 모형입니다.

## 첫걸음과 CLI

위쪽 첫걸음 메뉴에서 정의 중심 1~10차시와 알파2 11~16차시를 순서대로 따라갈 수 있습니다.
11~16차시는 모양·매마디, 보기값 셈씨, 보개짜임, 물체 돋, 논리 입력, 기록·재생을 다룹니다.
논리 입력 활동은 안내의 ‘이 활동의 논리 입력 계약 선택’을 누른 뒤 위쪽 ▶로 실행합니다.
학생용·교사용 PDF와 원문은 solutions/seamgrim_ui_mvp/first_steps/ 및 first_steps_alpha2/에 있습니다.
bin/ddonirang.exe, bin/ddonilang.exe, bin/또니랑.exe는 같은 CLI의 세 이름입니다.
예: ./bin/ddonirang.exe 실행 <원문.ddn> --source-profile v1-core-v25
논리 입력 파일 재생은 ‘논리입력재생 <원문.ddn> <기록파일> <입력계약.json>’을 사용합니다.

## 알려진 범위

반응·알림씨 훅과 글자 입력은 이 판에서 지원하지 않습니다.
보개는 공통 마디를 사용합니다. 고정·상대 마디(RD-136)는 알파3 작업이며,
원문에서 사용하면 E_BOGAE_TIME_POLICY_UNSUPPORTED로 명시 거부합니다.
소수 표시에는 19.019999999 같은 결과가 나타날 수 있습니다. Python 설치도 아직 필요합니다.
독립검토·Owner 스모크·공개 승인 상태는 배포 기록과 함께 확인하세요.
이 ZIP 조립만으로 그 승인이 생기지는 않습니다.
'''.encode('utf-8')
    if args.version in ALPHA3_OR_LATER:
        payload['README.ko.md'] = '''# 또니랑 1.0.0-alpha.3

수와 모형을 직접 바꾸고, 계산·그림·오차를 함께 비교하는 알파 체험판입니다.
첫걸음 코스, 함수 탐구, 이중 진자, 작은 시장과 테트리스를 담았습니다.

## 처음 열기

1. ZIP을 새 폴더에 모두 풉니다. Python 3.10 이상이 필요합니다.
2. START_SEAMGRIM.cmd를 두 번 클릭합니다. PowerShell에서는 ./START_SEAMGRIM.cmd로 실행합니다.
3. 브라우저가 열리지 않으면 http://127.0.0.1:8787/ 로 들어갑니다.
4. 종료할 때는 서버 창에서 Ctrl+C를 누릅니다.

## 스모크 순서

첫걸음17차시에서 계산값과 표시를 비교하고, 함수·시장·이중진자에서 상황과
조절값을 바꿔 다시 실행해 보세요. 재생 중 곁보기를 닫고 다시 열거나 물체를
눌러 돋보기를 확인합니다. 20차시에서는 같은 마디의 사건 순서를 바꿔 보세요.
작품 자료 창의 실행 상태 저장·재열기는 원문과 함께 현재 상태를 복원합니다.

## 알려진 경계

잠정 결정은 Owner의 개별 승인이 아닙니다. 이 ZIP은 로컬 스모크 후보이며
공개 승인이 아닙니다. 글자 행동 입력·고급 텐서/계량은 지원하지 않습니다.
실행 상태 저장의 브라우저 트랜잭션 검증은 RD-669 내구 영수증이 아닙니다.
복원된 실행에서 완전한 입력 기록이 없으면 Tape V1 내보내기를 거부합니다.
독립검토·Owner 스모크 상태와 정확한 원본 커밋은 RELEASE_MANIFEST.json에 있습니다.
'''.encode('utf-8')
    if args.version in ALPHA4_OR_LATER:
        payload['README.ko.md'] = '''# 또니랑 1.0.0-alpha.4 잠정 체험 후보

초기조건 슬라이더·추 끌기, 화면 확대·이동, 수·좌표 조작 기록과 LRL 실험실을 담았습니다.
첫걸음 29~30차시 학생용·교사용 PDF가 first_steps_alpha4에 있습니다.

## 처음 열기

1. ZIP을 새 폴더에 모두 풉니다. Python 3.10 이상이 필요합니다.
2. START_SEAMGRIM.cmd를 두 번 클릭합니다. PowerShell에서는 ./START_SEAMGRIM.cmd로 실행합니다.
3. 브라우저에서 http://127.0.0.1:8787/ 로 들어갑니다. 종료는 서버 창의 Ctrl+C입니다. 배치 종료 확인이 나오면 Y를 누릅니다.

## 써 보기

이중 진자를 실행하고 추를 끌어 원문의 초기각을 바꾼 뒤 ▶로 다시 계산하세요.
확대·이동은 화면만 바꿉니다. LRL 실험실에서 원·타원·섭동 상황을 비교하고 공에서 끌어 발사 속도를 편집한 뒤 ▶로 다시 실행하세요.
점과 반지름 실습은 ▶로 실행하고 입력 기록을 펼쳐 수·좌표를 조작합니다.
일시정지·한 마디 뒤 기록을 저장하고 같은 원문·계약으로 불러옵니다.
CLI는 조작입력재생 <원문.ddn> <기록파일> <입력계약.json>을 사용합니다.

## 또니 직접 스모크 체크리스트

각 항목을 직접 사용한 뒤 통과/실패/미실행으로 기록하세요. 자동 검증이 대신하지 않습니다.

- [ ] 새 폴더에서 ./START_SEAMGRIM.cmd로 열기. 종료는 Ctrl+C와 배치 종료 확인 Y.
- [ ] 이중 진자 ▶ 실행 → 추 끌기 → 원문 초기각 변경 확인 → ▶ 재실행. 끌기만으로 현재 세계가 바뀌지 않는지 확인.
- [ ] 확대·이동 → 겹 구별 → 재생 중 곁 닫기·다시 열기 → 최신 측정값 갱신 확인.
- [ ] LRL 원·타원·섭동 상황을 고르고 명시 재실행. 공에서 끌어 발사 속도를 편집한 뒤 ▶로 새 궤도 확인.
- [ ] 점과 반지름 실습에서 재생 중 수·좌표 조작 → 일시정지 → 기록 저장 → 같은 원문·계약으로 재생.
- [ ] 첫걸음 29~30차시 학생용 PDF를 열고 예상·수정·이유 설명 활동 수행.
- [ ] 테트리스를 공식 모음에서 열고 ▶ 하나로 키 입력 시작. 첫걸음 기존 예제 하나도 실행.

문제가 있으면 항목, 창 크기, 원문 변경 여부, 오류 문구와 재현 순서를 함께 남겨 주세요.

## 경계

수값·좌표값 조회와 정수 저장·대상·단위·범위 결속은 승인되었습니다. 단계 철자와 기록 이름·파일 머리표·V1 통합은 미확정이며 베타1 동결 때 정합니다.
이전 기록은 새 형식으로 몰래 이관하지 않습니다. 글자 입력·고급 텐서는 미지원입니다.
초기값 편집은 실행 입력 기록이 아닙니다. 실험실의 모형 가정과 수치 오차를 구별하세요.
독립검토·또니 스모크·공개 승인은 별도이며 조립만으로 PASS가 생기지 않습니다.
'''.encode('utf-8')
    if args.version == '1.0.0-beta.1':
        payload['README.ko.md'] = '''# 또니랑 1.0.0-beta.1 개발 후보

여러 파일 작품을 편집하고 저장한 뒤 다시 여는 흐름을 확인하기 위한 내부 후보입니다.
이 패키지의 생성은 베타1 완료·표면 동결·독립검토·공개 승인을 뜻하지 않습니다.
알파2~4 예제와 첫걸음 코스도 포함합니다.

## 일반 실행의 정체와 결정론

저장 없는 일반 run은 등록 기록 파일을 만들지 않고 매 실행 임시 정체를 새로 발급합니다.
이름씨·원자·임자를 쓰면 같은 원문·입력이어도 실행 간 state_hash가 다를 수 있습니다.
결정론은 초기 정체·입력·난수 상태 등 필요한 문맥이 보존된 같은 기록을 재생할 때 보장합니다.
입력 Tape 하나만으로 이 문맥 전체가 보존되는 것은 아닙니다. 저장한 작품은 보존된 정체를 그대로 사용합니다.
임시 정체를 저장·내보내기로 이어 실행의 권한으로 옮길 수는 없습니다.

## 처음 열기

1. ZIP을 새 폴더에 모두 풉니다. Python이나 Rust의 별도 설치가 필요하지 않습니다.
2. START_SEAMGRIM.cmd를 두 번 클릭합니다. PowerShell에서는 ./START_SEAMGRIM.cmd로 실행합니다.
3. 브라우저가 자동으로 열립니다. 서버는 기존과 같은 http://127.0.0.1:8787/ 을 사용하므로 이 주소의 브라우저 저장소가 유지됩니다. 포트가 사용 중이면 명시 오류로 끝나며 다른 서버를 종료하지 않습니다. 종료하려면 서버 창에서 Enter를 누르세요. 브라우저 창만 닫으면 서버는 계속 실행됩니다. Ctrl+C를 쓴 경우 배치 종료 확인이 나오면 Y를 누릅니다.
4. 자동 열기가 어려우면 bin/ddonilang.exe workbench --no-open으로 실행하고, 출력의 startup_url 뒤에 ?token=automation_token 값을 붙여 여세요. 이 값은 이번 실행의 접근 열쇠이므로 공유하지 마세요.
5. 격리 검사에는 --port 0으로 빈 포트를 사용할 수 있습니다. 주소(포트)가 바뀌면 브라우저 저장소도 별개입니다. 8790은 사용하지 않습니다.

## 직접 스모크

- [ ] 예제 열기 → 값 변경 → 실행 → 오류 만들기 → 진단 읽기 → 수정 → 정상 실행.
- [ ] 명시 뿌리가 있는 여러 파일 작품을 가져와 보조 모듈 편집 → 원문 저장 → 실행.
- [ ] 일시정지 → 실행 상태 저장 → 브라우저 재열기 → 원문 재열기 → 실행 상태 재열기 → 한 마디.
- [ ] 키 입력 작품은 원문 재열기 뒤 같은 입력 계약을 명시 선택하고 실행 상태를 다시 엽니다. 저장본 안의 계약을 자동으로 기대 계약으로 삼지 않습니다.
- [ ] 실행 상태를 파일로 내보내고 안내된 CLI 명령으로 이어 실행.
- [ ] 보개 열기·닫기·확대·이동과 첫걸음·테트리스 회귀 확인.

## 남은 경계

Native 저장·종료·재열기·이어 실행을 직접 확인하려면
solutions/seamgrim_ui_mvp/samples/beta1_logical_counter/README.ko.md를 따르세요.
세 파일 작품이 실제로 3에서 6으로 진행합니다. 이 예제는 테트리스 저장 지원을 대신하지 않습니다.
테트리스의 3마디 저장→새 프로세스 4마디 이어 실행 안내는
solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/README.ko.md에 있습니다.

브라우저 저장 트랜잭션과 다시 읽기 검사는 RD-669의 내구 영수증·전체 장애 복구가 아닙니다.
Native SQLite는 공통 Rust의 제한된 일반·논리 실행 저장·복원·이어 실행·재저장 경로에 연결했습니다.
CLI 세 이름의 실제 새 프로세스 검증은 있으나 모든 작품과 브라우저 저장에 대한 완료를 뜻하지 않습니다.
논리 실행의 고정 씨앗은 승인된 mode03으로 보존하고, 일반 실행의 mode01 XOR-마디 규칙은 유지합니다.
일반 mode01은 실제 처리 입력과 원문·난수·이어 실행 상태를 결속하며 현재 기본 입력 구성의 반복 Native 저장을 지원합니다. 다른 입력 구성은 명시 거부합니다.
글·참거짓은 정식 자원 값으로 저장합니다. 테트리스는 같은 원문으로 3마디 저장·종료 후 4마디 이어 실행한 결과가 7마디 연속 실행과 일치하는지 release CLI 세 이름과 새 WASM에서 확인했습니다.
옛 resource_json 채널에 의존하는 저장본은 명시 거부합니다. 자동 변환하거나 두 채널을 함께 읽지 않습니다.
지원되는 논리 입력 작품은 기존 checkpoint와 원문·현재 상태 자료를 같은 브라우저 거래로 저장합니다.
브라우저 종료 후 원문과 입력 계약을 명시 선택하여 상태를 다시 열고 다음 마디를 진행할 수 있습니다.
이는 Native 내구 영수증이 아니며 일반 실행 상태로 대신 저장하지 않습니다. 처리되지 않은 입력이나 원문·계약 불일치는 저장/복원을 거부합니다.
저장 머리를 바꾸기 전 실패는 기존 정상 저장본과 머리를 보존하며 성공을 게시하지 않습니다. 머리를 바꾼 뒤 확인 실패는 RecoveryRequired로 처리하며 성공을 게시하지 않고 임의로 되돌리지 않습니다. 정전 복구 증명과 RD-669 전체 영수증은 베타2 이후 범위입니다.
Native 저장 도중 프로세스가 중단된 자료의 자동 복구는 아직 완료되지 않았습니다. 새 SQL 머리가 보이는 것만으로 저장 성공이나 이어 실행 권한이 생기지 않습니다.
재열기에서 사용할 복구 기록·토큰의 영속 공급과 동일 요청 재시도 경로는 미완입니다. 이번 공통 저장 실행기 검증을 전체 복구 보장으로 읽지 마세요. 검사 전용 빌드의 네 단계 강제 종료 근거와 배포 CLI의 정상 저장·이어 실행 근거는 구분합니다.
완전한 입력 이력이 없는 복원은 Tape 내보내기를 명시 거부합니다.
범용 참조의 전체 생성·읽기·저장 수명은 아직 지원 완료가 아닙니다.
위치형 값꾸러미(Tuple)는 셈씨 내부의 값으로 생성하고 순서를 구분하며, 정적 리터럴 위치·반열림 범위 선택과 같은 위치씨의 교체를 지원합니다.
원문에서 위치별 씨가 검증되는 한정 경로는 Tuple의 세계 자원·저장·복원과 정적 선택을 지원합니다.
samples/beta1_typed_tuple/README.ko.md에서 같은 값의 바른수·셈수 구분과 저장·이어 실행을 직접 확인할 수 있습니다.
타입 공급이 없거나 결속이 모호한 경로는 명시 거부하며 차림이나 짝맞춤으로 몰래 저장하지 않습니다. 전체 VC13 행 폐쇄는 별도 검토입니다.
수 리터럴의 기본 위치씨는 정수 `바른수`, 소수 `셈수`입니다. 원문으로 검증한 위치별 씨를 Tuple의 생성·선택·교체·저장·복원에 공급합니다. 명시된 나눔수 문맥의 닫힌 리터럴 산술식은 원문 자릿수에서 정확한 분수로 받습니다.
VC13 B01 전체가 끝난 것은 아닙니다. 정수 리터럴의 셈수 자리 사용과 혼합 리터럴 정책은 후속 SSOT 결정 전까지 기존 실행을 유지합니다. 미해소 식의 위치씨를 값 모양으로 추측하거나 저장 검증을 풀어 지원한 것처럼 처리하지 않습니다.
고름씨는 원문에 선언한 소유자와 갈래를 구별해 값을 만들고 분기합니다. 갈래의 내용을 필드나 숫자 위치로 해체하지 않습니다.
복수 payload의 선언된 이름 중 필요한 것만 고를 수 있고, 이름을 바꾸거나 이미 결속된 이름을 잡으면 거부합니다. 적용 맺음은 명시한 씨와 값·이유 이름을 사용하며 있없을 갈래값으로 바꾸지 않습니다.
고름씨 pattern의 exact-name 정렬은 PD-WO19-001 내부 canon 후보입니다. 정본 ParamPin 물리 comparator는 미완·미동결이며 출시 전 clean-break 대상입니다. 일반 payload 기본값과 사용자 generic 고름씨 선언은 이 후보의 검증 범위에 포함하지 않습니다.
고름씨 값의 새 세계 자원·저장 carrier는 추가하지 않았습니다. 현재 지원하는 지역 값 실행과 저장 지원을 구분합니다.
명시한 모듈의 고름씨 선언과 지역 분기 계산을 저장 원문에 포함할 수 있습니다. 지역 계산 뒤 남은 수 자원의 저장·이어 실행과 고름씨 값 자체의 저장은 서로 다른 지원 범위입니다.
글자 행동 입력은 미지원이며 잠정 입력 기록 이름·머리표·단계 철자는 동결되지 않았습니다.
정확한 원본과 검토·Human·공개 상태는 RELEASE_MANIFEST.json을 확인하세요.
'''.encode('utf-8')
    if args.version == '1.0.0-beta.1':
        payload['README.ko.md'] = _beta1_progress_notes(payload['README.ko.md'].decode('utf-8'), identity).encode('utf-8')
        if identity.get('beta1_freeze_contract_candidate') == 'r65':
            payload['BETA1_SMOKE.md'] = source(BASE + 'BETA1_SMOKE.md')
            payload['BETA1_KNOWN_TEST_FAILURES.md'] = source(BASE + 'BETA1_KNOWN_TEST_FAILURES.md')
            payload['README.ko.md'] += '\n직접 체험 순서와 기대 결과는 BETA1_SMOKE.md를 보세요.\n'.encode('utf-8')
        if BETA1_COURSE + '학생용_31-34차시.txt' in payload:
            payload['README.ko.md'] += ('\n## 첫걸음 보충 31~34차시\n\n'
                '맨 바깥 Tuple, 정확한 나눔수, 다른 수 종류의 비교 오류, 테트리스 저장·이어 실행을 배웁니다.\n'
                '[학생용 설명](solutions/seamgrim_ui_mvp/first_steps_beta1/학생용_31-34차시.txt)과 '
                '[기대 결과](solutions/seamgrim_ui_mvp/first_steps_beta1/검증목록.json)를 함께 보세요.\n'
                '34차시는 기존 samples/beta1_logical_tetris의 저장 안내와 원문을 그대로 사용합니다.\n').encode('utf-8')
    release = {'version': args.version, 'source': {'commit': commit, 'tree': tree, 'parents': parents},
        'baseline': {'path': str(Path(args.base).resolve()), 'sha256': args.base_sha256},
        'build_identity': identity, 'baseline_preserved': [rel for rel in sorted(payload)
            if rel.startswith(FIRST_STEPS)],
        'third_party_notices': notices,
        'independent_review': 'PENDING', 'owner_human_smoke': 'PENDING', 'public_approval': 'PENDING'}
    payload['RELEASE_MANIFEST.json'] = encode(release)
    paths = sorted(set(payload) | METADATA)
    payload['PACKAGE_ALLOWLIST.json'] = encode({'paths': paths, 'entry_count': len(paths)})
    payload['SHA256SUMS.txt'] = ''.join(sha(payload[rel]) + '  ' + rel + '\n'
        for rel in sorted(payload)).encode('utf-8')
    result = write_archive(Path(args.output).resolve(), payload, retired_hashes, args.version)
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()

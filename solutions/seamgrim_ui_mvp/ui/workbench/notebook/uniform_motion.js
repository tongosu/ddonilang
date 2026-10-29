const LENGTH = [1, 0, 0, 0, 0, 0, 0, 0, 0];
const TIME = [0, 1, 0, 0, 0, 0, 0, 0, 0];
const SPEED = [1, -1, 0, 0, 0, 0, 0, 0, 0];

function coefficient(value) {
  const { numerator: n, denominator: d } = value ?? {};
  if (typeof n !== 'string' || typeof d !== 'string' || n.length > 310 || d.length > 309
      || !/^-?(0|[1-9][0-9]*)$/u.test(n) || !/^[1-9][0-9]*$/u.test(d)) {
    throw new Error('공통 결과의 정확 계수가 올바르지 않습니다.');
  }
  return { text: d === '1' ? n : `${n}/${d}`, coordinate: Number(n) / Number(d), n, d };
}
function quantity(value, expected) {
  if (!Array.isArray(value?.dimension) || value.dimension.length !== expected.length
      || !value.dimension.every((n, i) => n === expected[i])) {
    throw new Error('공통 결과의 차원이 표시 대상과 다릅니다.');
  }
  return coefficient(value.coefficient);
}

// Exact quantities come from Rust. The sole Number conversion below feeds
// display coordinates, never equation solving, equality or saved values.
export function projectUniformMotion(report, sourceProjectText) {
  if (report?.schema !== 'ddn.math-notebook.observation.v1' || report.model !== 'uniform_motion'
      || report.proof !== false || report.source_profile !== 'v1-core-v25'
      || report.time_domain !== 't>=0' || report.normalization?.unit_dimension_schema !== 'ddn.unit_dimension_wire.v2'
      || ['length_basis', 'time_basis', 'velocity_basis'].some(key => report.normalization[key] !== 1)
      || !Array.isArray(report.rows) || !report.rows.length || report.rows.length > 256) {
    throw new Error('지원하는 공통 등속운동 관찰 결과가 아닙니다.');
  }
  const initial = quantity(report.inputs?.initial_position, LENGTH);
  const velocity = quantity(report.inputs?.velocity, SPEED);
  const target = quantity(report.inputs?.target, LENGTH);
  const samples = report.rows.map(row => ({ time: quantity(row.time, TIME), position: quantity(row.position, LENGTH) }));
  let solutionMessage;
  switch (report.solution?.kind) {
    case 'solution': {
      const time = quantity(report.solution.time, TIME);
      const left = quantity(report.solution.substitution?.left, LENGTH);
      const right = quantity(report.solution.substitution?.right, LENGTH);
      if (typeof report.solution.substitution.equal !== 'boolean') throw new Error('검산 결과 공급이 없습니다.');
      solutionMessage = `도달 시간 ${time.text} s · 검산 ${left.text} m ${report.solution.substitution.equal ? '=' : '≠'} ${right.text} m`;
      break;
    }
    case 'no_solution': solutionMessage = '정한 시간 범위에서 목표에 도달하는 해가 없습니다.'; break;
    case 'non_unique': solutionMessage = '목표에 도달하는 시각을 하나로 정할 수 없습니다.'; break;
    case 'outside_domain': {
      const time = quantity(report.solution.algebraic_time, TIME);
      const left = quantity(report.solution.substitution?.left, LENGTH);
      const right = quantity(report.solution.substitution?.right, LENGTH);
      if (typeof report.solution.substitution.equal !== 'boolean') throw new Error('검산 결과 공급이 없습니다.');
      solutionMessage = `계산된 시각 ${time.text} s는 정한 시간 범위(t>=0) 밖입니다. · 원식 검산 ${left.text} m ${report.solution.substitution.equal ? '=' : '≠'} ${right.text} m`;
      break;
    }
    default: throw new Error('지원하지 않는 풀이 결과입니다.');
  }
  const canPlot = samples.every(row => Number.isFinite(row.time.coordinate) && Number.isFinite(row.position.coordinate));
  const artifact = sourceProjectText ? JSON.parse(sourceProjectText) : null;
  return {
    columns: [{ label: '시각', unit: 's' }, { label: '위치', unit: 'm' }],
    rows: samples.map(row => [row.time.text, row.position.text]),
    graph: canPlot ? { schema: 'seamgrim.graph.v0', series: [{ id: '위치',
      points: samples.map(row => ({ x: row.time.coordinate, y: row.position.coordinate })) }] } : null,
    axisLabels: { x: '시각 (s)', y: '위치 (m)' },
    positions: canPlot ? samples.map(row => ({ label: `${row.time.text} s`,
      coordinate: row.position.coordinate, exact: `${row.position.text} m` })) : null,
    positionAxis: '위치 (m)', solutionMessage,
    inputSummary: `처음 위치 ${initial.text} m · 속도 ${velocity.text} m/s · 목표 ${target.text} m`,
    comparisonKey: { model: report.model, profile: report.source_profile,
      entryPath: artifact?.source_snapshot?.entry_path ?? null,
      rootSource: artifact?.source_snapshot?.files?.find(file => file.path === artifact.source_snapshot.root_path)?.text ?? null,
      domain: report.time_domain, initial: [initial.n, initial.d], target: [target.n, target.d],
      times: samples.map(row => [row.time.n, row.time.d]) },
    comparedInput: [velocity.n, velocity.d],
  };
}

export function createUniformMotionProvider(runtimeBridge) {
  return {
    async inspect(raw) {
      const artifact = JSON.parse(raw);
      const snapshot = artifact?.source_snapshot;
      if (artifact?.schema !== 'ddn.authored_project_artifact.v1' || !snapshot
          || !snapshot.root_path || snapshot.files?.length !== 2) {
        throw new Error('첫 모형은 명시한 뿌리와 계산 원문으로 된 작품이 필요합니다.');
      }
      const file = snapshot.files.find(item => item.path === snapshot.entry_path);
      if (typeof file?.text !== 'string') throw new Error('명시한 시작 원문을 확인할 수 없습니다.');
      const project = JSON.parse(await runtimeBridge.authoredProjectOperation({ operation: 'source_project', text: raw }));
      const graph = await runtimeBridge.projectGraph(project);
      if (graph.root_path !== snapshot.root_path || graph.entry_path !== snapshot.entry_path
          || typeof graph.start_definition !== 'string' || !graph.start_definition || Object.keys(graph.module_paths ?? {}).length !== 1
          || Object.keys(graph.gaji_aliases ?? {}).length || graph.diagnostics?.length) {
        throw new Error('첫 모형은 의존 가지 없이 명시한 시작 정의를 실행하는 작품이어야 합니다.');
      }
      const report = await runtimeBridge.observeMathNotebook({ sourceProjectText: raw });
      const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(raw));
      const expected = Array.from(new Uint8Array(bytes), byte => byte.toString(16).padStart(2, '0')).join('');
      if (report.authored_source?.artifact_sha256 !== expected
          || report.authored_source?.content_sha256 !== artifact.content_sha256
          || report.authored_source?.project_identity !== artifact.authored_identity.logical_dependency_id
          || report.authored_source?.artifact_schema !== artifact.schema
          || report.authored_source?.source_profile !== artifact.source_profile) {
        throw new Error('계산 결과와 실제 원문이 일치하지 않습니다.');
      }
      return {...report, notebookContext: uniformMotionSourceContext(raw)};
    },
    project: projectUniformMotion,
  };
}

// Source references only. Actual quantities/model admission belong to Rust.
export function uniformMotionSourceContext(raw) {
  const snapshot = JSON.parse(raw).source_snapshot;
  const whole = path => {
    const text = snapshot.files.find(file => file.path === path)?.text;
    if (typeof text !== 'string') throw new Error('재현할 실제 원문이 없습니다.');
    return {path, startByte: 0, endByte: new TextEncoder().encode(text).length, text};
  };
  if (snapshot.files.length !== 2 || snapshot.files.some(file => file.path === 'ddn.lock'))
    throw new Error('첫 모형은 외부 의존 없는 두 원문 작품만 저장합니다.');
  return {inputs:[whole(snapshot.entry_path)], units:[whole(snapshot.entry_path)],
    assumptions:[whole(snapshot.entry_path)], dependencies:[whole(snapshot.root_path)]};
}

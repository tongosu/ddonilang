// Read-only consumer for the current checked source revision. Menu policy stays with its owner.
export function createProjectLanguageSupport({runtimeBridge, readSelection}) {
  let selection = null;
  let generation = 0;
  let disposed = false;
  let state = {status: 'unknown', reason: 'source_revision_required'};

  function current() {
    if (disposed) return null;
    const value = readSelection();
    if (!value || typeof value.sourceArtifactText !== 'string' || !value.sourceArtifactText
        || value.sourceArtifactText.length > 8_000_000
        || !/^[0-9a-f]{64}$/u.test(value.projectHex ?? '')
        || !/^(?:[0-9a-f]{2})+$/u.test(value.revisionHex ?? '')) return null;
    try {
      // This DTO comparison only rejects a dirty editor. Rust remains the admission owner.
      const envelope = JSON.parse(value.sourceArtifactText);
      const payload = JSON.parse(envelope.project_payload_utf8);
      const project = JSON.parse(payload.authored_project_json_utf8);
      const saved = project.source_snapshot;
      const edited = value.sourceSnapshot;
      if (project.source_profile !== value.sourceProfileIdentity
          || !saved || !edited || saved.entry_path !== edited.entry_path
          || (saved.root_path ?? null) !== (edited.root_path ?? null)
          || !Array.isArray(saved.files) || !Array.isArray(edited.files)
          || saved.files.length !== edited.files.length
          || !saved.files.every((file, index) => file.path === edited.files[index].path
            && file.text === edited.files[index].text)) return null;
    } catch {
      return null;
    }
    return {sourceArtifactText: value.sourceArtifactText,
      projectHex: value.projectHex, revisionHex: value.revisionHex,
      sourceProfileIdentity: value.sourceProfileIdentity};
  }
  function same(a, b) {
    return a === b || Boolean(a && b && a.projectHex === b.projectHex
      && a.revisionHex === b.revisionHex && a.sourceArtifactText === b.sourceArtifactText);
  }
  function synchronize() {
    const next = current();
    if (!same(next, selection)) {
      selection = next;
      generation += 1;
      state = {status: 'unknown', reason: 'source_revision_required'};
    }
    return selection;
  }
  function getState() {
    synchronize();
    return structuredClone(state);
  }
  async function refresh() {
    const captured = synchronize();
    const request = ++generation;
    if (!captured) return getState();
    state = {status: 'loading'};
    try {
      const raw = await runtimeBridge.authoredProjectOperation({
        operation: 'source_language_support',
        text: JSON.stringify({source_artifact_utf8: captured.sourceArtifactText,
          project_hex: captured.projectHex, revision_hex: captured.revisionHex}),
      });
      const result = JSON.parse(raw);
      const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(captured.sourceArtifactText));
      const hash = Array.from(new Uint8Array(digest), value => value.toString(16).padStart(2, '0')).join('');
      if (result.project_hex !== captured.projectHex || result.revision_hex !== captured.revisionHex
          || result.source_profile !== captured.sourceProfileIdentity
          || result.source_artifact_sha256 !== hash || result.complete !== false
          || result.total_language_count !== null || !Array.isArray(result.verified_languages)
          || result.verified_languages.length === 0
          || result.verified_languages.some(name => typeof name !== 'string' || !name)
          || new Set(result.verified_languages).size !== result.verified_languages.length
          || result.verified_language_count !== result.verified_languages.length
          || !Array.isArray(result.evidence) || result.evidence.length === 0
          || result.evidence.some(row => !result.verified_languages.includes(row.canonical_name)
            || !['source_syntax', 'module_public_callables'].includes(row.scope))
          || result.verified_languages.some(name => !result.evidence.some(row => row.canonical_name === name))) {
        throw new Error('E_LANGUAGE_SUPPORT_RESPONSE_CONTEXT');
      }
      if (request === generation && same(synchronize(), captured)) {
        state = {...structuredClone(result), status: 'verified'};
      }
    } catch (error) {
      if (request === generation && same(synchronize(), captured)) {
        state = {status: 'unavailable', error: String(error?.message ?? error)};
      }
    }
    return getState();
  }
  function dispose() {
    disposed = true;
    selection = null;
    generation += 1;
    state = {status: 'unknown', reason: 'disposed'};
  }
  return {getState, refresh, dispose};
}

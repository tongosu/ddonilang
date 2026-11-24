import {createProjectLanguageSupport} from './project_languages.js';

// Connect the opened editor to existing read-only source storage. A locator selects
// a read; only the existing reader and Rust query can validate the saved revision.
export function createCurrentProjectLanguageSupport({runtimeBridge, readContext, storage}) {
  let generation = 0;
  let key = null;
  let selected = null;
  let disposed = false;
  let state = {status: 'unknown', reason: 'source_revision_required'};

  function capture() {
    if (disposed) return null;
    try {
      const raw = readContext();
      if (typeof raw?.artifactText !== 'string' || !raw.artifactText
          || typeof raw.profile !== 'string' || !Array.isArray(raw.sourceSnapshot?.files)) return null;
      const artifact = JSON.parse(raw.artifactText);
      const projectHex = artifact.authored_identity?.logical_dependency_id;
      if (!/^[0-9a-f]{64}$/u.test(projectHex ?? '') || artifact.source_profile !== raw.profile) return null;
      const context = {viewProjectId: raw.projectId, artifactText: raw.artifactText,
        profile: raw.profile, sourceSnapshot: raw.sourceSnapshot, locator: raw.locator ?? ''};
      return {...structuredClone(context), projectHex, key: JSON.stringify(context)};
    } catch {
      return null;
    }
  }
  const provider = createProjectLanguageSupport({runtimeBridge, readSelection: () => {
    const current = capture();
    return current?.key === key ? selected : null;
  }});
  function synchronize() {
    if (disposed) return null;
    const current = capture();
    const nextKey = current?.key ?? null;
    if (nextKey !== key) {
      key = nextKey;
      generation += 1;
      selected = null;
      state = {status: 'unknown', reason: 'source_revision_required'};
      provider.getState();
    }
    return current;
  }
  function getState() {
    synchronize();
    if (selected) state = provider.getState();
    return structuredClone(state);
  }
  function isCurrent(captured, request) {
    const current = synchronize();
    return !disposed && request === generation && current?.key === captured.key;
  }
  async function refresh() {
    const captured = synchronize();
    const request = ++generation;
    selected = null;
    provider.getState();
    state = {status: 'unknown', reason: disposed ? 'disposed' : 'source_revision_required'};
    if (!captured) return getState();
    let locator;
    try {locator = JSON.parse(captured.locator);} catch {return getState();}
    if (locator?.native_source_project_id !== captured.projectHex
        && locator?.source_project_id !== captured.projectHex) return getState();
    const unchanged = () => {
      if (!isCurrent(captured, request)) throw new Error('E_LANGUAGE_SUPPORT_CONTEXT_CHANGED');
    };
    state = {status: 'loading'};
    try {
      let received = {status: 'supply_missing'};
      const opened = await storage.reopen(captured.projectHex, unchanged, structuredClone(locator), {
        receiveSourceArtifact: value => {unchanged(); received = value;},
      });
      unchanged();
      if (opened !== captured.artifactText) throw new Error('E_LANGUAGE_SUPPORT_SAVED_CONTEXT');
      if (received?.status === 'supply_missing') {
        state = {status: 'unknown', reason: 'source_revision_required'};
        return getState();
      }
      if (received?.status !== 'accepted' || typeof received.sourceArtifactText !== 'string') {
        throw new Error(received?.error ?? 'E_LANGUAGE_SUPPORT_SAVED_SOURCE');
      }
      const source = JSON.parse(received.sourceArtifactText);
      if (source.schema !== 'ddn.source_only_project_artifact.v1') {
        state = {status: 'unknown', reason: 'source_carrier_not_supported'};
        return getState();
      }
      const payload = JSON.parse(source.project_payload_utf8);
      if (source.project_hex !== captured.projectHex || source.profile !== captured.profile
          || payload.authored_project_json_utf8 !== opened) {
        throw new Error('E_LANGUAGE_SUPPORT_SAVED_CONTEXT');
      }
      selected = {sourceArtifactText: received.sourceArtifactText, projectHex: captured.projectHex,
        revisionHex: source.revision_hex, sourceProfileIdentity: captured.profile,
        sourceSnapshot: captured.sourceSnapshot};
      const result = await provider.refresh();
      if (isCurrent(captured, request)) state = result;
    } catch (error) {
      if (isCurrent(captured, request)) {
        selected = null;
        state = {status: 'unavailable', error: String(error?.message ?? error)};
      }
    }
    return getState();
  }
  function dispose() {
    disposed = true;
    generation += 1;
    key = null;
    selected = null;
    provider.dispose();
    state = {status: 'unknown', reason: 'disposed'};
  }
  return {getState, refresh, dispose};
}

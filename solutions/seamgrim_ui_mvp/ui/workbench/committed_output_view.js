// Only committed, identity-bound output may occupy the existing primary view.
// This selects an already produced channel; it does not infer simulation data.
export function selectCommittedOutputView(payloads, frame, sourceHash) {
  if (!frame) return null;
  const bound = (payload) => payload?.status === 'available'
    && payload.schema === 'seamgrim.semantic_view_payload.v1_candidate'
    && String(payload.identity?.state_hash ?? '') === String(frame.state_hash ?? '')
    && String(payload.identity?.source_hash ?? '') === String(frame.source_hash ?? sourceHash ?? '')
    && Number(payload.identity?.tick_id) === Number(frame.madi)
    && Boolean(frame.state_hash);
  const space = payloads?.space2d;
  const console = payloads?.console;
  const lines = bound(console) && console.family === 'console'
    && Array.isArray(console.content?.ordered_entries)
    ? console.content.ordered_entries.map(row => String(row.text ?? '')) : [];
  if (bound(space) && space.family === 'space2d'
      && space.provenance?.classification === 'AUTHORITATIVE_STRUCTURED'
      && space.content?.space2d?.schema === 'seamgrim.space2d.v0'
      && Array.isArray(space.content.space2d.drawlist)
      && space.content.space2d.drawlist.length > 0) {
    return {family: 'space2d', space2d: space.content.space2d, lines,
      payload_hash: space.candidate_payload_hash};
  }
  if (bound(console) && console.family === 'console'
      && Array.isArray(console.content?.ordered_entries)
      && console.content.ordered_entries.length > 0) {
    return {family: 'console', lines: console.content.ordered_entries.map(row => String(row.text ?? '')),
      payload_hash: console.candidate_payload_hash};
  }
  return null;
}

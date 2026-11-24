// Presentation of existing Rust tables only. No joins, type or identity issuance.
export function committedObservationRows(frame) {
  const projection=frame?.wasm_state?.public_observation_table_projection;
  if (projection?.schema!=='ddn.public_observation_table_projection.v2'
      || projection.presentation_state!=='ready' || !Array.isArray(projection.tables)) return [];
  return projection.tables.flatMap(table=>{
    if (!Array.isArray(table.columns)||!Array.isArray(table.rows)) return [];
    return table.rows.filter(row=>row.state_hash===frame.state_hash
      && row.committed_madi===frame.madi && Array.isArray(row.cells)
      && row.cells.length===table.columns.length).map(row=>({
        columns:table.columns,cells:row.cells,
        source_trace_hash:projection.source_trace_hash,projection_hash:projection.projection_hash,
        run:projection.run,committed_madi:row.committed_madi,state_hash:row.state_hash,
      }));
  });
}

// Raw-byte transport only. Edition is explicitly selected by the UI action;
// parser, type, canonicalization and execution admission remain common Rust.
export async function readDdnSourceFile(file) {
  const name=String(file?.name??'');
  if(!name || /[\\/:]/u.test(name) || !/\.(ddn|또니)$/iu.test(name))
    throw new Error('E_SOURCE_FILE_NAME: 단일 .ddn 또는 .또니 원문 파일을 고르세요.');
  if(file.size>2_000_000)throw new Error('E_SOURCE_SNAPSHOT_LIMIT: 원문이 2,000,000바이트를 넘습니다.');
  const bytes=new Uint8Array(await file.arrayBuffer());
  if(bytes.length>2_000_000)throw new Error('E_SOURCE_SNAPSHOT_LIMIT: 원문이 2,000,000바이트를 넘습니다.');
  if(bytes[0]===0xef && bytes[1]===0xbb && bytes[2]===0xbf)
    throw new Error('E_SOURCE_UTF8_BOM_FORBIDDEN: 원문은 BOM 없는 UTF-8이어야 합니다.');
  let source;
  try{source=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);}
  catch{throw new Error('E_SOURCE_UTF8_INVALID: 원문은 올바른 UTF-8이어야 합니다.');}
  return {name,source};
}

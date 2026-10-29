// Verification only. Never used by the product to admit/encode world values.
// RD-674/682/683; A3-P6 provisional product physical profile.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
const pack=fs.readFileSync(new URL('../pack/alpha3_basic_tensor_physical_r1/vectors.json',import.meta.url));
const vectors=JSON.parse(pack);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const u32=n=>{const b=Buffer.alloc(4);b.writeUInt32BE(n);return b;};
const u64=n=>{const b=Buffer.alloc(8);b.writeBigUInt64BE(BigInt(n));return b;};
const field=(tag,bytes)=>Buffer.concat([Buffer.from([tag]),u32(bytes.length),bytes]);
function expectedCarrier(vector){
  assert.equal(vector.integers.length,vector.shape.reduce((a,b)=>a*b,1));
  const type=Buffer.concat([Buffer.from('DDN-BASIC-TENSOR-ELEMENT-TYPE-IDENTITY-V1\0'),Buffer.from([0,1,1,1,0,0,0,1,1])]);
  assert.equal(type.length,51);
  assert.equal(sha(type),'f334b6f3e3530ac476fbf00af39d40e1a4cec8af95e4f1126161e91900680b05');
  const elements=vector.integers.map(n=>{
    assert.ok(Number.isSafeInteger(n));
    const raw=Buffer.alloc(9);raw[0]=2;raw.writeBigInt64LE(BigInt(n)*(1n<<32n),1);
    return Buffer.concat([u32(9),raw]);
  });
  return Buffer.concat([
    Buffer.from('DDN-BASIC-TENSOR-VALUE-V1\0'),Buffer.from([0,1,7]),
    field(1,Buffer.from('ddn.tensor.basic')),field(2,Buffer.from([0,1])),field(3,type),
    field(4,u32(vector.shape.length)),field(5,Buffer.concat([u32(vector.shape.length),...vector.shape.map(u64)])),
    field(6,u64(elements.length)),field(7,Buffer.concat([u32(elements.length),...elements]))
  ]);
}
export function verifyTensorPhysicalCase(id,resources){
  const expected=vectors.cases[id];assert.ok(expected,id+'/physical pack coverage');
  const rows=[];
  for(const [name,vector] of Object.entries(expected)){
    // Run-summary uses value_det; persistent VM exposes the same typed carrier
    // as value_json. Neither adapter reconstructs bytes from display text.
    const leaf=(resources.value_det??resources.value_json)[name];assert.equal(leaf?.type,'tensor_v1',id+'/'+name);
    const carrier=expectedCarrier(vector),saved=Buffer.concat([Buffer.from([0x0e]),carrier]);
    assert.equal(leaf.carrier_base64url,carrier.toString('base64url'),id+'/'+name+'/exact independent carrier bytes');
    assert.equal(leaf.carrier_byte_length,carrier.length);
    assert.equal(leaf.carrier_sha256,sha(carrier));
    assert.equal(leaf.resource_value_byte_length,saved.length);
    assert.equal(leaf.resource_value_sha256,sha(saved));
    rows.push({resource:name,shape:vector.shape,carrier_sha256:sha(carrier),resource_value_sha256:sha(saved)});
  }
  return {pack_sha256:sha(pack),rows};
}

// Recompute integrity digests after poisoning meaning. Rejection must therefore
// come from the shared typed byte admission, not merely a checksum mismatch.
export function poisonTensorTransport(leaf,kind){
  const carrier=Buffer.from(leaf.carrier_base64url,'base64url');
  let offset=Buffer.byteLength('DDN-BASIC-TENSOR-VALUE-V1\0')+3;
  const fields=new Map();
  for(let i=0;i<7;i++){
    const tag=carrier[offset],length=carrier.readUInt32BE(offset+1);
    fields.set(tag,{start:offset+5,length});offset+=5+length;
  }
  assert.equal(offset,carrier.length);
  if(kind==='element-identity')carrier[fields.get(3).start+50]^=1;
  else if(kind==='shape-count')carrier[fields.get(6).start+7]^=1;
  else if(kind==='element-value-tag'){
    assert.ok(carrier.readUInt32BE(fields.get(7).start)>0);
    carrier[fields.get(7).start+8]=3;
  }else assert.fail('unknown physical mutation');
  leaf.carrier_base64url=carrier.toString('base64url');leaf.carrier_sha256=sha(carrier);
  leaf.resource_value_sha256=sha(Buffer.concat([Buffer.from([0x0e]),carrier]));
}

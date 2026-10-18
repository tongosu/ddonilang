// Physical local-host transport only. A positive probe is not a VM permit.
let available;
export async function nativeSourceAvailable(){
  if(!available)available=fetch('/',{credentials:'same-origin',cache:'no-store'})
    .then(response=>response.ok&&response.headers.get('X-DDN-Native-Source')==='v1').catch(()=>false);
  return available;
}
export async function nativeSourceRequest(operation,artifactText){
  const response=await fetch('/__ddn/vc04/source',{method:'POST',credentials:'same-origin',cache:'no-store',
    headers:{'Content-Type':'application/json'},body:JSON.stringify({operation,authored_project_json_utf8:artifactText})});
  const result=await response.json();
  if(!response.ok)throw new Error(result.error??'E_NATIVE_SOURCE_REQUEST_REJECTED');
  return result;
}

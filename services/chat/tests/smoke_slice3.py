import base64,hashlib,hmac,json,time,urllib.request,urllib.error
from urllib.parse import quote
SECRET="smoke-secret-123";BASE="http://127.0.0.1:7720"
T="44444444-4444-4444-4444-444444444444"
A="a4a4a4a4-a4a4-a4a4-a4a4-a4a4a4a4a4a4";X="e5e5e5e5-e5e5-e5e5-e5e5-e5e5e5e5e5e5"
def b64u(b):return base64.urlsafe_b64encode(b).rstrip(b"=")
def mint(sub):
    h=b64u(json.dumps({"alg":"HS256","typ":"JWT"},separators=(",",":")).encode())
    p=b64u(json.dumps({"sub":sub,"tenant_id":T,"roles":[],"exp":int(time.time())+3600},separators=(",",":")).encode())
    seg=h+b"."+p;sig=b64u(hmac.new(SECRET.encode(),seg,hashlib.sha256).digest());return (seg+b"."+sig).decode()
def req(m,p,t=None,b=None,raw=False):
    data=json.dumps(b).encode() if b is not None else None
    r=urllib.request.Request(BASE+p,data=data,method=m)
    if t:r.add_header("Authorization","Bearer "+t)
    if data is not None:r.add_header("Content-Type","application/json")
    try:
        resp=urllib.request.urlopen(r,timeout=5);body=resp.read()
        return resp.status,(body if raw else (json.loads(body) if body else None))
    except urllib.error.HTTPError as e:
        return e.code,(e.read() if raw else (e.read().decode() or ""))
P=[0];F=[0]
def ck(n,c):(P if c else F)[0]+=1;print(("  PASS " if c else "  FAIL ")+n+("" if c else " <<<"))
tA=mint(A);tX=mint(X)
s,ch=req("POST","/v1/chat/channels",tA,{"name":"vn"});ck("A creates channel",s==201);cid=ch["id"]
for txt in ["Xin chào Việt Nam","hello world","Cảm ơn bạn nhiều"]:
    s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tA,{"body":txt});ck("post: "+txt,s==201)
s,r=req("GET","/v1/chat/channels/%s/search?q=%s"%(cid,quote("viet nam")),tA);ck("search 'viet nam' finds accented msg",s==200 and len(r)==1 and "Việt" in r[0]["body"])
s,r=req("GET","/v1/chat/channels/%s/search?q=%s"%(cid,quote("CAM ON")),tA);ck("search 'CAM ON' (caps, no accent) finds 'Cảm ơn'",s==200 and len(r)==1)
s,r=req("GET","/v1/chat/channels/%s/search?q=zzzzz"%cid,tA);ck("search miss -> empty",s==200 and r==[])
s,_=req("GET","/v1/chat/channels/%s/search?q=hello"%cid,tX);ck("outsider search -> 403",s==403)
payload=b"hello-bytes-\x00\x01\x02 attach"
s,att=req("POST","/v1/chat/channels/%s/attachments"%cid,tA,{"filename":"note.bin","content_type":"application/octet-stream","data_base64":base64.b64encode(payload).decode()});ck("upload -> 201 + correct size",s==201 and att.get("size_bytes")==len(payload));aid=att["id"] if s==201 else "00000000-0000-0000-0000-000000000000"
s,raw=req("GET","/v1/chat/attachments/%s"%aid,tA,raw=True);ck("download returns the exact bytes",s==200 and raw==payload)
s,_=req("GET","/v1/chat/attachments/%s"%aid,tX,raw=True);ck("outsider download -> 403",s==403)
s,_=req("POST","/v1/chat/channels/%s/attachments"%cid,tA,{"filename":"x","data_base64":"not base64!!"});ck("bad base64 -> 400",s==400)
s,_=req("POST","/v1/chat/channels/%s/attachments"%cid,tX,{"filename":"x","data_base64":base64.b64encode(b"hi").decode()});ck("outsider upload -> 403",s==403)
print("\nRESULT: %d passed, %d failed"%(P[0],F[0]));import sys;sys.exit(1 if F[0] else 0)

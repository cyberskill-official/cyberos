import asyncio, base64, hashlib, hmac, json, time, urllib.request, urllib.error
SECRET="smoke-secret-123"; BASE="http://127.0.0.1:7720"
T1="11111111-1111-1111-1111-111111111111"; T2="22222222-2222-2222-2222-222222222222"
A="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"; B="bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"; C="cccccccc-cccc-cccc-cccc-cccccccccccc"
def b64(b): return base64.urlsafe_b64encode(b).rstrip(b"=")
def mint(sub,tenant,off=3600):
    h=b64(json.dumps({"alg":"HS256","typ":"JWT"},separators=(",",":")).encode())
    p=b64(json.dumps({"sub":sub,"tenant_id":tenant,"roles":[],"exp":int(time.time())+off},separators=(",",":")).encode())
    seg=h+b"."+p; sig=b64(hmac.new(SECRET.encode(),seg,hashlib.sha256).digest()); return (seg+b"."+sig).decode()
def req(method,path,token=None,body=None):
    data=json.dumps(body).encode() if body is not None else None
    r=urllib.request.Request(BASE+path,data=data,method=method)
    if token: r.add_header("Authorization","Bearer "+token)
    if data is not None: r.add_header("Content-Type","application/json")
    try:
        resp=urllib.request.urlopen(r,timeout=5); raw=resp.read(); return resp.status,(json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e: return e.code,e.read().decode()
P=[0]; F=[0]
def check(n,c):
    (P if c else F)[0]+=1; print(("  PASS " if c else "  FAIL ")+n)
tokA=mint(A,T1); tokB=mint(B,T1); tokC=mint(C,T2)
print("== HTTP ==")
s,_=req("POST","/v1/chat/channels",None,{"name":"x"}); check("no token -> 401", s==401)
s,_=req("POST","/v1/chat/channels","not.a.jwt",{"name":"x"}); check("garbage token -> 401", s==401)
s,ch=req("POST","/v1/chat/channels",tokA,{"name":"general"}); check("A creates channel -> 201", s==201)
cid=ch["id"] if isinstance(ch,dict) else None
s,lst=req("GET","/v1/chat/channels",tokA); check("A lists own channel", s==200 and isinstance(lst,list) and any(c["id"]==cid for c in lst))
s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tokA,{"body":"hello"}); check("A posts message -> 201", s==201)
s,ms=req("GET","/v1/chat/channels/%s/messages"%cid,tokA); check("A lists the message", s==200 and any(x["body"]=="hello" for x in ms))
s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tokB,{"body":"sneak"}); check("B (same tenant, non-member) -> 403", s==403)
s,lstC=req("GET","/v1/chat/channels",tokC); check("C (other tenant) sees no channels (RLS)", s==200 and lstC==[])
s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tokC,{"body":"x"}); check("C cross-tenant post -> 403", s==403)
print("== WEBSOCKET (live fan-out to two subscribers) ==")
import websockets
async def ws_test():
    uri="ws://127.0.0.1:7720/v1/chat/ws?channel=%s&access_token=%s"%(cid,tokA)
    async with websockets.connect(uri) as w1, websockets.connect(uri) as w2:
        await asyncio.sleep(0.3); loop=asyncio.get_event_loop()
        st,_=await loop.run_in_executor(None, lambda: req("POST","/v1/chat/channels/%s/messages"%cid,tokA,{"body":"live!"}))
        m1=json.loads(await asyncio.wait_for(w1.recv(),timeout=3)); m2=json.loads(await asyncio.wait_for(w2.recv(),timeout=3))
        return st,m1,m2
try:
    st,m1,m2=asyncio.run(ws_test())
    check("post during ws -> 201", st==201); check("ws client 1 got live message", m1.get("body")=="live!"); check("ws client 2 got live message", m2.get("body")=="live!")
except Exception as e:
    check("websocket live delivery (error: %s)"%e, False)
print("\nRESULT: %d passed, %d failed"%(P[0],F[0]))
import sys; sys.exit(1 if F[0] else 0)

import asyncio,base64,hashlib,hmac,json,time,urllib.request,urllib.error,websockets
SECRET="smoke-secret-123";HTTP="http://127.0.0.1:7720";WS="ws://127.0.0.1:7720"
T="55555555-5555-5555-5555-555555555555"
A="a5a5a5a5-a5a5-a5a5-a5a5-a5a5a5a5a5a5";B="b5b5b5b5-b5b5-b5b5-b5b5-b5b5b5b5b5b5"
def b64u(b):return base64.urlsafe_b64encode(b).rstrip(b"=")
def mint(sub):
    h=b64u(json.dumps({"alg":"HS256","typ":"JWT"},separators=(",",":")).encode())
    p=b64u(json.dumps({"sub":sub,"tenant_id":T,"roles":[],"exp":int(time.time())+3600},separators=(",",":")).encode())
    seg=h+b"."+p;sig=b64u(hmac.new(SECRET.encode(),seg,hashlib.sha256).digest());return (seg+b"."+sig).decode()
def req(m,p,t=None,b=None):
    data=json.dumps(b).encode() if b is not None else None
    r=urllib.request.Request(HTTP+p,data=data,method=m)
    if t:r.add_header("Authorization","Bearer "+t)
    if data is not None:r.add_header("Content-Type","application/json")
    try:
        resp=urllib.request.urlopen(r,timeout=5);raw=resp.read();return resp.status,(json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:return e.code,(e.read().decode() or "")
P=[0];F=[0]
def ck(n,c):(P if c else F)[0]+=1;print(("  PASS " if c else "  FAIL ")+n+("" if c else " <<<"))
tA=mint(A);tB=mint(B)
async def rj(w,t=2.0):return json.loads(await asyncio.wait_for(w.recv(),timeout=t))
async def main():
    s,ch=req("POST","/v1/chat/channels",tA,{"name":"rt"});ck("A creates channel",s==201);cid=ch["id"]
    s,_=req("POST","/v1/chat/channels/%s/members"%cid,tA,{"subject_id":B,"role":"member"});ck("owner adds B",s==201)
    s,_=req("POST","/v1/chat/devices",tB,{"platform":"ios","token":"tok-B-123"});ck("B registers device ->201",s==201)
    uriA=WS+"/v1/chat/ws?channel=%s&access_token=%s"%(cid,tA)
    uriB=WS+"/v1/chat/ws?channel=%s&access_token=%s"%(cid,tB)
    async with websockets.connect(uriA) as wA:
        await asyncio.sleep(0.3)
        s,pl=req("GET","/v1/chat/channels/%s/presence"%cid,tA);ck("presence = [A]",s==200 and pl==[A])
        async with websockets.connect(uriB) as wB:
            ev=await rj(wA);ck("A sees B come online",ev.get("type")=="presence" and ev.get("subject")==B and ev.get("status")=="online")
            s,pl=req("GET","/v1/chat/channels/%s/presence"%cid,tA);ck("presence = {A,B}",s==200 and set(pl)=={A,B})
            await wB.send(json.dumps({"type":"typing"}))
            ev=await rj(wA);ck("A sees B typing",ev.get("type")=="typing" and ev.get("subject")==B)
            await wA.send(json.dumps({"type":"signal","to":B,"data":{"sdp":"offer-1"}}))
            ev=await rj(wB);ck("B receives signal from A",ev.get("type")=="signal" and ev.get("from")==A and ev.get("data",{}).get("sdp")=="offer-1")
            try:
                await rj(wA,0.6);ck("A does NOT receive its own signal",False)
            except asyncio.TimeoutError:
                ck("A does NOT receive its own signal",True)
        ev=await rj(wA);ck("A sees B go offline",ev.get("type")=="presence" and ev.get("subject")==B and ev.get("status")=="offline")
        s,pl=req("GET","/v1/chat/channels/%s/presence"%cid,tA);ck("presence back to [A]",s==200 and pl==[A])
    s,m1=req("POST","/v1/chat/channels/%s/messages"%cid,tA,{"body":"r1"})
    s,m2=req("POST","/v1/chat/channels/%s/messages"%cid,tA,{"body":"r2"})
    s,u=req("GET","/v1/chat/channels/%s/unread"%cid,tB);ck("B unread = 2",s==200 and u.get("unread")==2)
    s,_=req("POST","/v1/chat/channels/%s/read"%cid,tB,{"message_id":m1["id"]});ck("B marks m1 read ->204",s==204)
    s,u=req("GET","/v1/chat/channels/%s/unread"%cid,tB);ck("B unread = 1 after reading m1",s==200 and u.get("unread")==1)
    s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tA,{"body":"ping B"})
    await asyncio.sleep(0.5)
asyncio.run(main())
print("\nRESULT: %d passed, %d failed"%(P[0],F[0]));import sys;sys.exit(1 if F[0] else 0)

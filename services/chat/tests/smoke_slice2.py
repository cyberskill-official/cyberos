import base64,hashlib,hmac,json,time,urllib.request,urllib.error
SECRET="smoke-secret-123";BASE="http://127.0.0.1:7720"
T="33333333-3333-3333-3333-333333333333"
A="a1a1a1a1-a1a1-a1a1-a1a1-a1a1a1a1a1a1";B="b2b2b2b2-b2b2-b2b2-b2b2-b2b2b2b2b2b2";C="c3c3c3c3-c3c3-c3c3-c3c3-c3c3c3c3c3c3"
def b64(b):return base64.urlsafe_b64encode(b).rstrip(b"=")
def mint(sub):
    h=b64(json.dumps({"alg":"HS256","typ":"JWT"},separators=(",",":")).encode())
    p=b64(json.dumps({"sub":sub,"tenant_id":T,"roles":[],"exp":int(time.time())+3600},separators=(",",":")).encode())
    seg=h+b"."+p;sig=b64(hmac.new(SECRET.encode(),seg,hashlib.sha256).digest());return (seg+b"."+sig).decode()
def req(m,p,t=None,b=None):
    data=json.dumps(b).encode() if b is not None else None
    r=urllib.request.Request(BASE+p,data=data,method=m)
    if t:r.add_header("Authorization","Bearer "+t)
    if data is not None:r.add_header("Content-Type","application/json")
    try:
        resp=urllib.request.urlopen(r,timeout=5);raw=resp.read();return resp.status,(json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:return e.code,(e.read().decode() or "")
P=[0];F=[0]
def ck(n,c):(P if c else F)[0]+=1;print(("  PASS " if c else "  FAIL ")+n+("" if c else " <<<"))
tA=mint(A);tB=mint(B);tC=mint(C)
s,ch=req("POST","/v1/chat/channels",tA,{"name":"team"});ck("A creates channel",s==201);cid=ch["id"]
s,_=req("POST","/v1/chat/channels/%s/members"%cid,tA,{"subject_id":B,"role":"member"});ck("owner adds B ->201",s==201)
s,_=req("POST","/v1/chat/channels/%s/members"%cid,tB,{"subject_id":C});ck("member B cannot add ->403",s==403)
s,_=req("POST","/v1/chat/channels/%s/members"%cid,tC,{"subject_id":C});ck("outsider C cannot add ->403",s==403)
s,mb=req("POST","/v1/chat/channels/%s/messages"%cid,tB,{"body":"hi from B"});ck("B (member) posts ->201",s==201);mbid=mb["id"]
s,m1=req("POST","/v1/chat/channels/%s/messages"%cid,tA,{"body":"top level"});ck("A posts top-level ->201",s==201);m1id=m1["id"]
s,rep=req("POST","/v1/chat/channels/%s/messages"%cid,tB,{"body":"a reply","parent_id":m1id});ck("B replies in thread ->201",s==201 and rep.get("parent_id")==m1id)
s,top=req("GET","/v1/chat/channels/%s/messages"%cid,tA);ck("top-level list excludes reply",s==200 and all(x["parent_id"] is None for x in top) and any(x["id"]==m1id for x in top))
s,thr=req("GET","/v1/chat/channels/%s/messages?parent_id=%s"%(cid,m1id),tA);ck("thread list returns the reply",s==200 and any(x["id"]==rep["id"] for x in thr))
s,ed=req("PATCH","/v1/chat/channels/%s/messages/%s"%(cid,mbid),tB,{"body":"edited by B"});ck("B edits own msg ->200 + edited_at",s==200 and ed.get("edited_at") and ed["body"]=="edited by B")
s,_=req("PATCH","/v1/chat/channels/%s/messages/%s"%(cid,m1id),tB,{"body":"hijack"});ck("B cannot edit A's msg ->404",s==404)
s,_=req("DELETE","/v1/chat/channels/%s/messages/%s"%(cid,mbid),tB);ck("B deletes own msg ->204",s==204)
s,after=req("GET","/v1/chat/channels/%s/messages"%cid,tA);ck("deleted msg gone from list",s==200 and all(x["id"]!=mbid for x in after))
s,_=req("DELETE","/v1/chat/channels/%s/messages/%s"%(cid,rep["id"]),tA);ck("owner manager-deletes member reply ->204",s==204)
s,_=req("DELETE","/v1/chat/channels/%s/members/%s"%(cid,B),tB);ck("member cannot remove members ->403",s==403)
s,_=req("DELETE","/v1/chat/channels/%s/members/%s"%(cid,B),tA);ck("owner removes B ->204",s==204)
s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tB,{"body":"after removal"});ck("removed B cannot post ->403",s==403)
print("\nRESULT: %d passed, %d failed"%(P[0],F[0]));import sys;sys.exit(1 if F[0] else 0)

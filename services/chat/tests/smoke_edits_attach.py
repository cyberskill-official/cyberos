import asyncio, json, base64, urllib.request, urllib.error, websockets
HTTP="http://127.0.0.1:7720"; WS="ws://127.0.0.1:7720"; AUTH="http://127.0.0.1:7700"
PNG="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
def grant():
    req=urllib.request.Request(AUTH+"/v1/auth/token", data=json.dumps({"grant_type":"password","tenant_slug":"cyberskill","handle":"@stephen","password":"CyberOS-Demo-2026!"}).encode(), headers={"content-type":"application/json"}, method="POST")
    return json.load(urllib.request.urlopen(req))["access_token"]
def req(method, path, token, body=None):
    data=json.dumps(body).encode() if body is not None else None
    h={"Authorization":"Bearer "+token}
    if data is not None: h["content-type"]="application/json"
    r=urllib.request.Request(HTTP+path, data=data, method=method, headers=h)
    try:
        resp=urllib.request.urlopen(r); raw=resp.read(); return resp.status,(json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e: return e.code, e.read().decode()
P=[0];F=[0]
def ck(n,c):(P if c else F)[0]+=1;print(("  PASS " if c else "  FAIL ")+n+("" if c else " <<<"))
async def rj(w,t=3.0): return json.loads(await asyncio.wait_for(w.recv(), timeout=t))
async def main():
    tok=grant()
    s,ch=req("POST","/v1/chat/channels",tok,{"name":"edits-test"}); ck("create channel", s==201); cid=ch["id"]
    uri=WS+"/v1/chat/ws?channel=%s&access_token=%s"%(cid,tok)
    async with websockets.connect(uri) as w:
        await asyncio.sleep(0.3)
        s,m=req("POST","/v1/chat/channels/%s/messages"%cid,tok,{"body":"original"}); ck("post message", s==201); mid=m["id"]
        ev=await rj(w); ck("ws receives message", ev.get("type")=="message" and ev.get("id")==mid)
        s,_=req("PATCH","/v1/chat/channels/%s/messages/%s"%(cid,mid),tok,{"body":"edited now"}); ck("edit -> 200", s==200)
        ev=await rj(w); ck("ws receives message_edited (live)", ev.get("type")=="message_edited" and ev.get("id")==mid and ev.get("body")=="edited now")
        s,_=req("DELETE","/v1/chat/channels/%s/messages/%s"%(cid,mid),tok); ck("delete -> 204", s==204)
        ev=await rj(w); ck("ws receives message_deleted (live)", ev.get("type")=="message_deleted" and ev.get("id")==mid)
        s,att=req("POST","/v1/chat/channels/%s/attachments"%cid,tok,{"filename":"tiny.png","content_type":"image/png","data_base64":PNG}); ck("upload attachment", s==201); aid=att["id"]
        s,am=req("POST","/v1/chat/channels/%s/messages"%cid,tok,{"body":"","attachment_id":aid}); ck("post attachment-link msg (empty body ok)", s==201 and am.get("attachment_id")==aid)
        ev=await rj(w); ck("ws message carries attachment_id", ev.get("type")=="message" and ev.get("attachment_id")==aid)
        s,meta=req("GET","/v1/chat/attachments/%s/meta"%aid,tok); ck("attachment meta endpoint", s==200 and meta.get("filename")=="tiny.png" and meta.get("content_type")=="image/png")
        # reject: empty body + no attachment
        s,_=req("POST","/v1/chat/channels/%s/messages"%cid,tok,{"body":"   "}); ck("empty body w/o attachment -> 400", s==400)
    print("\nRESULT: %d passed, %d failed"%(P[0],F[0])); import sys; sys.exit(1 if F[0] else 0)
asyncio.run(main())

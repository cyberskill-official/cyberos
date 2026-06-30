import asyncio, json, urllib.parse, urllib.request, urllib.error, websockets
HTTP="http://127.0.0.1:7720"; WS="ws://127.0.0.1:7720"; AUTH="http://127.0.0.1:7700"
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
    s,ch=req("POST","/v1/chat/channels",tok,{"name":"reactions-test"}); ck("create channel", s==201); cid=ch["id"]
    uri=WS+"/v1/chat/ws?channel=%s&access_token=%s"%(cid,tok)
    async with websockets.connect(uri) as w:
        await asyncio.sleep(0.3)
        s,m=req("POST","/v1/chat/channels/%s/messages"%cid,tok,{"body":"react to me"}); ck("post message", s==201); mid=m["id"]
        ev=await rj(w); ck("ws receives message", ev.get("type")=="message" and ev.get("id")==mid)
        # add a reaction (idempotent)
        s,_=req("POST","/v1/chat/channels/%s/messages/%s/reactions"%(cid,mid),tok,{"emoji":"\U0001F44D"}); ck("add reaction -> 204", s==204)
        ev=await rj(w); ck("ws reaction_changed added", ev.get("type")=="reaction_changed" and ev.get("message_id")==mid and ev.get("emoji")=="\U0001F44D" and ev.get("added") is True)
        s,_=req("POST","/v1/chat/channels/%s/messages/%s/reactions"%(cid,mid),tok,{"emoji":"\U0001F44D"}); ck("add same reaction again -> 204 (idempotent)", s==204)
        # list folds the reaction with count+mine
        s,msgs=req("GET","/v1/chat/channels/%s/messages"%cid,tok); ck("list -> 200", s==200)
        row=next((x for x in (msgs or []) if x["id"]==mid), None)
        rs=(row or {}).get("reactions") or []
        thumb=next((r for r in rs if r.get("emoji")=="\U0001F44D"), None)
        ck("message carries folded reaction (count 1, mine)", bool(thumb) and thumb.get("count")==1 and thumb.get("mine") is True)
        # remove the caller's own reaction
        emoji_path=urllib.parse.quote("\U0001F44D", safe="")
        s,_=req("DELETE","/v1/chat/channels/%s/messages/%s/reactions/%s"%(cid,mid,emoji_path),tok); ck("remove reaction -> 204", s==204)
        ev=await rj(w); ck("ws reaction_changed removed", ev.get("type")=="reaction_changed" and ev.get("message_id")==mid and ev.get("added") is False)
        # removing again -> 404 (nothing of the caller's left)
        s,_=req("DELETE","/v1/chat/channels/%s/messages/%s/reactions/%s"%(cid,mid,emoji_path),tok); ck("remove again -> 404", s==404)
        # reaction gone from the list
        s,msgs=req("GET","/v1/chat/channels/%s/messages"%cid,tok)
        row=next((x for x in (msgs or []) if x["id"]==mid), None)
        ck("reaction cleared from list", not ((row or {}).get("reactions") or []))
        # empty emoji rejected
        s,_=req("POST","/v1/chat/channels/%s/messages/%s/reactions"%(cid,mid),tok,{"emoji":"  "}); ck("empty emoji -> 400", s==400)
    print("\nRESULT: %d passed, %d failed"%(P[0],F[0])); import sys; sys.exit(1 if F[0] else 0)
asyncio.run(main())

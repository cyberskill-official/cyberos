# Remote chat LLM on a second Vultr box (over a private VPC)

Runs the chat translation / assistant LLM (chat.smart, chat.fast) on a dedicated Vultr instance instead of
the P0 box, so the P0 box does not need a RAM resize. The gateway keeps provider `kind: ollama`, so the ZDR,
residency, and cost gates in `deploy/vps/ai/tenants/org-cyberskill.yaml` stay exactly as they are.

Design in one line: a second Vultr instance in the SAME region as P0 (Singapore), joined to the P0 box by a
Vultr VPC 2.0 private network, running Ollama bound to the private IP only. The gateway's `OLLAMA_ENDPOINT`
points at that private IP. Nothing about the LLM is exposed to the internet, and the data stays in Singapore
(so `residency: sg-1` is physically true, which matters for PDPD).

Why same region + VPC: a Vultr VPC 2.0 is private, isolated from other tenants, and free for traffic between
instances, but it only spans instances in the same region. Ollama has no built-in auth, so it must never get
a public listener; the VPC removes the need for an auth proxy entirely.

## 1. Provision the LLM instance (Vultr, Singapore)

- Region: Singapore (same as the P0 box).
- Start on High Frequency Compute, 4 vCPU / 16 GB (about $96/mo). That runs `qwen2.5:3b-instruct` (about
  2.5 GB RSS) with lots of headroom and can host a 7-8B quantized model for better Vietnamese.
- Upgrade path if latency or a larger model needs it: a Vultr fractional Cloud GPU in Singapore (L40S or A16).
  Same region, same runbook - only the instance type changes.
- OS: Ubuntu 24.04 LTS.

## 2. Put both boxes on one VPC 2.0

- In the Vultr panel: Network -> VPC 2.0 -> add a VPC in Singapore (or reuse one).
- Attach BOTH the P0 box and the new LLM box to it.
- Note each box's private (RFC1918) address, e.g. the LLM box gets `10.2.0.4`. Confirm from the P0 box:

      ping -c1 <LLM_PRIVATE_IP>

## 3. Install Ollama on the LLM box, bound to the private IP only

    curl -fsSL https://ollama.com/install.sh | sh

Bind Ollama to the VPC address only (never the public interface). Create a systemd override:

    sudo systemctl edit ollama

Add:

    [Service]
    Environment="OLLAMA_HOST=http://<LLM_PRIVATE_IP>:11434"

Then:

    sudo systemctl daemon-reload
    sudo systemctl restart ollama

Firewall as defense in depth - drop public `:11434`, allow only the VPC subnet:

    sudo ufw allow from <VPC_SUBNET_CIDR> to any port 11434 proto tcp
    sudo ufw deny 11434
    sudo ufw enable

Confirm it is NOT reachable on the public IP (should time out / refuse):

    curl --max-time 3 http://<LLM_PUBLIC_IP>:11434/api/tags   # must fail

## 4. Pull the model on the LLM box

    ollama pull qwen2.5:3b-instruct
    # or a larger model for better VN<->EN, e.g. `ollama pull qwen2.5:7b-instruct`
    # (if you change it, also update model_alias_map in deploy/vps/ai/tenants/org-cyberskill.yaml)

## 5. Point the P0 gateway at it and redeploy

On the P0 box, in `.env.p0`:

    OLLAMA_ENDPOINT=http://<LLM_PRIVATE_IP>:11434
    OLLAMA_CHAT_MODEL=qwen2.5:3b-instruct

Leave `COMPOSE_PROFILES` WITHOUT `llm` (the in-compose ollama stays off - the model runs on the remote box).
Redeploy:

    cd /path/to/cyberos/deploy/vps && bash deploy.sh

`OLLAMA_ENDPOINT` in `docker-compose.p0*.yml` is `${OLLAMA_ENDPOINT:-http://ollama:11434}`, so the env value
takes effect with no compose edit. deploy.sh only pulls into a LOCAL ollama when the `llm` profile is on, so
with the remote endpoint it correctly skips that step.

## 6. Verify

- Gateway sees a healthy provider: `curl -fsS https://os.cyberskill.world/status/ai` (the status page probes
  the gateway `/healthz`).
- From the P0 box, the private path works: `curl http://<LLM_PRIVATE_IP>:11434/api/tags` lists the model.
- End to end: in chat, hit Translate on a message (chat.smart -> ollama). It should return a translation
  instead of the "translation unavailable" note.

## Notes

- Keep the provider `kind: ollama`. A managed cloud API would carry a region and retention and would trip
  `zdr_required: true` + `residency: sg-1`, and needs the deferred cloud-key adapter first.
- Residency: keeping the LLM box in Singapore keeps chat text in-region. A box in another region would still
  "pass" the sg-1 pin as coded (the gateway treats ollama as region-less), but the data would physically
  leave Singapore - a conscious PDPD choice, so prefer Singapore.
- Cost: the private VPC traffic is free; you pay only for the second instance. Start CPU; move to a fractional
  GPU only if the team's volume or a larger model needs it.
- Security: Ollama is unauthenticated by design - the VPC bind + firewall is what protects it. Never give it a
  public listener. If you ever must reach it off-VPC, front it with Caddy + a bearer token + an IP allowlist
  instead (a small gateway adapter change to attach the auth header).

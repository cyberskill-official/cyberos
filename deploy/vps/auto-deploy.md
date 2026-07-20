# Auto-deploy on push to main

Every push to main builds the auth and chat images on a GitHub runner, pushes them to GHCR, and rolls the VPS to the new images. The VPS never compiles Rust - it only pulls and restarts - which matters on a 4 GB box where a release build would be slow and could run out of memory. The console, Caddyfile, and compose come from the VPS git checkout, so a console-only change ships by a plain `git pull` with no rebuild.

The pieces: `.github/workflows/deploy.yml` (build + push + trigger), `deploy/vps/docker-compose.p0.images.yml` (prod compose that runs the GHCR images), and `deploy/vps/deploy.sh` (the script the VPS runs).

## One-time setup

Do this once. The very first bring-up is still the manual one from the P0 runbook (it creates `.env.p0` and starts the stack); after that, pushes roll it automatically.

### On the VPS (as linuxuser)

1. Install Docker with the compose plugin:

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker "$USER"   # log out and back in so the group applies
```

2. Give the VPS read access to the private repo with a deploy key, then clone to `~/cyberos`:

```bash
ssh-keygen -t ed25519 -C "cyberos-vps-deploy" -f ~/.ssh/id_ed25519 -N ""
cat ~/.ssh/id_ed25519.pub
# Add that public key in GitHub: repo -> Settings -> Deploy keys -> Add deploy key (read-only is enough).
git clone git@github.com:cyberskill-official/cyberos.git ~/cyberos
chmod +x ~/cyberos/deploy/vps/deploy.sh
```

3. Log Docker into GHCR so it can pull the private images (create a GitHub personal access token with the read:packages scope):

```bash
echo "<YOUR_READ_PACKAGES_PAT>" | docker login ghcr.io -u <your-github-username> --password-stdin
```

(Alternatively, set the two GHCR packages to public after the first workflow run and skip this login.)

4. Let GitHub Actions SSH in. Create a key pair for the CI connection - the private half becomes a GitHub secret below, the public half goes on the VPS:

```bash
ssh-keygen -t ed25519 -C "cyberos-ci" -f ~/ci_key -N ""
cat ~/ci_key.pub >> ~/.ssh/authorized_keys
cat ~/ci_key            # copy this PRIVATE key for the VPS_SSH_KEY secret, then delete it: rm ~/ci_key*
```

5. Do the initial bring-up once (P0 runbook Step 3b) so `.env.p0` exists and the stack is healthy.

### In GitHub (repo Settings -> Secrets and variables -> Actions)

Add three secrets:

- `VPS_HOST` - the server IP or domain (149.28.158.169 for now, or os.cyberskill.world once DNS is set).
- `VPS_USER` - `linuxuser`.
- `VPS_SSH_KEY` - the private CI key from step 4 (the whole `~/ci_key` contents).

The workflow uses the built-in `GITHUB_TOKEN` to push to GHCR, so no registry secret is needed.

## After setup

Push to main. The deploy workflow builds the images, pushes them, and runs `deploy.sh` on the VPS. Watch it under the repo's Actions tab. You can also trigger it by hand from there (Run workflow), since the workflow allows `workflow_dispatch`.

Manual deploy from the VPS, any time:

```bash
bash ~/cyberos/deploy/vps/deploy.sh
```

Roll back to a previous build: every image is also tagged with its commit sha. Set the tag in `.env.p0` (or export it) and re-run:

```bash
cd ~/cyberos/deploy/vps
CYBEROS_IMAGE_TAG=<previous-commit-sha> docker compose --env-file .env.p0 -f docker-compose.p0.images.yml up -d
```

## Notes

- The workflow deploys on every push to main that touches services, the console, or the deploy files. It is gated only by the image build succeeding (a compile break fails the build and skips the deploy). The full test suite still runs separately in `services.yml`; if you want tests to block deploys, add a dependency on that workflow.
- Migrations are not run automatically. A schema change is rare and applied to Supabase by hand (P0 runbook Step 3) before or alongside the code that needs it, so a deploy never surprises the database.
- The simpler alternative - SSH in and `docker compose up -d --build` on the VPS - was avoided on purpose: the 4 GB box would compile the Rust workspace on every deploy, which is slow and memory-tight. Building in CI keeps deploys fast and the box small.

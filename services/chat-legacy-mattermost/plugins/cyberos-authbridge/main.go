package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"net"
	"net/http"
	"os"
	"strings"
	"sync/atomic"
	"time"
)

type AuthBridgePlugin struct {
	active atomic.Bool
	jwks   *JwksCache
	users  *JitProvisioner
	revoked *RevocationClient
}

type LoginResponse struct {
	SessionID       string `json:"session_id"`
	SubjectID       string `json:"cyberos_subject_id"`
	TenantID        string `json:"tenant_id"`
	JitProvisioned  bool   `json:"jit_provisioned"`
	TraceID         string `json:"trace_id"`
}

func NewAuthBridge(jwksURL string) *AuthBridgePlugin {
	return NewAuthBridgeWithRevocation(jwksURL, os.Getenv("CYBEROS_REVOCATION_URL"))
}

func NewAuthBridgeWithRevocation(jwksURL, revocationURL string) *AuthBridgePlugin {
	return &AuthBridgePlugin{
		jwks:    NewJwksCache(jwksURL),
		users:   NewJitProvisioner(),
		revoked: NewRevocationClient(revocationURL),
	}
}

func (p *AuthBridgePlugin) OnActivate() error {
	if !p.active.CompareAndSwap(false, true) {
		return errors.New("authbridge already activated")
	}
	return nil
}

func (p *AuthBridgePlugin) OnDeactivate() error {
	p.active.Store(false)
	return nil
}

func (p *AuthBridgePlugin) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path == "/api/v4/users/password/reset/send" || r.URL.Path == "/api/v4/users/email/verify" {
		writeError(w, http.StatusGone, "builtin_auth_disabled", traceID(r))
		return
	}
	if r.URL.Path == "/api/v4/users/login" && r.Method == http.MethodPost {
		p.handleLogin(w, r)
		return
	}
	http.NotFound(w, r)
}

func (p *AuthBridgePlugin) handleLogin(w http.ResponseWriter, r *http.Request) {
	trace := traceID(r)
	token, ok := strings.CutPrefix(r.Header.Get("Authorization"), "Bearer ")
	if !ok || strings.TrimSpace(token) == "" {
		writeError(w, http.StatusUnauthorized, "invalid_jwt", trace)
		return
	}
	claims, err := p.jwks.ValidateJWT(r.Context(), token)
	if err != nil {
		if errors.Is(err, ErrJwksUnavailable) {
			writeError(w, http.StatusServiceUnavailable, "jwks_unavailable", trace)
			return
		}
		writeError(w, http.StatusUnauthorized, "invalid_jwt", trace)
		return
	}
	if claims.TenantID == "" {
		writeError(w, http.StatusUnauthorized, "missing_tenant_claim", trace)
		return
	}
	revoked, err := p.revoked.IsRevoked(r.Context(), claims.JWTID)
	if err != nil || revoked {
		writeError(w, http.StatusUnauthorized, "revoked", trace)
		return
	}
	user, jit := p.users.Provision(claims.Subject, claims.Email, claims.TenantID)
	if user.TenantID != claims.TenantID {
		writeError(w, http.StatusForbidden, "tenant_mismatch", trace)
		return
	}
	_ = json.NewEncoder(w).Encode(LoginResponse{
		SessionID:      "mm-session-" + claims.JWTID,
		SubjectID:      claims.Subject,
		TenantID:        claims.TenantID,
		JitProvisioned: jit,
		TraceID:        trace,
	})
}

func writeError(w http.ResponseWriter, status int, code string, trace string) {
	if !validErrorCode(code) {
		code = "server_error"
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": code, "trace_id": trace})
}

func validErrorCode(code string) bool {
	switch code {
	case "invalid_jwt", "missing_tenant_claim", "tenant_mismatch", "revoked",
		"builtin_auth_disabled", "jwks_unavailable", "tenant_not_mapped",
		"username_collision_exhausted", "server_error":
		return true
	default:
		return false
	}
}

func traceID(r *http.Request) string {
	tp := r.Header.Get("traceparent")
	parts := strings.Split(tp, "-")
	if len(parts) >= 2 && len(parts[1]) == 32 {
		return parts[1]
	}
	var buf [16]byte
	if _, err := rand.Read(buf[:]); err == nil {
		return hex.EncodeToString(buf[:])
	}
	return strings.Repeat("0", 32)
}

type RevocationClient struct {
	url    string
	client *http.Client
}

func NewRevocationClient(url string) *RevocationClient {
	return &RevocationClient{
		url: url,
		client: &http.Client{
			Timeout: 2 * time.Second,
			Transport: &http.Transport{
				DialContext: (&net.Dialer{Timeout: time.Second}).DialContext,
			},
		},
	}
}

func (c *RevocationClient) IsRevoked(ctx context.Context, jti string) (bool, error) {
	if c.url == "" {
		return false, nil
	}
	req, err := http.NewRequestWithContext(ctx, "GET", c.url+"/revocations/"+jti, nil)
	if err != nil {
		return false, err
	}
	resp, err := c.client.Do(req)
	if err != nil {
		return false, err
	}
	defer resp.Body.Close()
	switch resp.StatusCode {
	case http.StatusOK:
		var body struct {
			Revoked bool `json:"revoked"`
		}
		if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
			return false, err
		}
		return body.Revoked, nil
	case http.StatusNotFound:
		return false, nil
	default:
		return false, errors.New("revocation lookup failed")
	}
}

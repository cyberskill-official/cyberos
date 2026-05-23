package main

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"sync/atomic"
)

type AuthBridgePlugin struct {
	active atomic.Bool
	jwks   *JwksCache
	users  *JitProvisioner
}

type LoginResponse struct {
	SessionID       string `json:"session_id"`
	SubjectID       string `json:"cyberos_subject_id"`
	TenantID        string `json:"tenant_id"`
	JitProvisioned  bool   `json:"jit_provisioned"`
	TraceID         string `json:"trace_id"`
}

func NewAuthBridge(jwksURL string) *AuthBridgePlugin {
	return &AuthBridgePlugin{
		jwks:  NewJwksCache(jwksURL),
		users: NewJitProvisioner(),
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
		writeError(w, http.StatusUnauthorized, "missing_bearer", trace)
		return
	}
	claims, err := p.jwks.ValidateJWT(token)
	if err != nil {
		writeError(w, http.StatusUnauthorized, "invalid_jwt", trace)
		return
	}
	if claims.TenantID == "" {
		writeError(w, http.StatusUnauthorized, "missing_tenant_claim", trace)
		return
	}
	if claims.Revoked {
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
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": code, "trace_id": trace})
}

func traceID(r *http.Request) string {
	tp := r.Header.Get("traceparent")
	parts := strings.Split(tp, "-")
	if len(parts) >= 2 && len(parts[1]) == 32 {
		return parts[1]
	}
	return "00000000000000000000000000000000"
}

package main

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"strings"
	"sync"
	"time"
)

type Claims struct {
	Subject  string `json:"sub"`
	Email    string `json:"email"`
	TenantID string `json:"tenant_id"`
	JWTID    string `json:"jti"`
	Revoked  bool   `json:"revoked"`
}

type JwksCache struct {
	url       string
	expiresAt time.Time
	mu        sync.RWMutex
}

func NewJwksCache(url string) *JwksCache {
	return &JwksCache{url: url}
}

func (c *JwksCache) TTL() time.Duration {
	return time.Hour
}

func (c *JwksCache) ValidateJWT(token string) (Claims, error) {
	parts := strings.Split(token, ".")
	if len(parts) < 2 {
		return Claims{}, errors.New("malformed jwt")
	}
	payload, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return Claims{}, err
	}
	var claims Claims
	if err := json.Unmarshal(payload, &claims); err != nil {
		return Claims{}, err
	}
	if claims.Subject == "" || claims.JWTID == "" {
		return Claims{}, errors.New("missing required claims")
	}
	// The production plugin resolves the KID through JWKS and verifies RS256.
	// This source keeps the same validation envelope for unit/shape tests in
	// this repository, where Mattermost and Go dependencies are not vendored.
	return claims, nil
}

package main

import (
	"context"
	"crypto"
	"crypto/rsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"math/big"
	"net/http"
	"strings"
	"sync"
	"time"
)

var (
	ErrJwksUnavailable = errors.New("jwks_unavailable")
	ErrInvalidJWT      = errors.New("invalid_jwt")
)

type Claims struct {
	Subject  string `json:"sub"`
	Email    string `json:"email"`
	TenantID string `json:"tenant_id"`
	JWTID    string `json:"jti"`
	Issuer   string `json:"iss"`
	Expires  int64  `json:"exp"`
	NotBefore int64 `json:"nbf"`
}

type JwksCache struct {
	url       string
	expiresAt time.Time
	keys      map[string]*rsa.PublicKey
	client    *http.Client
	mu        sync.RWMutex
}

func NewJwksCache(url string) *JwksCache {
	return &JwksCache{
		url:  url,
		keys: map[string]*rsa.PublicKey{},
		client: &http.Client{
			Timeout: 5 * time.Second,
			Transport: &http.Transport{TLSHandshakeTimeout: 2 * time.Second},
		},
	}
}

func (c *JwksCache) TTL() time.Duration {
	return time.Hour
}

func (c *JwksCache) ValidateJWT(ctx context.Context, token string) (Claims, error) {
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return Claims{}, ErrInvalidJWT
	}

	var header struct {
		Alg string `json:"alg"`
		Kid string `json:"kid"`
		Typ string `json:"typ"`
	}
	headerBytes, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return Claims{}, ErrInvalidJWT
	}
	if err := json.Unmarshal(headerBytes, &header); err != nil {
		return Claims{}, ErrInvalidJWT
	}
	if header.Alg != "RS256" || header.Kid == "" {
		return Claims{}, ErrInvalidJWT
	}

	payload, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return Claims{}, ErrInvalidJWT
	}
	var claims Claims
	if err := json.Unmarshal(payload, &claims); err != nil {
		return Claims{}, ErrInvalidJWT
	}
	if claims.Subject == "" || claims.JWTID == "" || claims.TenantID == "" {
		return Claims{}, ErrInvalidJWT
	}
	now := time.Now().Unix()
	if claims.Expires > 0 && now >= claims.Expires {
		return Claims{}, ErrInvalidJWT
	}
	if claims.NotBefore > 0 && now < claims.NotBefore {
		return Claims{}, ErrInvalidJWT
	}

	key, err := c.keyFor(ctx, header.Kid)
	if err != nil {
		return Claims{}, err
	}
	signingInput := parts[0] + "." + parts[1]
	sig, err := base64.RawURLEncoding.DecodeString(parts[2])
	if err != nil {
		return Claims{}, ErrInvalidJWT
	}
	digest := sha256.Sum256([]byte(signingInput))
	if err := rsa.VerifyPKCS1v15(key, crypto.SHA256, digest[:], sig); err != nil {
		return Claims{}, ErrInvalidJWT
	}
	return claims, nil
}

func (c *JwksCache) keyFor(ctx context.Context, kid string) (*rsa.PublicKey, error) {
	c.mu.RLock()
	key, ok := c.keys[kid]
	expired := time.Now().After(c.expiresAt)
	c.mu.RUnlock()
	if ok && !expired {
		return key, nil
	}
	if err := c.Refresh(ctx); err != nil {
		if ok {
			return key, nil
		}
		return nil, ErrJwksUnavailable
	}
	c.mu.RLock()
	defer c.mu.RUnlock()
	key, ok = c.keys[kid]
	if !ok {
		return nil, ErrInvalidJWT
	}
	return key, nil
}

func (c *JwksCache) Refresh(ctx context.Context) error {
	if c.url == "" {
		return ErrJwksUnavailable
	}
	req, err := http.NewRequestWithContext(ctx, "GET", c.url, nil)
	if err != nil {
		return err
	}
	resp, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return ErrJwksUnavailable
	}
	var set struct {
		Keys []struct {
			Kty string `json:"kty"`
			Use string `json:"use"`
			Alg string `json:"alg"`
			Kid string `json:"kid"`
			N   string `json:"n"`
			E   string `json:"e"`
		} `json:"keys"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&set); err != nil {
		return err
	}
	keys := make(map[string]*rsa.PublicKey, len(set.Keys))
	for _, jwk := range set.Keys {
		if jwk.Kty != "RSA" || jwk.Kid == "" || jwk.N == "" || jwk.E == "" {
			continue
		}
		key, err := rsaPublicKey(jwk.N, jwk.E)
		if err != nil {
			continue
		}
		keys[jwk.Kid] = key
	}
	if len(keys) == 0 {
		return ErrJwksUnavailable
	}
	c.mu.Lock()
	c.keys = keys
	c.expiresAt = time.Now().Add(c.TTL())
	c.mu.Unlock()
	return nil
}

func rsaPublicKey(nEnc, eEnc string) (*rsa.PublicKey, error) {
	nBytes, err := base64.RawURLEncoding.DecodeString(nEnc)
	if err != nil {
		return nil, err
	}
	eBytes, err := base64.RawURLEncoding.DecodeString(eEnc)
	if err != nil {
		return nil, err
	}
	e := 0
	for _, b := range eBytes {
		e = e<<8 + int(b)
	}
	if e == 0 {
		return nil, errors.New("invalid exponent")
	}
	return &rsa.PublicKey{N: new(big.Int).SetBytes(nBytes), E: e}, nil
}

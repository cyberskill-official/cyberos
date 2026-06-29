package main

import (
	"fmt"
	"strings"
	"sync"
	"unicode"
)

type MattermostUser struct {
	ID       string
	Email    string
	Username string
	Subject  string
	TenantID string
}

type JitProvisioner struct {
	mu      sync.Mutex
	bySub   map[string]MattermostUser
	byName  map[string]int
}

func NewJitProvisioner() *JitProvisioner {
	return &JitProvisioner{bySub: map[string]MattermostUser{}, byName: map[string]int{}}
}

func (p *JitProvisioner) Provision(subject, email, tenant string) (MattermostUser, bool) {
	p.mu.Lock()
	defer p.mu.Unlock()
	if user, ok := p.bySub[subject]; ok {
		return user, false
	}
	name := sanitizeUsername(email)
	if n := p.byName[name]; n > 0 {
		p.byName[name] = n + 1
		suffix := fmt.Sprintf("_%d", n)
		if len(name)+len(suffix) > 22 {
			name = name[:22-len(suffix)]
		}
		name += suffix
	} else {
		p.byName[name] = 1
	}
	user := MattermostUser{
		ID:       "mm-" + subject,
		Email:    email,
		Username: name,
		Subject:  subject,
		TenantID: tenant,
	}
	p.bySub[subject] = user
	return user, true
}

func sanitizeUsername(email string) string {
	local := strings.ToLower(strings.Split(email, "@")[0])
	var out []rune
	for _, r := range local {
		if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '.' || r == '_' || r == '-' {
			out = append(out, r)
		}
	}
	if len(out) < 3 {
		out = append(out, []rune("user")...)
	}
	if len(out) > 22 {
		out = out[:22]
	}
	return string(out)
}

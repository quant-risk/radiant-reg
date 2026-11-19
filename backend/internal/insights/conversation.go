// Package insights — ConversationStore e ResponseCache (aposentados na Etapa 0).
//
// O histórico conversacional legado e o ResponseCache (cujo Invalidate apagava
// entradas cross-tenant) foram aposentados na Etapa 0 por compliance LC 105 /
// isolamento multi-tenant (ver docs/blueprint/21-ENGINE-IA.md).
package insights

import (
	"context"
	"database/sql"
	"errors"
	"time"
)

// ErrConversationNotFound is kept for compatibility; legacy conversation store is retired.
var ErrConversationNotFound = errors.New("conversation not found")

// ConversationStore is the retired per-tenant conversation store (fail-closed with ErrLegacyRetired).
type ConversationStore struct{}

// NewConversationStore returns an inert fail-closed ConversationStore without retaining the DB handle.
func NewConversationStore(_ *sql.DB) *ConversationStore {
	return &ConversationStore{}
}

// SaveMessage fails closed with ErrLegacyRetired.
func (c *ConversationStore) SaveMessage(_ context.Context, _ ConversationMessage) (string, error) {
	return "", ErrLegacyRetired
}

// ConversationMessage represents a stored conversation message.
type ConversationMessage struct {
	IfID    string `json:"if_id"`
	Role    string `json:"role"`
	Content string `json:"content"`
}

// GetHistory fails closed with ErrLegacyRetired.
func (c *ConversationStore) GetHistory(_ context.Context, _ string, _ int) ([]ConversationMessage, error) {
	return nil, ErrLegacyRetired
}

// ClearHistory fails closed with ErrLegacyRetired.
func (c *ConversationStore) ClearHistory(_ context.Context, _ string) error {
	return ErrLegacyRetired
}

// Prune fails closed with ErrLegacyRetired.
func (c *ConversationStore) Prune(_ context.Context) (int, error) {
	return 0, ErrLegacyRetired
}

// CountMessages fails closed with ErrLegacyRetired.
func (c *ConversationStore) CountMessages(_ context.Context, _ string) (int, error) {
	return 0, ErrLegacyRetired
}

// ResponseCache is the retired LLM response cache (fail-closed no-op).
// Never stores, serves, or shares cached entries across tenants.
type ResponseCache struct{}

// NewResponseCache returns an inert fail-closed ResponseCache without allocating cache storage.
func NewResponseCache(_ time.Duration, _ int) *ResponseCache {
	return &ResponseCache{}
}

// Get always misses (returns empty values and false) because the legacy cache is retired.
func (c *ResponseCache) Get(_, _ string) (answer, model string, ok bool) {
	return "", "", false
}

// Set is a fail-closed no-op so no response is ever cached.
func (c *ResponseCache) Set(_, _, _, _ string) {}

// Invalidate is a fail-closed no-op because no entries are stored.
func (c *ResponseCache) Invalidate(_ string) {}

// Err returns ErrLegacyRetired to signal that ResponseCache is permanently disabled.
func (c *ResponseCache) Err() error {
	return ErrLegacyRetired
}

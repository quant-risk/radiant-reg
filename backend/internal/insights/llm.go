// Package insights — AI Insights LLM service (aposentado na Etapa 0).
//
// Os clientes externos (MiniMaxChat / OpenAIChat) e o orquestrador LLMService
// foram aposentados na Etapa 0 por compliance LC 105 / isolamento multi-tenant
// (ver docs/blueprint/21-ENGINE-IA.md). Todas as chamadas ativas falham
// fechado com ErrLegacyRetired sem efetuar requisições HTTP ou leituras no DB.
package insights

import (
	"context"
	"database/sql"
	"errors"
	"log/slog"
	"time"
)

// ErrLegacyRetired is returned by legacy LLM insights / cache paths retired in Etapa 0.
var ErrLegacyRetired = errors.New("aposentado na Etapa 0 — ver docs/blueprint/21-ENGINE-IA.md")

// ErrInsightsDisabled is kept for compatibility; legacy LLM insights are retired.
var ErrInsightsDisabled = errors.New("insights: tenant has not enabled LLM insights")

// ErrRateLimited is kept for compatibility; legacy LLM insights are retired.
var ErrRateLimited = errors.New("insights: rate limit exceeded (5 req/min)")

// LLMClient is the interface for LLM providers.
type LLMClient interface {
	Chat(ctx context.Context, messages []Message) (string, error)
	StreamChat(ctx context.Context, messages []Message) (<-chan StreamChunk, error)
	Model() string
}

// StreamChunk is a chunk of streamed LLM response.
type StreamChunk struct {
	Text  string
	Done  bool
	Error error
}

// Message is an OpenAI-compatible chat message.
type Message struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// MiniMaxConfig configures the retired MiniMax client.
type MiniMaxConfig struct {
	APIKey  string
	Model   string
	BaseURL string
}

// MiniMaxChat is the retired MiniMax chat client (fail-closed with ErrLegacyRetired).
type MiniMaxChat struct{}

// NewMiniMaxChat returns an inert fail-closed MiniMaxChat without storing credentials or HTTP clients.
func NewMiniMaxChat(_ MiniMaxConfig) *MiniMaxChat {
	return &MiniMaxChat{}
}

// Chat fails closed with ErrLegacyRetired without performing any HTTP request.
func (m *MiniMaxChat) Chat(_ context.Context, _ []Message) (string, error) {
	return "", ErrLegacyRetired
}

// Model returns an empty string for the retired MiniMax client.
func (m *MiniMaxChat) Model() string { return "" }

// StreamChat fails closed with ErrLegacyRetired without performing any HTTP request.
func (m *MiniMaxChat) StreamChat(_ context.Context, _ []Message) (<-chan StreamChunk, error) {
	return nil, ErrLegacyRetired
}

// OpenAIChat is the retired OpenAI-compatible chat client (fail-closed with ErrLegacyRetired).
type OpenAIChat struct {
	APIKey  string
	BaseURL string
}

// NewOpenAIChat returns an inert fail-closed OpenAIChat without storing credentials or HTTP clients.
func NewOpenAIChat(_, _, _ string) *OpenAIChat {
	return &OpenAIChat{}
}

// Chat fails closed with ErrLegacyRetired without performing any HTTP request.
func (o *OpenAIChat) Chat(_ context.Context, _ []Message) (string, error) {
	return "", ErrLegacyRetired
}

// Model returns an empty string for the retired OpenAI client.
func (o *OpenAIChat) Model() string { return "" }

// StreamChat fails closed with ErrLegacyRetired without performing any HTTP request.
func (o *OpenAIChat) StreamChat(_ context.Context, _ []Message) (<-chan StreamChunk, error) {
	return nil, ErrLegacyRetired
}

// LLMConfig configures the retired LLM service.
type LLMConfig struct {
	LLMClient       LLMClient
	DB              *sql.DB
	Logger          *slog.Logger
	RateLimit       int
	RateWindow      time.Duration
	MaxHistoryPairs int
	MaxEvents       int
	ConvStore       *ConversationStore
	RespCache       *ResponseCache
}

// LLMService is the retired AI insights orchestrator (fail-closed with ErrLegacyRetired).
type LLMService struct{}

// NewLLMService returns an inert fail-closed LLMService without retaining DB, client, or cache references.
func NewLLMService(_ LLMConfig) *LLMService {
	return &LLMService{}
}

// Ask fails closed with ErrLegacyRetired.
func (s *LLMService) Ask(_ context.Context, _, _ string) (*LLMAnswer, error) {
	return nil, ErrLegacyRetired
}

// StreamAsk fails closed with ErrLegacyRetired.
func (s *LLMService) StreamAsk(_ context.Context, _, _ string) (<-chan LLMAnswerChunk, error) {
	return nil, ErrLegacyRetired
}

// GetHistory fails closed with ErrLegacyRetired.
func (s *LLMService) GetHistory(_ context.Context, _ string, _ int) ([]ConversationMessage, error) {
	return nil, ErrLegacyRetired
}

// ClearHistory fails closed with ErrLegacyRetired.
func (s *LLMService) ClearHistory(_ context.Context, _ string) error {
	return ErrLegacyRetired
}

// LLMAnswerChunk is a chunk of streamed LLM answer.
type LLMAnswerChunk struct {
	Text  string
	Model string
	Error error
	Done  bool
}

// LLMAnswer is the response from Ask.
type LLMAnswer struct {
	Answer string `json:"answer"`
	Model  string `json:"model"`
	Cached bool   `json:"cached,omitempty"`
}

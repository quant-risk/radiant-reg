package synth

import (
	"context"
	"encoding/json"
	"log/slog"

	"github.com/fortvna/radiant-norma/backend/internal/audit"
)

// Forge é o orchestrator legado do AuditForge (aposentado na Etapa 0 — fail-closed).
type Forge struct {
	cfg Config
}

// Config configura o Forge legado.
type Config struct {
	Cadoc       CadocType
	LLM         LLMClient
	Count       int
	FailureRate float64
	MaxRounds   int
}

// NewForge cria um Forge inerte (aposentado na Etapa 0) sem reter LLMClient nem audit.Service.
func NewForge(cfg Config, _ *audit.Service, _ *slog.Logger) *Forge {
	return &Forge{
		cfg: Config{Cadoc: cfg.Cadoc},
	}
}

// Run falha fechado com ErrLegacyRetired sem invocar LLM ou solvers legados.
func (f *Forge) Run(_ context.Context) ([]Case, error) {
	return nil, ErrLegacyRetired
}

// ExportJSON exporta os casos para JSON.
func ExportJSON(cases []Case) ([]byte, error) {
	return json.MarshalIndent(cases, "", "  ")
}

// ExportFailures exporta apenas os casos que falham regras.
func ExportFailures(cases []Case) []Case {
	var out []Case
	for _, c := range cases {
		if c.IsFailure {
			out = append(out, c)
		}
	}
	return out
}

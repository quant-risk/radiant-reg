package synth

import (
	"context"

	"github.com/fortvna/radiant-norma/backend/internal/audit"
)

// WeakSolver é o solver fraco legado do AuditForge (aposentado na Etapa 0 — fail-closed).
type WeakSolver struct{}

// NewWeakSolver cria um WeakSolver inerte (aposentado na Etapa 0).
func NewWeakSolver(_ *audit.Service) *WeakSolver {
	return &WeakSolver{}
}

// Solve falha fechado com ErrLegacyRetired.
func (w *WeakSolver) Solve(_ context.Context, _ *Case) (float64, error) {
	return 0, ErrLegacyRetired
}

// StrongSolver é o solver forte legado do AuditForge (aposentado na Etapa 0 — fail-closed).
type StrongSolver struct{}

// NewStrongSolver cria um StrongSolver inerte (aposentado na Etapa 0).
func NewStrongSolver() *StrongSolver {
	return &StrongSolver{}
}

// Solve falha fechado com ErrLegacyRetired (o stub antigo retornava 1.0 para qualquer XML não-vazio).
func (s *StrongSolver) Solve(_ context.Context, _ *Case) (float64, error) {
	return 0, ErrLegacyRetired
}

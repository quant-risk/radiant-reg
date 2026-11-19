package synth

import "context"

// JudgeResult é o veredito do Judge sobre um caso.
type JudgeResult struct {
	Accept      bool
	Realism     string
	Difficulty  string
	WeakScore   float64
	StrongScore float64
	Gap         float64
	Feedback    string
}

// Judge é o avaliador legado do AuditForge (aposentado na Etapa 0 — fail-closed).
type Judge struct{}

// NewJudge cria um Judge inerte sem reter LLMClient.
func NewJudge(_ LLMClient) *Judge {
	return &Judge{}
}

// JudgeCase falha fechado com ErrLegacyRetired sem invocar LLM nem aceitar casos por default.
func (j *Judge) JudgeCase(_ context.Context, _ *Case) (*JudgeResult, error) {
	return nil, ErrLegacyRetired
}

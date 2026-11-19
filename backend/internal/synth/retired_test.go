package synth

import (
	"context"
	"errors"
	"log/slog"
	"testing"

	"github.com/fortvna/radiant-norma/backend/internal/insights"
)

type spySynthLLM struct {
	called bool
}

func (s *spySynthLLM) Chat(_ context.Context, _ []Message) (string, error) {
	s.called = true
	return "<Doc3040/>", nil
}

func (s *spySynthLLM) StreamChat(_ context.Context, _ []Message) (<-chan insights.StreamChunk, error) {
	s.called = true
	return nil, nil
}

func (s *spySynthLLM) Model() string { return "spy" }

func TestSynthComponents_FailClosedWithErrLegacyRetired(t *testing.T) {
	ctx := context.Background()
	spy := &spySynthLLM{}

	forge := NewForge(Config{
		Cadoc:     Cadoc3040,
		LLM:       spy,
		Count:     3,
		MaxRounds: 2,
	}, nil, slog.Default())
	if cases, err := forge.Run(ctx); !errors.Is(err, ErrLegacyRetired) || !errors.Is(err, insights.ErrLegacyRetired) || cases != nil {
		t.Fatalf("Forge.Run = (%v, %v), want (nil, ErrLegacyRetired)", cases, err)
	}

	gen := NewGenerator(GeneratorConfig{Cadoc: Cadoc3040, LLM: spy, Count: 2})
	if cases, err := gen.Generate(ctx); !errors.Is(err, ErrLegacyRetired) || cases != nil {
		t.Fatalf("Generator.Generate = (%v, %v), want (nil, ErrLegacyRetired)", cases, err)
	}

	strong := NewStrongSolver()
	if score, err := strong.Solve(ctx, &Case{Cadoc: "3040", XML: "<Doc3040/>"}); !errors.Is(err, ErrLegacyRetired) || score != 0 {
		t.Fatalf("StrongSolver.Solve = (%v, %v), want (0, ErrLegacyRetired)", score, err)
	}

	weak := NewWeakSolver(nil)
	if score, err := weak.Solve(ctx, &Case{Cadoc: "3040", XML: "<Doc3040/>"}); !errors.Is(err, ErrLegacyRetired) || score != 0 {
		t.Fatalf("WeakSolver.Solve = (%v, %v), want (0, ErrLegacyRetired)", score, err)
	}

	judge := NewJudge(spy)
	if res, err := judge.JudgeCase(ctx, &Case{Cadoc: "3040", XML: "<Doc3040/>", IsFailure: true}); !errors.Is(err, ErrLegacyRetired) || res != nil {
		t.Fatalf("Judge.JudgeCase = (%v, %v), want (nil, ErrLegacyRetired)", res, err)
	}

	// Nil receivers must also fail closed safely.
	var nilForge *Forge
	if _, err := nilForge.Run(ctx); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("nil Forge.Run err = %v, want ErrLegacyRetired", err)
	}

	if spy.called {
		t.Fatal("retired synth components must never invoke LLM")
	}
}

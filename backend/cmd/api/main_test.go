package main

import "testing"

func TestInitInsightsLLM_AlwaysNil(t *testing.T) {
	t.Setenv("LLM_API_KEY", "sk-test-should-be-ignored")
	t.Setenv("LLM_PROVIDER", "openai")
	if got := initInsightsLLM(); got != nil {
		t.Fatalf("initInsightsLLM() = %v, want nil (retired in Etapa 0)", got)
	}

	t.Setenv("LLM_PROVIDER", "minimax")
	if got := initInsightsLLM(); got != nil {
		t.Fatalf("initInsightsLLM() with minimax = %v, want nil (retired in Etapa 0)", got)
	}
}

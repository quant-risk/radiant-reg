package main

import (
	"errors"
	"testing"

	"github.com/fortvna/radiant-norma/backend/internal/insights"
	"github.com/fortvna/radiant-norma/backend/internal/synth"
)

func TestSynthGen_RunFailsClosedWithErrLegacyRetired(t *testing.T) {
	err := run()
	if !errors.Is(err, synth.ErrLegacyRetired) {
		t.Fatalf("run() err = %v, want synth.ErrLegacyRetired", err)
	}
	if !errors.Is(err, insights.ErrLegacyRetired) {
		t.Fatalf("run() err = %v, want insights.ErrLegacyRetired", err)
	}
}

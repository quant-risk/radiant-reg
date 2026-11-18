// Testes mínimos do golden-export: rodam a extração em um tempdir e
// validam que os fixtures são JSON parseável e que os counts são > 0.
package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestExportProducesValidJSON(t *testing.T) {
	out := t.TempDir()
	if err := run(out, "../../internal/audit/rules"); err != nil {
		t.Fatalf("run: %v", err)
	}

	// inventory.json — JSON válido com counts > 0 e todas as classes presentes.
	raw, err := os.ReadFile(filepath.Join(out, "inventory.json"))
	if err != nil {
		t.Fatalf("ler inventory.json: %v", err)
	}
	var inv Inventory
	if err := json.Unmarshal(raw, &inv); err != nil {
		t.Fatalf("inventory.json não é JSON válido: %v", err)
	}
	if inv.Counts.Total == 0 || len(inv.Rules) != inv.Counts.Total {
		t.Fatalf("counts.Total=%d, len(rules)=%d (esperado >0 e iguais)", inv.Counts.Total, len(inv.Rules))
	}
	if inv.Counts.Wired == 0 {
		t.Fatal("counts.Wired deveria ser > 0 (regras 3040/2062 são despachadas por applyRegra)")
	}
	if inv.Counts.Orphan == 0 {
		t.Fatal("counts.Orphan deveria ser > 0 (3050/2070/3044/DRM etc. nunca são despachadas)")
	}
	if inv.Counts.Stub == 0 {
		t.Fatal("counts.Stub deveria ser > 0 (stubs severity-I return nil)")
	}
	if inv.SourceCommit == "" {
		t.Fatal("source_commit vazio")
	}
	classes := map[string]bool{}
	for _, r := range inv.Rules {
		switch r.Class {
		case "wired", "orphan", "stub":
			classes[r.Class] = true
		default:
			t.Fatalf("classe inválida %q na regra %s", r.Class, r.Code)
		}
	}
	if len(classes) != 3 {
		t.Fatalf("esperava as 3 classes presentes, got %v", classes)
	}

	// fixtures_3040.json — deve existir com ≥1 caso e XML parseável-JSON.
	raw3040, err := os.ReadFile(filepath.Join(out, "fixtures_3040.json"))
	if err != nil {
		t.Fatalf("ler fixtures_3040.json: %v", err)
	}
	var f3040 FixturesFile
	if err := json.Unmarshal(raw3040, &f3040); err != nil {
		t.Fatalf("fixtures_3040.json não é JSON válido: %v", err)
	}
	if f3040.Counts.Cases == 0 || len(f3040.Cases) != f3040.Counts.Cases {
		t.Fatalf("counts.Cases=%d, len(cases)=%d", f3040.Counts.Cases, len(f3040.Cases))
	}
	for _, c := range f3040.Cases {
		if !strings.Contains(c.XML, "<") {
			t.Fatalf("caso %s: xml sem tag", c.ID)
		}
		if c.Expect != "nil" && c.Expect != "substring" {
			t.Fatalf("caso %s: expect inválido %q", c.ID, c.Expect)
		}
	}

	// not_extracted.json — válido; com a extração AST atual a maioria dos
	// tests (helpers programáticos) não é extraível, então deve ser > 0.
	rawNE, err := os.ReadFile(filepath.Join(out, "not_extracted.json"))
	if err != nil {
		t.Fatalf("ler not_extracted.json: %v", err)
	}
	var ne NotExtractedManifest
	if err := json.Unmarshal(rawNE, &ne); err != nil {
		t.Fatalf("not_extracted.json não é JSON válido: %v", err)
	}
	if len(ne.NotExtracted) == 0 {
		t.Fatal("not_extracted deveria listar casos não extraíveis")
	}
	if !strings.Contains(ne.ExtractionCoverage, "de") {
		t.Fatalf("extraction_coverage malformado: %q", ne.ExtractionCoverage)
	}
}

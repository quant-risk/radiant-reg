// cmd/synth-gen: CLI legado do AuditForge (aposentado na Etapa 0).
//
// Falha fechado com synth.ErrLegacyRetired ("aposentado na Etapa 0 — ver docs/blueprint/21-ENGINE-IA.md")
// para impedir chamadas HTTP externas ou geração sintética fora da nova arquitetura.
package main

import (
	"flag"
	"log/slog"
	"os"

	"github.com/fortvna/radiant-norma/backend/internal/synth"
)

func run() error {
	return synth.ErrLegacyRetired
}

func main() {
	_ = flag.String("cadoc", "3040", "tipo de CADOC (aposentado)")
	_ = flag.Int("count", 50, "número de casos a gerar (aposentado)")
	_ = flag.Int("max-rounds", 5, "máximo de iterações por caso (aposentado)")
	_ = flag.String("output-dir", "./synth-out", "diretório de output (aposentado)")
	_ = flag.String("db", "radiant.db", "path para o banco SQLite (aposentado)")
	flag.Parse()

	logger := slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelInfo}))
	if err := run(); err != nil {
		logger.Error("synth-gen aposentado", "err", err)
		os.Exit(1)
	}
}

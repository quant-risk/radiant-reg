// Package synth — AuditForge legado (aposentado na Etapa 0).
//
// Os pipelines LLM de geração/avaliação sintética (Forge, Generator,
// WeakSolver, StrongSolver, Judge) foram aposentados na Etapa 0 por
// compliance LC 105 / governança de engine IA (ver docs/blueprint/21-ENGINE-IA.md)
// e falham fechado com ErrLegacyRetired.
package synth

import (
	"context"

	"github.com/fortvna/radiant-norma/backend/internal/insights"
)

// ErrLegacyRetired is returned by all legacy synth/AuditForge execution paths retired in Etapa 0.
var ErrLegacyRetired = insights.ErrLegacyRetired

// CadocType é o tipo de documento CADOC.
type CadocType string

const (
	Cadoc2030 CadocType = "2030"
	Cadoc2060 CadocType = "2060"
	Cadoc2061 CadocType = "2061"
	Cadoc2062 CadocType = "2062"
	Cadoc2070 CadocType = "2070"
	Cadoc2160 CadocType = "2160"
	Cadoc2170 CadocType = "2170"
	Cadoc3040 CadocType = "3040"
	Cadoc3050 CadocType = "3050"
	Cadoc4111 CadocType = "4111"
)

// AllCadocTypes retorna a lista de todos os CADOCs suportados.
func AllCadocTypes() []CadocType {
	return []CadocType{
		Cadoc2030, Cadoc2060, Cadoc2061, Cadoc2062, Cadoc2070,
		Cadoc2160, Cadoc2170, Cadoc3040, Cadoc3050, Cadoc4111,
	}
}

// ValidCadocType verifica se o CadocType é suportado pelo synth-gen.
func ValidCadocType(c CadocType) bool {
	for _, v := range AllCadocTypes() {
		if v == c {
			return true
		}
	}
	return false
}

// LLMClient re-exporta a interface do package insights para convenience.
type LLMClient = insights.LLMClient

// Message re-exporta insights.Message.
type Message = insights.Message

// GeneratorConfig configura o gerador legado aposentado.
type GeneratorConfig struct {
	Cadoc       CadocType
	LLM         LLMClient
	Count       int
	FailureRate float64
	MaxRounds   int
}

// Case representa um caso sintético.
type Case struct {
	ID            string  `json:"id"`
	Cadoc         string  `json:"cadoc"`
	XML           string  `json:"xml"`
	RuleCode      string  `json:"rule_code,omitempty"`
	ErrorMsg      string  `json:"error_msg,omitempty"`
	Realism       string  `json:"realism,omitempty"`
	Difficulty    string  `json:"difficulty,omitempty"`
	IsFailure     bool    `json:"is_failure"`
	GeneratedBy   string  `json:"generated_by"`
	JudgedBy      string  `json:"judged_by,omitempty"`
	Rounds        int     `json:"rounds"`
	WeakScore     float64 `json:"weak_score,omitempty"`
	StrongScore   float64 `json:"strong_score,omitempty"`
	JudgeFeedback string  `json:"judge_feedback,omitempty"`
}

// Generator é o gerador sintético legado (fail-closed com ErrLegacyRetired).
type Generator struct {
	cfg GeneratorConfig
}

// NewGenerator cria um Generator inerte (aposentado na Etapa 0) sem reter LLMClient.
func NewGenerator(cfg GeneratorConfig) *Generator {
	return &Generator{cfg: GeneratorConfig{Cadoc: cfg.Cadoc}}
}

// promptForCadoc retorna o prompt de grounding para o tipo de CADOC.
func promptForCadoc(cadoc CadocType) string {
	switch cadoc {
	case Cadoc3040:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 3040 (SCR — Risco de Crédito).
O XML deve seguir esta estrutura EXATA:
<Doc3040 dataBase="YYYY-MM" cnpj="XXXXXXXX" remessa="N" parte="N" tpArq="F" nomeResp="..." emailResp="..." telResp="..." totalCli="N">
  <Agreg natuOp="01" mod="1000" origemRec="1" vincME="N" classOp="A" faixaVlr="1" przProvm="N" localiz="SP" tpCli="1" desempOp="01" provConsttd="0" qtdOp="0" qtdCli="0">
    <Venc v110="0.00" v120="0.00" v150="0.00" v160="0.00" v165="0.00"/>
  </Agreg>
</Doc3040>

Regras que devem ser testadas:
- B06: remessa deve ser >= 1
- B07: parte deve ser >= 1
- C01-C05: campos obrigatórios condicionais
- S01-S05: regras semânticas
- ClassOp: deve ser A, B, C, D, E, F, G ou H
- Vencimentos: valores monetários N15,2 (até 13 dígitos + 2 decimais)
- tpArq: deve ser F (full) ou S (substituição)

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc3050:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gera um XML válido para o documento CADOC 3050 (TXB — Taxa de Câmbio).
Estrutura: <Doc3050 dataBase="YYYY-MM" cnpj="XXXXXXXX" remessa="N" parte="N" tpArq="F">...</Doc3050>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc4111:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 4111 (COSIF — Plano Contábil).
Estrutura: <Documento4111 dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <Cliente qtdCli="N" cnpj="XXXXXXXX">
    <Modalidade codigo="XXXX" indicacao="S|N"/>
  </Cliente>
  ...
</Documento4111>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2030:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2030 (DRSAC — Risco Socioambiental / ESG).
Estrutura: <DocDRSAC dataBase="YYYY-MM" cnpj="XXXXXXXX" subsegmento="S1|S2|S3|S4|S5">
  <Concentracao cnpj="XXXXXXXX" faixaRisco="1|2|3|4|5" valor="N" participacao="0.00"/>
  ...
</DocDRSAC>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2060:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2060 (DRM — Risco de Mercado).
Estrutura: <DocDRM dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <Posicao moeda="BRL|USD|EUR" valor="N" var="N" svar="N" rwacom="0.00"/>
  ...
</DocDRM>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2061:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2061 (DLO — Limites Operacionais).
Estrutura: <DocDLO dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <Conta codigo="XXXX" elem="X" rwacam="0.00" patrimonial="0.00"/>
  ...
</DocDLO>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2062:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2062 (DLI — Limites Individuais).
Estrutura: <DocDLI dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <Limite tipo="XXXX" valor="N" parametro="0.0000"/>
  ...
</DocDLI>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2070:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2070 (DDR — Requerimento de Capital Diário).
Estrutura: <DocDDR dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <Posicao codigo="XXXX" moeda="BRL|USD" valorExposicao="0.00" requerimento="0.00"/>
  ...
</DocDDR>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2160:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2160 (DRL — Liquidez / LCR).
Estrutura: <DocDRL dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <HQLA nivel="1|2A|2B" valor="0.00"/>
  <Outflows categoria="retail|wholesale" valor="0.00"/>
  <Inflows categoria="operational|nonoperational" valor="0.00"/>
  ...
</DocDRL>

Gere APENAS o XML, sem markdown, sem explicação.`
	case Cadoc2170:
		return `Você é um gerador de dados sintéticos para validação de regras regulatórias bancárias brasileiras (BACEN).
Gere um XML válido para o documento CADOC 2170 (DLP — Liquidez de Longo Prazo / NSFR).
Estrutura: <DocDLP dataBase="YYYY-MM" cnpj="XXXXXXXX">
  <ASF categoria="equity|stable|less_stable" valor="0.00"/>
  <RSF categoria="unencumbered|secured|unsecured" valor="0.00"/>
  ...
</DocDLP>

Gere APENAS o XML, sem markdown, sem explicação.`
	default:
		return "Gere XML para o documento " + string(cadoc) + ". APENAS o XML, sem markdown."
	}
}

// Generate falha fechado com ErrLegacyRetired sem invocar qualquer LLM.
func (g *Generator) Generate(_ context.Context) ([]Case, error) {
	return nil, ErrLegacyRetired
}

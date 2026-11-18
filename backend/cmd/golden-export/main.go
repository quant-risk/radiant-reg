// Command golden-export extrai o golden corpus do port Go→Rust do motor
// Radiant Norma para fixtures JSON consumíveis pelos testes Rust
// (rust/crates/reg-cadoc/tests/golden/).
//
// Ferramenta TEMPORÁRIA e honesta: os números de cobertura declarados nos
// JSONs refletem exatamente o que foi extraído mecanicamente (go/ast +
// go/parser), sem invenção. Casos não extraíveis (tables construídas com
// helpers programáticos) vão para o manifest "not_extracted" com motivo.
//
// Uso (a partir de backend/):
//
//	go run ./cmd/golden-export -out ../../rust/crates/reg-cadoc/tests/golden/
//
// Saídas:
//   - inventory.json       — todas as regras do registry com classificação
//     honesta: "wired" (despachada por applyRegra em service.go para
//     3040/2062), "orphan" (implementada mas nunca despachada) ou "stub"
//     (corpo da função é pass-through return nil).
//   - fixtures_<cadoc>.json — casos de teste com XML literal em backticks,
//     a regra aplicada e o expected (nil ou substring do erro).
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/fortvna/radiant-norma/backend/internal/audit/rules"
)

// ============================================================
// Modelos de saída
// ============================================================

type RuleEntry struct {
	Code     string `json:"code"`
	Cadoc    string `json:"cadoc"`
	Sheet    string `json:"sheet"`
	Severity string `json:"severity"`
	Class    string `json:"class"` // wired | orphan | stub
}

type InventoryCounts struct {
	Wired  int `json:"wired"`
	Orphan int `json:"orphan"`
	Stub   int `json:"stub"`
	Total  int `json:"total"`
}

type Inventory struct {
	Rules        []RuleEntry     `json:"rules"`
	Counts       InventoryCounts `json:"counts"`
	GeneratedAt  string          `json:"generated_at"`
	SourceCommit string          `json:"source_commit"`
}

type FixtureCase struct {
	ID        string `json:"id"`
	TestName  string `json:"test_name"`
	RuleCode  string `json:"rule_code"`
	XML       string `json:"xml"`
	Expect    string `json:"expect"` // "nil" | "substring"
	Substring string `json:"substring"`
	Note      string `json:"note,omitempty"`
}

type FixtureCounts struct {
	Cases int `json:"cases"`
	Tests int `json:"tests"` // funções Test com ≥1 caso extraído
}

type FixturesFile struct {
	Cases              []FixtureCase `json:"cases"`
	Counts             FixtureCounts `json:"counts"`
	ExtractionCoverage string        `json:"extraction_coverage"`
	GeneratedAt        string        `json:"generated_at"`
	SourceCommit       string        `json:"source_commit"`
}

type NotExtracted struct {
	TestName string `json:"test_name"`
	File     string `json:"file"`
	Reason   string `json:"reason"`
}

type NotExtractedManifest struct {
	NotExtracted       []NotExtracted `json:"not_extracted"`
	ExtractionCoverage string         `json:"extraction_coverage"`
	GeneratedAt        string         `json:"generated_at"`
	SourceCommit       string         `json:"source_commit"`
}

// ============================================================
// main
// ============================================================

func main() {
	outDir := flag.String("out", "../../rust/crates/reg-cadoc/tests/golden/", "diretório de saída dos fixtures")
	rulesDir := flag.String("rules", "internal/audit/rules", "diretório das regras (fonte para AST)")
	flag.Parse()

	resolved := *outDir
	// Guarda de segurança: backend/ fica um nível abaixo da raiz do repo, e o
	// repo tem rust/ na raiz. O caminho "../../rust/..." (default e exemplos
	// de uso) resolve para FORA do repo a partir de backend/. Se o destino
	// absoluto cair fora do repo e o equivalente "../rust/..." cair dentro,
	// remapeia para não vazar fixtures para fora do repositório.
	if abs, err := filepath.Abs(resolved); err == nil {
		repoRoot := findRepoRoot()
		if repoRoot != "" && !strings.HasPrefix(abs, repoRoot+string(filepath.Separator)) {
			if strings.HasPrefix(resolved, "../../rust/") {
				alt := "../" + strings.TrimPrefix(resolved, "../../")
				if altAbs, err2 := filepath.Abs(alt); err2 == nil && strings.HasPrefix(altAbs, repoRoot+string(filepath.Separator)) {
					fmt.Fprintf(os.Stderr, "golden-export: aviso: %s cai fora do repo; usando %s\n", resolved, alt)
					resolved = alt
				}
			}
		}
	}

	if err := run(resolved, *rulesDir); err != nil {
		fmt.Fprintf(os.Stderr, "golden-export: %v\n", err)
		os.Exit(1)
	}
}

// findRepoRoot sobe a partir do cwd até achar o go.mod (raiz do módulo,
// backend/) e retorna o diretório pai — a raiz do repositório.
func findRepoRoot() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, "go.mod")); err == nil {
			parent := filepath.Dir(dir)
			if filepath.Base(dir) == "backend" {
				return parent
			}
			return dir
		}
		up := filepath.Dir(dir)
		if up == dir {
			return ""
		}
		dir = up
	}
}

func dirExists(p string) bool {
	fi, err := os.Stat(p)
	return err == nil && fi.IsDir()
}

func run(outDir, rulesDir string) error {
	commit := gitHead()
	now := time.Now().UTC().Format(time.RFC3339)

	if err := os.MkdirAll(outDir, 0o755); err != nil {
		return fmt.Errorf("criar %s: %w", outDir, err)
	}

	// ---------- PARTE 1: inventário ----------
	stubTypes := detectStubBodies(rulesDir)
	inv := buildInventory(stubTypes)
	inv.GeneratedAt = now
	inv.SourceCommit = commit
	writeJSON(filepath.Join(outDir, "inventory.json"), inv)

	// Mapa typeName → code para a extração de fixtures.
	typeNameToCode := map[string]string{}
	for _, r := range inv.Rules {
		if src := lookupTypeSource(r.Code, r.Cadoc); src != "" {
			typeNameToCode[src] = r.Code
		}
	}

	// ---------- PARTE 2: fixtures ----------
	casesByCadoc, notExtracted, covered, totalTests := extractFixtures(rulesDir, typeNameToCode)
	for cadoc, cases := range casesByCadoc {
		uniqTests := map[string]bool{}
		for _, c := range cases {
			uniqTests[c.TestName] = true
		}
		ff := FixturesFile{
			Cases:              cases,
			Counts:             FixtureCounts{Cases: len(cases), Tests: len(uniqTests)},
			ExtractionCoverage: fmt.Sprintf("%d de %d funções Test cobertas", len(uniqTests), totalTests),
			GeneratedAt:        now,
			SourceCommit:       commit,
		}
		writeJSON(filepath.Join(outDir, fmt.Sprintf("fixtures_%s.json", cadoc)), ff)
	}

	man := NotExtractedManifest{
		NotExtracted:       notExtracted,
		ExtractionCoverage: fmt.Sprintf("%d de %d funções Test cobertas", covered, totalTests),
		GeneratedAt:        now,
		SourceCommit:       commit,
	}
	writeJSON(filepath.Join(outDir, "not_extracted.json"), man)

	fmt.Printf("inventory: wired=%d orphan=%d stub=%d total=%d\n",
		inv.Counts.Wired, inv.Counts.Orphan, inv.Counts.Stub, inv.Counts.Total)
	for _, cadoc := range sortedKeys(casesByCadoc) {
		fmt.Printf("fixtures_%s.json: %d casos\n", cadoc, len(casesByCadoc[cadoc]))
	}
	fmt.Printf("extracao: %d de %d funcoes Test cobertas; %d nao extraidas (ver not_extracted.json)\n",
		covered, totalTests, len(notExtracted))
	return nil
}

func gitHead() string {
	out, err := exec.Command("git", "rev-parse", "HEAD").Output()
	if err != nil {
		return "unknown"
	}
	return strings.TrimSpace(string(out))
}

func writeJSON(path string, v any) {
	b, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		fatalf("marshal %s: %v", path, err)
	}
	b = append(b, '\n')
	if err := os.WriteFile(path, b, 0o644); err != nil {
		fatalf("escrever %s: %v", path, err)
	}
}

func fatalf(f string, a ...any) {
	fmt.Fprintf(os.Stderr, "golden-export: "+f+"\n", a...)
	os.Exit(1)
}

func sortedKeys(m map[string][]FixtureCase) []string {
	ks := make([]string, 0, len(m))
	for k := range m {
		ks = append(ks, k)
	}
	sort.Strings(ks)
	return ks
}

// ============================================================
// PARTE 1 — inventário com classificação honesta
// ============================================================
//
// Fonte da classificação "wired": o dispatch real em
// internal/audit/service.go::applyRegra, que só consulta:
//   - registry.GetRaw / registry.Get      → CADOC 3040
//   - registry.Get2062                    → CADOC 2062 (DLI)
// Todo o resto (3050, 2070, 2061/DLO, 2160/DRL, 2170/DLP, 3044, 2060/DRM)
// está implementado+registrado, mas NUNCA é despachado → "orphan".

func buildInventory(stubTypes map[string]bool) Inventory {
	var entries []RuleEntry

	add := func(v any, cadoc, class string, hasSheet bool) {
		code := codeOf(v)
		sheet := ""
		if hasSheet {
			sheet = sheetOf(v)
		}
		cls := class
		tn := reflect.TypeOf(v).Name()
		if stubTypes[tn] {
			cls = "stub" // stub é propriedade do corpo, sobrepõe wiring
		}
		entries = append(entries, RuleEntry{
			Code:     code,
			Cadoc:    cadoc,
			Sheet:    sheet,
			Severity: severityOf(v),
			Class:    cls,
		})
	}

	b := rules.Builtin3040()

	// wired: despachadas por applyRegra
	for _, r := range b.AllRaw() {
		add(r, "3040", "wired", true)
	}
	for _, r := range b.All() {
		add(r, "3040", "wired", true)
	}
	for _, r := range b.All2062() {
		add(r, "2062", "wired", true)
	}

	// orphan: implementadas+testadas, nunca despachadas por applyRegra
	for _, r := range rules.Builtin3050().All3050() {
		add(r, "3050", "orphan", true)
	}
	for _, r := range rules.Builtin2070().All2070() {
		add(r, "2070", "orphan", true)
	}
	for _, r := range b.All2061() {
		add(r, "2061", "orphan", true)
	}
	for _, r := range b.All2160() {
		add(r, "2160", "orphan", true)
	}
	for _, r := range b.All2170() {
		add(r, "2170", "orphan", true)
	}
	for _, r := range b.All3044() {
		add(r, "3044", "orphan", false) // Rule3044 não expõe Sheet()
	}
	for _, r := range b.AllDRM() {
		add(r, "2060", "orphan", true)
	}

	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Cadoc != entries[j].Cadoc {
			return entries[i].Cadoc < entries[j].Cadoc
		}
		return entries[i].Code < entries[j].Code
	})

	var c InventoryCounts
	c.Total = len(entries)
	for _, e := range entries {
		switch e.Class {
		case "wired":
			c.Wired++
		case "orphan":
			c.Orphan++
		case "stub":
			c.Stub++
		}
	}
	return Inventory{Rules: entries, Counts: c}
}

// Os wrappers abaixo usam uma interface mínima para extrair metadata sem
// depender do tipo concreto de cada CADOC.

type coded interface{ Code() string }
type severity interface{ Severity() string }
type sheet interface{ Sheet() string }

func codeOf(v any) string     { return v.(coded).Code() }
func severityOf(v any) string { return v.(severity).Severity() }
func sheetOf(v any) string    { return v.(sheet).Sheet() }

// lookupTypeSource retorna o nome do tipo Go de uma regra a partir do code.
// Faz uma busca pontual nos registries (barato e sem duplicar a lista).
func lookupTypeSource(code, cadoc string) string {
	b := rules.Builtin3040()
	var found any
	switch cadoc {
	case "3040":
		if r := b.Get(code); r != nil {
			found = r
		} else if r := b.GetRaw(code); r != nil {
			found = r
		}
	case "2062":
		found = b.Get2062(code)
	case "3050":
		found = rules.Builtin3050().Get3050(code)
	case "2070":
		found = rules.Builtin2070().Get2070(code)
	case "2061":
		found = b.Get2061(code)
	case "2160":
		found = b.Get2160(code)
	case "2170":
		found = b.Get2170(code)
	case "3044":
		found = b.Get3044(code)
	case "2060":
		found = b.GetDRM(code)
	}
	if found == nil {
		return ""
	}
	return reflect.TypeOf(found).Name()
}

// detectStubBodies faz parse AST do diretório de regras e retorna o conjunto
// de nomes de tipos cujo método Apply*/Apply3050/Apply2070/ApplyRaw é um
// pass-through (corpo só tem `return nil`, assigns em branco `_ = x` e/ou
// comentários).
func detectStubBodies(rulesDir string) map[string]bool {
	stubs := map[string]bool{}
	fset := token.NewFileSet()
	pkgs, err := parser.ParseDir(fset, rulesDir, func(fi os.FileInfo) bool {
		return !strings.HasSuffix(fi.Name(), "_test.go")
	}, 0)
	if err != nil {
		fatalf("parse %s: %v", rulesDir, err)
	}
	for _, pkg := range pkgs {
		for _, f := range pkg.Files {
			for _, decl := range f.Decls {
				fd, ok := decl.(*ast.FuncDecl)
				if !ok || fd.Recv == nil || len(fd.Recv.List) == 0 {
					continue
				}
				if !strings.HasPrefix(fd.Name.Name, "Apply") || fd.Body == nil {
					continue
				}
				recv := recvTypeName(fd.Recv.List[0].Type)
				if recv == "" {
					continue
				}
				if isPassThroughBody(fd.Body) {
					stubs[recv] = true
				} else {
					// Se algum método real existe (ex: destravada), não é stub.
					stubs[recv] = false
				}
			}
		}
	}
	// Remove entradas falsas (false marca "tem corpo real").
	out := map[string]bool{}
	for k, v := range stubs {
		if v {
			out[k] = true
		}
	}
	return out
}

func recvTypeName(expr ast.Expr) string {
	switch t := expr.(type) {
	case *ast.Ident:
		return t.Name
	case *ast.StarExpr:
		return recvTypeName(t.X)
	}
	return ""
}

func isPassThroughBody(body *ast.BlockStmt) bool {
	if len(body.List) == 0 {
		return true // corpo vazio retorna nil implicitamente
	}
	for _, s := range body.List {
		switch st := s.(type) {
		case *ast.ReturnStmt:
			if len(st.Results) != 1 || !isNilIdent(st.Results[0]) {
				return false
			}
		case *ast.AssignStmt:
			// só permite `_ = x` / `_, y = ...` (descarte explícito)
			for _, lhs := range st.Lhs {
				id, ok := lhs.(*ast.Ident)
				if !ok || id.Name != "_" {
					return false
				}
			}
		case *ast.EmptyStmt:
			// ok
		default:
			return false
		}
	}
	return true
}

func isNilIdent(e ast.Expr) bool {
	id, ok := e.(*ast.Ident)
	return ok && id.Name == "nil"
}

// ============================================================
// PARTE 2 — fixtures de XML literal (go/ast)
// ============================================================

func extractFixtures(rulesDir string, typeNameToCode map[string]string) (
	map[string][]FixtureCase, []NotExtracted, int, int) {

	fset := token.NewFileSet()
	pkgs, err := parser.ParseDir(fset, rulesDir, func(fi os.FileInfo) bool {
		return strings.HasSuffix(fi.Name(), "_test.go")
	}, parser.ParseComments)
	if err != nil {
		fatalf("parse %s: %v", rulesDir, err)
	}

	casesByCadoc := map[string][]FixtureCase{}
	var notExtracted []NotExtracted
	covered, totalTests := 0, 0

	// Ordena arquivos/declarações para saída determinística.
	var files []*ast.File
	for _, pkg := range pkgs {
		for _, f := range pkg.Files {
			files = append(files, f)
		}
	}
	sort.Slice(files, func(i, j int) bool {
		return fset.Position(files[i].Pos()).Filename < fset.Position(files[j].Pos()).Filename
	})

	for _, f := range files {
		fname := filepath.Base(fset.Position(f.Pos()).Filename)
		for _, decl := range f.Decls {
			fd, ok := decl.(*ast.FuncDecl)
			if !ok || !strings.HasPrefix(fd.Name.Name, "Test") {
				continue
			}
			totalTests++
			testName := fd.Name.Name

			ruleTypes := collectRuleTypes(fd, typeNameToCode)
			xmls := collectXMLLiterals(fd)

			switch {
			case len(xmls) > 0:
				covered++
				cadocCounts := map[string]int{}
				for i, xml := range xmls {
					code := ""
					if i < len(ruleTypes) {
						code = ruleTypes[i]
					} else if len(ruleTypes) > 0 {
						code = ruleTypes[0]
					}
					cadoc := cadocOfCode(code)
					if cadoc == "" {
						cadoc = cadocOfTestFile(fname)
					}
					cadocCounts[cadoc]++
					expect, substr, note := expectationOf(fd)
					id := fmt.Sprintf("%s-%s-%02d", cadoc, testName, cadocCounts[cadoc])
					casesByCadoc[cadoc] = append(casesByCadoc[cadoc], FixtureCase{
						ID:        id,
						TestName:  testName,
						RuleCode:  code,
						XML:       xml,
						Expect:    expect,
						Substring: substr,
						Note:      note,
					})
				}
			case len(ruleTypes) > 0:
				notExtracted = append(notExtracted, NotExtracted{
					TestName: testName,
					File:     fname,
					Reason:   "inputs construidos programaticamente (structs/helpers tipo docValidoBaseOp), sem XML literal no corpo do teste",
				})
			default:
				notExtracted = append(notExtracted, NotExtracted{
					TestName: testName,
					File:     fname,
					Reason:   "teste de infraestrutura (registry/parser/leiaute) sem caso isolado de regra com XML literal",
				})
			}
		}
	}
	return casesByCadoc, notExtracted, covered, totalTests
}

// collectRuleTypes retorna os codes das regras instanciadas no corpo
// (CompositeLit `rules.X{}` / `X{}` ou declaração `var r rules.X`), na
// ordem de aparição.
func collectRuleTypes(fd *ast.FuncDecl, typeNameToCode map[string]string) []string {
	var out []string
	seen := map[string]bool{}
	add := func(name string) {
		if code, ok := typeNameToCode[name]; ok && !seen[code] {
			seen[code] = true
			out = append(out, code)
		}
	}
	ast.Inspect(fd.Body, func(n ast.Node) bool {
		switch t := n.(type) {
		case *ast.CompositeLit:
			switch ct := t.Type.(type) {
			case *ast.Ident:
				add(ct.Name)
			case *ast.SelectorExpr:
				add(ct.Sel.Name)
			}
		case *ast.SelectorExpr:
			// pega `var r rules.B01ArquivoXMLValido` (tipo sem composite literal)
			add(t.Sel.Name)
		}
		return true
	})
	return out
}

// collectXMLLiterals extrai strings literais (backtick e interpretadas) que
// parecem XML: começam com "<" e contêm ">".
func collectXMLLiterals(fd *ast.FuncDecl) []string {
	var out []string
	seen := map[string]bool{}
	ast.Inspect(fd.Body, func(n ast.Node) bool {
		bl, ok := n.(*ast.BasicLit)
		if !ok || bl.Kind != token.STRING {
			return true
		}
		v := unquoteGo(bl.Value)
		trimmed := strings.TrimSpace(v)
		if strings.HasPrefix(trimmed, "<") && strings.Contains(trimmed, ">") {
			if !seen[v] {
				seen[v] = true
				out = append(out, v)
			}
		}
		return true
	})
	return out
}

func unquoteGo(lit string) string {
	if strings.HasPrefix(lit, "`") {
		return strings.Trim(lit, "`")
	}
	s, err := strconv.Unquote(lit)
	if err != nil {
		return lit
	}
	return s
}

// expectationOf determina o expected do teste via AST:
//   - "substring": o teste espera erro (assert err==nil → Fatal) e, quando
//     possível, extrai o literal de strings.Contains(err.Error(), "x").
//   - "nil": o teste espera sucesso.
func expectationOf(fd *ast.FuncDecl) (expect, substring, note string) {
	expectError := false
	expectNil := false
	tableDriven := false
	ast.Inspect(fd.Body, func(n ast.Node) bool {
		if st, ok := n.(*ast.IfStmt); ok {
			bin, ok := st.Cond.(*ast.BinaryExpr)
			if !ok {
				return true
			}
			op := bin.Op.String()
			lhsIsErr := isErrIdent(bin.X)
			rhsIsErr := isErrIdent(bin.Y)
			lhsIsNil := isNilIdent(bin.X)
			rhsIsNil := isNilIdent(bin.Y)
			isErrVsNil := (lhsIsErr && rhsIsNil) || (lhsIsNil && rhsIsErr)
			if !hasTAssert(st.Body) {
				return true
			}
			switch {
			case isErrVsNil && op == "==":
				expectError = true // if err == nil { t.Fatal("deveria detectar erro") }
			case isErrVsNil && op == "!=":
				// `if err != nil { ... }` normalmente espera sucesso, mas só
				// marca se a mensagem indica isso (evita confundir com asserts
				// de setup tipo "ParseDoc3040: %v").
				if branchExpectsNil(st.Body) {
					expectNil = true
				}
			}
		}
		if se, ok := n.(*ast.SelectorExpr); ok && se.Sel.Name == "wantErr" {
			tableDriven = true
		}
		return true
	})

	// substring: literal em Contains(err.Error(), "lit") ou Contains(err.Error(), var-of-literal)
	ast.Inspect(fd.Body, func(n ast.Node) bool {
		if substring != "" {
			return true
		}
		ce, ok := n.(*ast.CallExpr)
		if !ok || len(ce.Args) != 2 {
			return true
		}
		fn, ok := ce.Fun.(*ast.Ident)
		if !ok {
			if s, ok2 := ce.Fun.(*ast.SelectorExpr); ok2 {
				fn = s.Sel
			} else {
				return true
			}
		}
		if fn.Name != "Contains" {
			return true
		}
		if !errDotError(ce.Args[0]) {
			return true
		}
		if bl, ok := ce.Args[1].(*ast.BasicLit); ok && bl.Kind == token.STRING {
			substring = unquoteGo(bl.Value)
		}
		return true
	})

	switch {
	case expectError:
		expect = "substring" // erro esperado; substring vazia se não for literal
	case expectNil:
		expect = "nil"
	case tableDriven:
		// Table-driven com wantErr por linha: expectativa é dinâmica.
		expect = "nil"
		note = "table-driven com wantErr por linha; expectativa variável, expect fixado como nil e substrings extraidas quando literais"
	default:
		expect = "nil"
		note = "sem assert de erro detectado estaticamente"
	}
	return expect, substring, note
}

// branchExpectsNil verifica se a mensagem de t.Fatal/Errorf no branch
// `if err != nil` indica que o teste esperava sucesso (nil).
func branchExpectsNil(body *ast.BlockStmt) bool {
	found := false
	ast.Inspect(body, func(n ast.Node) bool {
		if bl, ok := n.(*ast.BasicLit); ok && bl.Kind == token.STRING {
			msg := strings.ToLower(unquoteGo(bl.Value))
			if strings.Contains(msg, "deveria passar") ||
				strings.Contains(msg, "esperava nil") ||
				strings.Contains(msg, "want nil") ||
				strings.Contains(msg, "não deveria") ||
				strings.Contains(msg, "nao deveria") ||
				strings.Contains(msg, "wanterr=false") {
				found = true
			}
		}
		return true
	})
	return found
}

func hasTAssert(body *ast.BlockStmt) bool {
	found := false
	ast.Inspect(body, func(n ast.Node) bool {
		if ce, ok := n.(*ast.CallExpr); ok {
			if se, ok := ce.Fun.(*ast.SelectorExpr); ok {
				if id, ok := se.X.(*ast.Ident); ok && id.Name == "t" {
					switch se.Sel.Name {
					case "Fatal", "Fatalf", "Error", "Errorf":
						found = true
					}
				}
			}
		}
		return true
	})
	return found
}

func isErrIdent(e ast.Expr) bool {
	id, ok := e.(*ast.Ident)
	return ok && id.Name == "err"
}

func errDotError(e ast.Expr) bool {
	// casa `err.Error()` — a chamada, não o seletor direto
	ce, ok := e.(*ast.CallExpr)
	if !ok {
		return false
	}
	se, ok := ce.Fun.(*ast.SelectorExpr)
	if !ok {
		return false
	}
	return se.Sel.Name == "Error" && isErrIdent(se.X)
}

// cadocOfTestFile deriva o CADOC do nome do arquivo de teste como fallback.
func cadocOfTestFile(fname string) string {
	switch {
	case strings.HasPrefix(fname, "3040"):
		return "3040"
	case strings.HasPrefix(fname, "3050"):
		return "3050"
	case strings.HasPrefix(fname, "2070"):
		return "2070"
	case strings.HasPrefix(fname, "dli"):
		return "2062"
	case strings.HasPrefix(fname, "dlp"):
		return "2170"
	case strings.HasPrefix(fname, "drl"):
		return "2160"
	case strings.HasPrefix(fname, "drm"):
		return "2060"
	case strings.HasPrefix(fname, "rule3044"):
		return "3044"
	case strings.HasPrefix(fname, "raw_rules"):
		return "3040"
	default:
		return "3040"
	}
}

func cadocOfCode(code string) string {
	// Deriva do inventário carregado em memória.
	if c, ok := codeToCadoc[code]; ok {
		return c
	}
	return ""
}

var codeToCadoc = map[string]string{}

func init() {
	inv := buildInventoryQuick()
	for _, e := range inv {
		codeToCadoc[e.code] = e.cadoc
	}
}

type kv struct{ code, cadoc string }

func buildInventoryQuick() []kv {
	var out []kv
	add := func(v any, cadoc string) {
		if c, ok := v.(coded); ok {
			out = append(out, kv{c.Code(), cadoc})
		}
	}
	b := rules.Builtin3040()
	for _, r := range b.AllRaw() {
		add(r, "3040")
	}
	for _, r := range b.All() {
		add(r, "3040")
	}
	for _, r := range b.All2062() {
		add(r, "2062")
	}
	for _, r := range rules.Builtin3050().All3050() {
		add(r, "3050")
	}
	for _, r := range rules.Builtin2070().All2070() {
		add(r, "2070")
	}
	for _, r := range b.All2061() {
		add(r, "2061")
	}
	for _, r := range b.All2160() {
		add(r, "2160")
	}
	for _, r := range b.All2170() {
		add(r, "2170")
	}
	for _, r := range b.All3044() {
		add(r, "3044")
	}
	for _, r := range b.AllDRM() {
		add(r, "2060")
	}
	return out
}

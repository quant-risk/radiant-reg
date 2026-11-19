// Sprint 53 v3.34.35: AI Insights LLM handlers (aposentados na Etapa 0).
//
// Todos os endpoints legados de insights via LLM (/v1/insights/ask,
// /v1/insights/ask/stream, /v1/insights/history) retornam explicitamente
// 410 Gone por compliance LC 105 / isolamento multi-tenant
// (ver docs/blueprint/21-ENGINE-IA.md).
package api

import "net/http"

func writeLegacyInsightsRetired(w http.ResponseWriter) {
	writeJSON(w, http.StatusGone, map[string]string{
		"error":   "legacy_insights_retired",
		"code":    "LEGACY_INSIGHTS_RETIRED",
		"message": "insights LLM legados foram aposentados na Etapa 0 (LC 105 / isolamento multi-tenant — ver docs/blueprint/21-ENGINE-IA.md)",
	})
}

// streamAskLLM handles GET /v1/insights/ask/stream — retired in Etapa 0 (410 Gone).
func (s *Server) streamAskLLM(w http.ResponseWriter, _ *http.Request) {
	writeLegacyInsightsRetired(w)
}

// AskLLM handles POST /v1/insights/ask — retired in Etapa 0 (410 Gone).
func (s *Server) AskLLM(w http.ResponseWriter, _ *http.Request) {
	writeLegacyInsightsRetired(w)
}

// getInsightsHistory handles GET /v1/insights/history — retired in Etapa 0 (410 Gone).
func (s *Server) getInsightsHistory(w http.ResponseWriter, _ *http.Request) {
	writeLegacyInsightsRetired(w)
}

// deleteInsightsHistory handles DELETE /v1/insights/history — retired in Etapa 0 (410 Gone).
func (s *Server) deleteInsightsHistory(w http.ResponseWriter, _ *http.Request) {
	writeLegacyInsightsRetired(w)
}

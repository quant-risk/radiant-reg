package api_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/fortvna/radiant-norma/backend/internal/insights"
)

func TestLegacyInsightsRoutes_Return410Gone(t *testing.T) {
	srv, _ := newTestServer(t)
	handler := srv.Router()

	cases := []struct {
		name   string
		method string
		path   string
		body   any
	}{
		{
			name:   "POST /v1/insights/ask",
			method: http.MethodPost,
			path:   "/v1/insights/ask",
			body:   map[string]string{"question": "qual status?"},
		},
		{
			name:   "GET /v1/insights/ask/stream",
			method: http.MethodGet,
			path:   "/v1/insights/ask/stream?question=qual+status",
		},
		{
			name:   "GET /v1/insights/history",
			method: http.MethodGet,
			path:   "/v1/insights/history",
		},
		{
			name:   "DELETE /v1/insights/history",
			method: http.MethodDelete,
			path:   "/v1/insights/history",
		},
	}

	headers := map[string]string{"X-IF-ID": "demo"}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			w := reqJSON(handler, tc.method, tc.path, tc.body, headers)
			if w.Code != http.StatusGone {
				t.Fatalf("status = %d, want %d (410 Gone), body = %s", w.Code, http.StatusGone, w.Body.String())
			}

			var resp map[string]string
			if err := json.Unmarshal(w.Body.Bytes(), &resp); err != nil {
				t.Fatalf("decode JSON: %v (body=%s)", err, w.Body.String())
			}
			if resp["error"] != "legacy_insights_retired" {
				t.Errorf("error = %q, want legacy_insights_retired", resp["error"])
			}
			if !strings.Contains(resp["message"], "Etapa 0") || !strings.Contains(resp["message"], "LC 105") {
				t.Errorf("message should explain Etapa 0 / LC 105 retirement, got %q", resp["message"])
			}
		})
	}
}

func TestLegacyInsightsRoutes_Return410GoneEvenIfInsightsLLMInjected(t *testing.T) {
	srv, _ := newTestServer(t)
	srv.InsightsLLM = insights.NewLLMService(insights.LLMConfig{})
	handler := srv.Router()

	w := reqJSON(handler, http.MethodPost, "/v1/insights/ask",
		map[string]string{"question": "status"}, map[string]string{"X-IF-ID": "demo"})
	if w.Code != http.StatusGone {
		t.Fatalf("expected 410 Gone even with InsightsLLM non-nil, got %d: %s", w.Code, w.Body.String())
	}

	// Unauthenticated request still fails auth middleware with 401.
	unauth := reqJSON(handler, http.MethodPost, "/v1/insights/ask",
		map[string]string{"question": "status"}, nil)
	if unauth.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 without auth, got %d", unauth.Code)
	}

	// Direct handler call also returns 410 Gone.
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/v1/insights/ask", nil)
	srv.AskLLM(rec, req)
	if rec.Code != http.StatusGone {
		t.Fatalf("direct AskLLM status = %d, want 410 Gone", rec.Code)
	}
}

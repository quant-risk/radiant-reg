package insights

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

type spyLLMClient struct {
	called bool
}

func (s *spyLLMClient) Chat(_ context.Context, _ []Message) (string, error) {
	s.called = true
	return "should never be called", nil
}

func (s *spyLLMClient) StreamChat(_ context.Context, _ []Message) (<-chan StreamChunk, error) {
	s.called = true
	return nil, nil
}

func (s *spyLLMClient) Model() string { return "spy" }

func TestErrLegacyRetired_Message(t *testing.T) {
	const want = "aposentado na Etapa 0 — ver docs/blueprint/21-ENGINE-IA.md"
	if ErrLegacyRetired == nil || ErrLegacyRetired.Error() != want {
		t.Fatalf("ErrLegacyRetired = %v, want %q", ErrLegacyRetired, want)
	}
}

func TestLegacyLLMClients_FailClosedWithoutHTTP(t *testing.T) {
	httpCalled := false
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		httpCalled = true
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	ctx := context.Background()
	msgs := []Message{{Role: "user", Content: "oi"}}

	minimax := NewMiniMaxChat(MiniMaxConfig{APIKey: "secret", Model: "MiniMax-Text-01", BaseURL: srv.URL})
	if ans, err := minimax.Chat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) || ans != "" {
		t.Fatalf("MiniMaxChat.Chat = (%q, %v), want (\"\", ErrLegacyRetired)", ans, err)
	}
	if ch, err := minimax.StreamChat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) || ch != nil {
		t.Fatalf("MiniMaxChat.StreamChat = (%v, %v), want (nil, ErrLegacyRetired)", ch, err)
	}
	if got := minimax.Model(); got != "" {
		t.Fatalf("MiniMaxChat.Model = %q, want \"\"", got)
	}

	openai := NewOpenAIChat("secret", "gpt-4o-mini", srv.URL)
	if ans, err := openai.Chat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) || ans != "" {
		t.Fatalf("OpenAIChat.Chat = (%q, %v), want (\"\", ErrLegacyRetired)", ans, err)
	}
	if ch, err := openai.StreamChat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) || ch != nil {
		t.Fatalf("OpenAIChat.StreamChat = (%v, %v), want (nil, ErrLegacyRetired)", ch, err)
	}
	if got := openai.Model(); got != "" {
		t.Fatalf("OpenAIChat.Model = %q, want \"\"", got)
	}

	// Struct literals / nil receivers must also fail closed.
	var nilMiniMax *MiniMaxChat
	if _, err := nilMiniMax.Chat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("nil MiniMaxChat.Chat err = %v, want ErrLegacyRetired", err)
	}
	directOpenAI := &OpenAIChat{APIKey: "secret", BaseURL: srv.URL}
	if _, err := directOpenAI.Chat(ctx, msgs); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("direct OpenAIChat.Chat err = %v, want ErrLegacyRetired", err)
	}

	if httpCalled {
		t.Fatal("retired LLM clients must never issue outbound HTTP calls")
	}
}

func TestLegacyLLMService_FailClosed(t *testing.T) {
	ctx := context.Background()
	spy := &spyLLMClient{}
	svc := NewLLMService(LLMConfig{
		LLMClient: spy,
		ConvStore: NewConversationStore(nil),
		RespCache: NewResponseCache(5*time.Minute, 100),
	})

	if ans, err := svc.Ask(ctx, "if-1", "status?"); !errors.Is(err, ErrLegacyRetired) || ans != nil {
		t.Fatalf("LLMService.Ask = (%v, %v), want (nil, ErrLegacyRetired)", ans, err)
	}
	if ch, err := svc.StreamAsk(ctx, "if-1", "status?"); !errors.Is(err, ErrLegacyRetired) || ch != nil {
		t.Fatalf("LLMService.StreamAsk = (%v, %v), want (nil, ErrLegacyRetired)", ch, err)
	}
	if hist, err := svc.GetHistory(ctx, "if-1", 10); !errors.Is(err, ErrLegacyRetired) || hist != nil {
		t.Fatalf("LLMService.GetHistory = (%v, %v), want (nil, ErrLegacyRetired)", hist, err)
	}
	if err := svc.ClearHistory(ctx, "if-1"); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("LLMService.ClearHistory err = %v, want ErrLegacyRetired", err)
	}

	var nilSvc *LLMService
	if _, err := nilSvc.Ask(ctx, "if-1", "status?"); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("nil LLMService.Ask err = %v, want ErrLegacyRetired", err)
	}

	if spy.called {
		t.Fatal("LLMService must never invoke underlying LLMClient")
	}
}

func TestLegacyConversationStoreAndResponseCache_FailClosed(t *testing.T) {
	ctx := context.Background()
	store := NewConversationStore(nil)

	if id, err := store.SaveMessage(ctx, ConversationMessage{IfID: "if-1", Role: "user", Content: "hi"}); !errors.Is(err, ErrLegacyRetired) || id != "" {
		t.Fatalf("SaveMessage = (%q, %v), want (\"\", ErrLegacyRetired)", id, err)
	}
	if hist, err := store.GetHistory(ctx, "if-1", 10); !errors.Is(err, ErrLegacyRetired) || hist != nil {
		t.Fatalf("GetHistory = (%v, %v), want (nil, ErrLegacyRetired)", hist, err)
	}
	if err := store.ClearHistory(ctx, "if-1"); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("ClearHistory err = %v, want ErrLegacyRetired", err)
	}
	if n, err := store.Prune(ctx); !errors.Is(err, ErrLegacyRetired) || n != 0 {
		t.Fatalf("Prune = (%d, %v), want (0, ErrLegacyRetired)", n, err)
	}
	if n, err := store.CountMessages(ctx, "if-1"); !errors.Is(err, ErrLegacyRetired) || n != 0 {
		t.Fatalf("CountMessages = (%d, %v), want (0, ErrLegacyRetired)", n, err)
	}

	cache := NewResponseCache(5*time.Minute, 100)
	cache.Set("if-1", "q", "secret-answer", "model")
	if ans, model, ok := cache.Get("if-1", "q"); ok || ans != "" || model != "" {
		t.Fatalf("ResponseCache.Get = (%q, %q, %v), want miss (\"\", \"\", false)", ans, model, ok)
	}
	cache.Invalidate("if-1")
	if err := cache.Err(); !errors.Is(err, ErrLegacyRetired) {
		t.Fatalf("ResponseCache.Err = %v, want ErrLegacyRetired", err)
	}
}

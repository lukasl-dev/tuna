package tuna

import (
	"context"
	"log/slog"

	"github.com/chromedp/chromedp"
)

func debug(msg string, args ...any) chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug(msg, args...)
		return nil
	})
}

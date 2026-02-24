package tuna

import (
	"context"
	"time"

	"github.com/chromedp/chromedp"
)

const timeout = 10 * time.Second

func withTimeout(action chromedp.Action) chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		waitCtx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		return action.Do(waitCtx)
	})
}

func waitVisible(sel any, opts ...chromedp.QueryOption) chromedp.Action {
	return withTimeout(chromedp.WaitVisible(sel, opts...))
}

func waitReady(sel any, opts ...chromedp.QueryOption) chromedp.Action {
	return withTimeout(chromedp.WaitReady(sel, opts...))
}

func waitEnabled(sel any, opts ...chromedp.QueryOption) chromedp.Action {
	return withTimeout(chromedp.WaitEnabled(sel, opts...))
}

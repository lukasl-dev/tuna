package tutut

import (
	"context"
	"log/slog"

	"github.com/chromedp/chromedp"
)

func login(username, password, totp string) chromedp.Action {
	const url = "https://tiss.tuwien.ac.at/admin/authentifizierung"

	tasks := chromedp.Tasks{
		chromedp.Navigate(url),
		chromedp.WaitVisible("#core\\:loginuserpass", chromedp.ByQuery),

		chromedp.ActionFunc(func(ctx context.Context) error {
			slog.Debug("waiting for TU login fields to be ready")
			return chromedp.Tasks{
				chromedp.WaitReady("username", chromedp.ByID),
				chromedp.WaitReady("password", chromedp.ByID),
			}.Do(ctx)
		}),

		chromedp.ActionFunc(func(ctx context.Context) error {
			slog.Debug("typing username into TU login form", "username", username)
			return chromedp.SendKeys("username", username, chromedp.ByID).Do(ctx)
		}),

		chromedp.ActionFunc(func(ctx context.Context) error {
			slog.Debug("typing password into TU login form", "password", password)
			return chromedp.SendKeys("password", password, chromedp.ByID).Do(ctx)
		}),
	}
	if totp != "" {
		tasks = append(tasks,
			chromedp.ActionFunc(func(ctx context.Context) error {
				slog.Debug("waiting for TU login TOTP field to be ready")
				return chromedp.WaitReady("totp", chromedp.ByID).Do(ctx)
			}),

			chromedp.ActionFunc(func(ctx context.Context) error {
				slog.Debug("typing TOTP into TU login form", "totp", totp)
				return chromedp.SendKeys("totp", totp, chromedp.ByID).Do(ctx)
			}),
		)
	}
	tasks = append(tasks,
		chromedp.ActionFunc(func(ctx context.Context) error {
			slog.Debug("submitting TU login form")
			return chromedp.Submit("samlloginbutton", chromedp.ByID).Do(ctx)
		}),

		chromedp.WaitVisible("a.toolLogout", chromedp.ByQuery),
	)

	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("navigating to TISS login page", "url", url)
		err := tasks.Do(ctx)
		if err != nil {
			slog.Error("failed to navigate to TISS login page", "url", url, "error", err)
			return err
		}
		slog.Debug("TISS login completed")
		return nil
	})
}

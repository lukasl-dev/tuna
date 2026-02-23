package tutut

import (
	"context"
	"log/slog"

	"github.com/chromedp/chromedp"
)

func login(username, password, totp string) chromedp.Action {
	t := chromedp.Tasks{
		waitTULoginFieldsReady(),
		typeTULoginUsername(username),
		typeTULoginPassword(password),
	}
	if totp != "" {
		t = append(t, waitTULoginTOTPReady(), typeTULoginTOTP(totp))
	}
	t = append(t, clickTULoginSubmit())
	return t
}

func waitTULoginFieldsReady() chromedp.Action {
	tasks := chromedp.Tasks{
		chromedp.WaitReady("username", chromedp.ByID),
		chromedp.WaitReady("password", chromedp.ByID),
	}

	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("waiting for TU login fields to be ready")
		return tasks.Do(ctx)
	})
}

func waitTULoginTOTPReady() chromedp.Action {
	tasks := chromedp.Tasks{
		chromedp.WaitReady("totp", chromedp.ByID),
	}

	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("waiting for TU login TOTP field to be ready")
		return tasks.Do(ctx)
	})
}
func typeTULoginUsername(username string) chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("typing username into TU login form", "username", username)
		return chromedp.SendKeys("username", username, chromedp.ByID).Do(ctx)
	})
}

func typeTULoginPassword(password string) chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("typing password into TU login form", "password", password)
		return chromedp.SendKeys("password", password, chromedp.ByID).Do(ctx)
	})
}

func typeTULoginTOTP(totp string) chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("typing TOTP into TU login form", "totp", totp)
		return chromedp.SendKeys("totp", totp, chromedp.ByID).Do(ctx)
	})
}

func clickTULoginSubmit() chromedp.Action {
	return chromedp.ActionFunc(func(ctx context.Context) error {
		slog.Debug("submitting TU login form")
		return chromedp.Submit("samlloginbutton", chromedp.ByID).Do(ctx)
	})
}

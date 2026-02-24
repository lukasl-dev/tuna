package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
)

type root struct {
	LogLevel string `short:"l" help:"Set log level" enum:"debug,info,warn,error" default:"info"`

	ListGroups     listGroups     `cmd:"list-groups" help:"List groups of a course"`
	RegisterCourse registerCourse `cmd:"register-course" help:"Register in a course"`
	RegisterGroup  registerGroup  `cmd:"register-group" help:"Register in a group"`
}

func (r root) configure() error {
	level := slog.LevelInfo
	switch r.LogLevel {
	case "debug":
		level = slog.LevelDebug
	case "info":
		level = slog.LevelInfo
	case "warn":
		level = slog.LevelWarn
	case "error":
		level = slog.LevelError
	default:
		return fmt.Errorf("unknown log level %q", r.LogLevel)
	}

	handler := slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
		Level: level,
	})
	log := slog.New(handler)
	slog.SetDefault(log)

	return nil
}

func credentials(username, password, totpURL string) (tuna.LoginCredentials, error) {
	var code string

	if totpURL != "" {
		slog.Debug("parsing TOTP URL")
		key, err := otp.NewKeyFromURL(totpURL)
		if err != nil {
			slog.Error("failed to parse TOTP URL", "error", err)
			return tuna.LoginCredentials{}, err
		}

		slog.Debug("generating TOTP code")
		code, err = totp.GenerateCode(key.Secret(), time.Now())
		if err != nil {
			slog.Error("failed to generate TOTP code", "error", err)
			return tuna.LoginCredentials{}, nil
		}
	}

	creds := tuna.LoginCredentials{
		Username: username,
		Password: password,
		TOTP:     code,
	}
	return creds, nil
}

func chromedpContext(headless bool) (context.Context, func()) {
	const runTimeout = 10 * time.Second

	allocatorCtx, cancelAllocator := chromedp.NewExecAllocator(context.TODO(),
		append(
			chromedp.DefaultExecAllocatorOptions[:],
			chromedp.Flag("headless", headless),
		)...,
	)

	ctx, cancel := chromedp.NewContext(allocatorCtx)
	timeoutCtx, cancelTimeout := context.WithTimeout(ctx, runTimeout)

	return timeoutCtx, func() {
		cancelTimeout()
		cancelAllocator()
		cancel()
	}
}

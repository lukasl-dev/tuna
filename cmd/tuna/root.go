package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"strings"
	"time"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
)

type root struct {
	LogLevel string        `short:"l" help:"Set log level" enum:"debug,info,warn,error" default:"info"`
	Postpone *postponeTime `help:"Run command at this timestamp (execution starts 5s after it)" placeholder:"YYYY-MM-DD HH:MM:SS+TZ"`
	Retries  uint          `help:"Retry failed command this many times" default:"0"`

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

func (r root) waitPostpone() error {
	if r.Postpone == nil {
		return nil
	}

	const postponeDelay = 5 * time.Second

	target := r.Postpone.Time.Add(postponeDelay)
	now := time.Now()
	if !target.After(now) {
		slog.Warn("postpone time already passed, running immediately", "postpone", r.Postpone.Time.Format(time.RFC3339Nano), "target", target.Format(time.RFC3339Nano))
		return nil
	}

	delay := time.Until(target)
	slog.Info("postponing command execution", "postpone", r.Postpone.Time.Format(time.RFC3339Nano), "target", target.Format(time.RFC3339Nano), "delay", delay)

	fired := make(chan struct{})
	timer := time.AfterFunc(delay, func() {
		close(fired)
	})
	defer timer.Stop()

	<-fired

	if remaining := time.Until(target); remaining > 0 {
		time.Sleep(remaining)
	}

	return nil
}

func (r root) runWithRetries(run func() error) error {
	maxAttempts := int(r.Retries) + 1
	var lastErr error

	for attempt := 1; attempt <= maxAttempts; attempt++ {
		err := run()
		if err == nil {
			if attempt > 1 {
				slog.Info("command succeeded after retry", "attempt", attempt, "max_attempts", maxAttempts)
			}
			return nil
		}

		lastErr = err
		if attempt < maxAttempts {
			slog.Warn("command failed, retrying", "attempt", attempt, "max_attempts", maxAttempts, "error", err)
		}
	}

	return fmt.Errorf("command failed after %d attempt(s): %w", maxAttempts, lastErr)
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
	allocatorCtx, cancelAllocator := chromedp.NewExecAllocator(context.TODO(),
		append(
			chromedp.DefaultExecAllocatorOptions[:],
			chromedp.Flag("headless", headless),
		)...,
	)

	ctx, cancel := chromedp.NewContext(allocatorCtx)

	return ctx, func() {
		cancelAllocator()
		cancel()
	}
}

type postponeTime struct {
	time.Time
}

func (p *postponeTime) UnmarshalText(text []byte) error {
	input := strings.TrimSpace(string(text))
	layouts := []string{
		"2006-01-02 15:04:05-07:00",
		time.RFC3339,
	}

	for _, layout := range layouts {
		parsed, err := time.Parse(layout, input)
		if err == nil {
			p.Time = parsed
			return nil
		}
	}

	return fmt.Errorf("invalid postpone timestamp %q (expected YYYY-MM-DD HH:MM:SS+TZ or RFC3339)", input)
}

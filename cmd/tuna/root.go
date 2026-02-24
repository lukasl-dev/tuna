package main

import (
	"fmt"
	"log/slog"
	"os"
)

type root struct {
	LogLevel string `short:"l" help:"Set log level" enum:"debug,info,warn,error" default:"info"`

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

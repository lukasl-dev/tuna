package main

import (
	"log/slog"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
)

type registerCourse struct {
	Headless bool `short:"h" help:"Headless" default:"true"`

	Username string `short:"u" help:"TU Username" required:"true"`
	Password string `short:"p" help:"TU Password" required:"true"`
	TOTP     string `short:"t" help:"TOTP Url" default:""`

	Semester string `short:"s" help:"The semester to register for" required:"true"`
	Course   string `short:"c" help:"The course to register for" required:"true"`
}

func (r registerCourse) Run() error {
	ctx, cancel := chromedpContext(r.Headless)
	defer cancel()

	creds, err := credentials(r.Username, r.Password, r.TOTP)
	if err != nil {
		slog.Error("failed to build credentials", "error", err)
		return err
	}

	err = chromedp.Run(ctx, chromedp.Tasks{
		tuna.Login(creds),
		tuna.RegisterCourse(r.Semester, r.Course),
	})
	if err != nil {
		captureErrorScreenshot(ctx, "register-course")
		slog.Error("flow failed", "error", err)
		return err
	}

	return nil
}
